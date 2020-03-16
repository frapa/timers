use std::{thread, time};
use std::io::prelude::*;
use std::ops::Add;

use colored::*;
use chrono;
use term_size;

use super::util::*;

use timers;
use itertools::Itertools;

pub fn log_command(matches: &clap::ArgMatches) {
    // Cannot panic as the argument parser already ensures it exist
    let task = matches.value_of("TASK").unwrap();

    if task.len() == 0 {
        println!("Cannot create empty task.");
    }

    let time = match matches.value_of("AT") {
        Some(raw_time) => match parse_time(raw_time) {
            Some(time) => time,
            None => return
        },
        None => chrono::Utc::now(),
    };

    if !confirm_stop_current(time) {
        return
    }

    if task.starts_with('@') {
        // This will strip multiple @ if present,
        // currently it's not worth to fix this behavior
        let task_id_result = task
            .trim_start_matches('@')
            .parse::<u32>();

        match task_id_result {
            Ok(task_id) => match timers::log_task_at(task_id, time) {
                Ok(task) => print_status(&task),
                Err(err) => println!("Error logging on task: {}", err),
            },
            Err(_) => println!("'{}' is an invalid task ID", task),
        };
    } else {
        match timers::create_log_task_at(task, time) {
            Ok(task) => print_status(&task),
            Err(err) => println!("Error creating task: {}", err),
        }
    }
}

fn confirm_stop_current(time: chrono::DateTime<chrono::Utc>) -> bool {
    match timers::get_current_log_task() {
        Ok(Some(task)) => {
            println!(
                "Currently logging on task {} {}",
                format!("@{}:", task.id).yellow().bold(),
                task.name.red().bold(),
            );
            let answer = user_input("Do you want to start the new task? [y/n] ");

            if answer.trim() == "n" || answer.trim() == "no" {
                println!("aborting");
                false
            } else {
                match timers::stop_current_task_at(time) {
                    Err(err) => {
                        println!("Error stopping current task: {}", err);
                        false
                    },
                    // stopped current, continue with new
                    _ => true,
                }
            }
        },
        Err(err) => {
            println!("Error finding current task: {}", err);
            false
        },
        // no current task, continue with new
        _ => true,
    }
}

pub fn status_command(matches: &clap::ArgMatches) {
    let minutes = match matches.value_of("watch") {
        Some(val) => match parse_float(val) {
            Ok(val) => val,
            Err(_) => {
                println!("Invalid watch interval '{}'", val);
                return;
            },
        },
        None => 1f64,
    } as f64;

    loop {
        // clear screen
        if matches.occurrences_of("watch") != 0 {
            print!("\x1B[H\x1B[2J\r");
        }

        if matches.is_present("timeline") {
            let length = match matches.value_of("total") {
                Some(total) => match parse_float(total) {
                    Ok(total) => Some(total),
                    Err(_) => {
                        println!("Invalid value for total: '{}'", total);
                        return
                    },
                },
                None => None,
            };

            let start = chrono::Utc::today()
                .and_hms(0, 0, 0);
            let end = chrono::Utc::today()
                .and_hms(0, 0, 0)
                .add(chrono::Duration::days(1));
            print_timeline(start, end, length);
        }

        match timers::get_current_log_task() {
            Ok(task) => match task {
                Some(task) => print_status(&task),
                None => print!("You are not logging on any task."),
            },
            Err(err) => print!("Error finding current task: {}", err),
        };
        std::io::stdout().flush().unwrap();

        if matches.occurrences_of("watch") == 0 {
            break
        }

        thread::sleep(time::Duration::from_secs((minutes * 60f64) as u64));
    }
}

fn print_timeline(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    length: Option<f64>,
) {
    let width = match term_size::dimensions() {
        Some((w, _)) => w,
        None => 20,
    };

    let total = match length {
        Some(len) => len,
        None => 8f64,
    };

    let unit = width as f64 / total;

    let tasks = match timers::get_all_tasks_between(start, end) {
        Ok(tasks) => tasks,
        Err(err) => {
            println!("Error computing tasks length: {}", err);
            return;
        }
    };

    let mut total_logged = chrono::Duration::zero();
    for task in tasks.values() {
        total_logged = total_logged + task.duration();
    }

    let mut cumulative = 0 as f64;
    let mut symbol = 0;
    for id in tasks.keys().sorted() {
        let logged = tasks.get(id).unwrap().duration().num_seconds() as f64 / 3600. * unit;

        for _ in (cumulative as i32)..((cumulative + logged) as i32) {
            match symbol {
                0 => print!("█"),
                1 => print!("▒"),
                2 => print!("▓"),
                _ => {},
            };
        }

        cumulative += logged;
        symbol = (symbol + 1) % 3;
    }

    for _ in (cumulative as usize)..width {
        print!("░");
    }

    cumulative = 0 as f64;
    symbol = 0;
    for id in tasks.keys().sorted() {
        let task = tasks.get(id).unwrap();
        let logged = task.duration().num_seconds() as f64 / 3600f64 * unit;
        let int_logged_len = (cumulative + logged) as usize - cumulative as usize;

        if logged > 12 as f64 {
            if int_logged_len < task.name.len() {
                let truncated_name: String = task.name.chars().take(int_logged_len-4).collect();
                print!("{}... ", truncated_name);
            } else {
                print!("{}", task.name);
                let fill = int_logged_len - task.name.len();
                for _ in 0..fill {
                    print!(" ");
                }
            }
        } else {
            for _ in 0..int_logged_len {
                print!(" ");
            }
        }

        cumulative += logged;
        symbol = (symbol + 1) % 3;
    }

    println!("\n");
}

pub fn stop_command(matches: &clap::ArgMatches) {
    let time = match matches.value_of("AT") {
        Some(raw_time) => match parse_time(raw_time) {
            Some(time) => time,
            None => return
        },
        None => chrono::Utc::now(),
    };

    match timers::stop_current_task_at(time) {
        Ok(task) => print_status(&task),
        Err(timers::Error::Value(_)) => println!(
            "Cannot stop because you're not logging on any task."
        ),
        Err(err) => println!("An stopping task: {}", err)
    }
}