use std::ops::Add;
use std::{thread, time};

use chrono;
use colored::*;
use term_size;

use super::util::*;

use itertools::Itertools;
use std::collections::HashMap;
use timers;

pub fn log_command(matches: &clap::ArgMatches) {
    // Cannot panic as the argument parser already ensures it exist
    let task = matches.value_of("TASK").unwrap();

    if task.len() == 0 {
        println!("Cannot create empty task.");
    }

    let time = match matches.value_of("AT") {
        Some(raw_time) => match parse_time(raw_time) {
            Some(time) => time,
            None => return,
        },
        None => chrono::Utc::now(),
    };

    if !confirm_stop_current(time) {
        return;
    }

    if task.starts_with('@') {
        // This will strip multiple @ if present,
        // currently it's not worth to fix this behavior
        let task_id_result = task.trim_start_matches('@').parse::<u32>();

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
                    }
                    // stopped current, continue with new
                    _ => true,
                }
            }
        }
        Err(err) => {
            println!("Error finding current task: {}", err);
            false
        }
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
            }
        },
        None => 1.,
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
                        return;
                    }
                },
                None => None,
            };

            let start = chrono::Utc::today().and_hms(0, 0, 0);
            let end = chrono::Utc::today()
                .and_hms(0, 0, 0)
                .add(chrono::Duration::days(1));
            print_timeline(start, end, length);
        } else {
            match timers::get_current_log_task() {
                Ok(task) => match task {
                    Some(task) => print_status(&task),
                    None => print!("You are not logging on any task."),
                },
                Err(err) => print!("Error finding current task: {}", err),
            };
        }

        if matches.occurrences_of("watch") == 0 {
            break;
        }

        thread::sleep(time::Duration::from_secs((minutes * 60.) as u64));
    }
}

fn get_logged_hours(tasks: &HashMap<u32, timers::Task>) -> f64 {
    let mut logged = chrono::Duration::zero();
    for task in tasks.values() {
        logged = logged + task.duration();
    }
    let logged_hours = logged.num_seconds() as f64 / 3600.;
    logged_hours
}

fn get_term_height() -> f64 {
    (match term_size::dimensions() {
        Some((_, h)) => h,
        None => 24,
    }) as f64
}

fn print_timeline(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    length: Option<f64>,
) {
    let tasks = match timers::get_all_tasks_between(start, end) {
        Ok(tasks) => tasks,
        Err(err) => {
            println!("Error computing tasks length: {}", err);
            return;
        }
    };

    let total_logged_hours = get_logged_hours(&tasks);
    let height = get_term_height() - 1.;

    let mut total = length.unwrap_or(8.);
    if total_logged_hours > total {
        total = total_logged_hours;
    }

    let unit = height as f64 / total;

    print_vertical_timeline(&tasks, unit);
}

fn print_vertical_timeline(tasks: &HashMap<u32, timers::Task>, unit: f64) {
    let mut cumulative = 0 as f64;
    for id in tasks.keys().sorted() {
        let task = tasks.get(id).unwrap();
        let size = task.duration().num_seconds() as f64 / 3600. * unit;

        let int_size = (size - cumulative).round() as i32;

        let duration = timers::format_duration(task.duration());
        let start = task.logs.last().unwrap().start.format("%H:%M").to_string();
        let end = match task.logs.last().unwrap().end {
            Some(end) => end.format("%H:%M").to_string(),
            None => String::from(""),
        };
        match int_size {
            0 => println!(
                " ◇ {} -> {} {} [{}]",
                start,
                end,
                task.name.bold(),
                duration,
            ),
            1 => println!(
                " ◇ {} -> {} {} [{}]",
                start,
                end,
                task.name.bold(),
                duration,
            ),
            2 => {
                println!(" ◇ {} {} [{}]", start, task.name.bold(), duration);
                println!(" | {}", end);
            }
            3 => {
                println!(" ◇ {}", task.logs.last().unwrap().start.format("%H:%M"));
                println!(" | {} [{}]", task.name.bold(), duration);
                println!(" ◆ {}", end);
            }
            n => {
                println!(" ◇ {}", task.logs.last().unwrap().start.format("%H:%M"));
                println!(" | {}", task.name.bold());
                println!(" | {}", duration);
                for _ in 0..(n - 4) {
                    println!(" |");
                }
                println!(" ◆ {}", end);
            }
        };
    }
}

pub fn stop_command(matches: &clap::ArgMatches) {
    let time = match matches.value_of("AT") {
        Some(raw_time) => match parse_time(raw_time) {
            Some(time) => time,
            None => return,
        },
        None => chrono::Utc::now(),
    };

    match timers::stop_current_task_at(time) {
        Ok(task) => print_status(&task),
        Err(timers::Error::Value(_)) => {
            println!("Cannot stop because you're not logging on any task.")
        }
        Err(err) => println!("An stopping task: {}", err),
    }
}
