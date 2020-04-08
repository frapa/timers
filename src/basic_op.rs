use std::ops::Add;
use std::{thread, time};

use chrono;
use colored::*;
use term_size;

use super::util::*;

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
            let start = chrono::Local::today()
                .and_hms(0, 0, 0)
                .with_timezone(&chrono::Utc);
            let end = chrono::Local::today()
                .and_hms(0, 0, 0)
                .add(chrono::Duration::days(1))
                .with_timezone(&chrono::Utc);
            print_timeline(start, end);
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

fn get_term_height() -> f64 {
    (match term_size::dimensions() {
        Some((_, h)) => h,
        None => 24,
    }) as f64
}

fn print_timeline(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) {
    let mut logs = match timers::get_all_logs_between(start, end) {
        Ok(logs) => logs,
        Err(err) => {
            println!("Error while retrieving logs: {}", err);
            return;
        }
    };

    if logs.len() == 0 {
        println!("There are no logs.");
        return;
    }

    clip_logs(start, end, &mut logs).unwrap();

    let unit = compute_unit(&logs).unwrap();

    let mut cumulative = 0.;
    let mut printed_size = 0;
    for (task, log) in logs.iter() {
        let size = log.duration().num_seconds() as f64 * unit;
        cumulative += size;

        let print_size = cumulative as i32 - printed_size;

        print_timeline_log(&task.name, print_size, log.start, log.end());

        printed_size += print_size;
    }
}

fn clip_logs(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    logs: &mut Vec<(timers::Task, timers::Log)>
) -> Result<(), timers::Error> {
    if logs.len() == 0 {
        return Err(timers::Error::Value(timers::ValueError::new("There are no logs.")));
    }

    {
        let mut first = logs.first_mut().unwrap();
        let logs_start = first.1.start;

        if logs_start < start {
            first.1.start = start;
        }
    }

    {
        let mut last = logs.last_mut().unwrap();
        let logs_end = last.1.end();

        if logs_end > end {
            last.1.end = Some(end);
        }
    }

    Ok(())
}

fn compute_unit(logs: &Vec<(timers::Task, timers::Log)>) -> Result<f64, timers::Error> {
    if logs.len() == 0 {
        return Err(timers::Error::Value(timers::ValueError::new("There are no logs.")));
    }

    let start = logs.first().unwrap().1.start;
    let end = logs.last().unwrap().1.end();

    let timespan = end - start;
    let height = get_term_height() - 1.;

    let unit = height as f64 / timespan.num_seconds() as f64;

    Ok(unit)
}

fn print_timeline_log(
    name: &str,
    size: i32,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) {
    let duration = timers::format_duration(end - start);
    let start = start.with_timezone(&chrono::Local)
        .format("%H:%M").to_string();
    let end = end.with_timezone(&chrono::Local)
        .format("%H:%M").to_string();
    match size {
        0 => println!(
            " ◇ {} -> {} {} [{}]",
            start,
            end,
            name.bold(),
            duration,
        ),
        1 => println!(
            " ◇ {} -> {} {} [{}]",
            start,
            end,
            name.bold(),
            duration,
        ),
        2 => {
            println!(" ◇ {} {} [{}]", start, name.bold(), duration);
            println!(" | {}", end);
        }
        3 => {
            println!(" ◇ {}", start);
            println!(" | {} [{}]", name.bold(), duration);
            println!(" ◆ {}", end);
        }
        n => {
            println!("{} {}", n, size);
            println!(" ◇ {}", start);
            println!(" | {}", name.bold());
            println!(" | {}", duration);
            for _ in 0..(n - 4) {
                println!(" |");
            }
            println!(" ◆ {}", end);
        }
    };
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
