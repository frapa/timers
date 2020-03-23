use std::ops::{Sub, Add};

use colored::*;
use chrono::{Timelike, Datelike};

use timers;

pub fn report_days_command(matches: &clap::ArgMatches) {
    if !matches.is_present("plain") {
        println!("{:<12} {:<14} {}", "DAY", "TIME LOGGED", "TASKS");
        println!("{}", "-".repeat(34));
    }

    let week_offset = chrono::Local::now().weekday().num_days_from_monday() as i64;
    let week_start = chrono::Local::now()
        .with_hour(0).unwrap()
        .with_minute(0).unwrap()
        .with_second(0).unwrap()
        .with_nanosecond(0).unwrap()
        .sub(chrono::Duration::days(week_offset))
        .with_timezone(&chrono::Utc);

    for i in 0..7 {
        let start = week_start.add(chrono::Duration::days(i));
        let end = week_start.add(chrono::Duration::days(i+1));

        let local_start = start.with_timezone(&chrono::Local);
        let tasks = timers::get_all_tasks_between(start, end)
            .unwrap_or_else(|err| {
                println!("Error retrieving tasks: {}", err);
                std::process::exit(2);
            });

        let duration = timers::get_total_duration(start, end)
            .unwrap_or_else(|err| {
                println!("Error computing duration: {}", err);
                std::process::exit(2);
            });

        println!(
            "{:<12} {:<14} {}",
            if i < 5 {
                local_start.format("%A").to_string().green()
            } else {
                local_start.format("%A").to_string().red()
            },
            timers::format_duration(duration),
            tasks.len(),
        )
    }

    if !matches.is_present("plain") {
        println!("{}", "-".repeat(34));

        let week_end = week_start.add(chrono::Duration::weeks(1));

        let tasks = timers::get_all_tasks_between(week_start, week_end)
            .unwrap_or_else(|err| {
                println!("Error retrieving tasks: {}", err);
                std::process::exit(2);
            });

        let duration = timers::get_total_duration(week_start, week_end)
            .unwrap_or_else(|err| {
                println!("Error computing duration: {}", err);
                std::process::exit(2);
            });

        println!(
            "{:<12} {:<14} {}",
            "Total",
            timers::format_duration(duration),
            tasks.len(),
        )
    }
}