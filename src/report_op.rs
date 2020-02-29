use std::ops::{Sub, Add};

use colored::*;
use chrono::{Timelike, Datelike};

use timers;

pub fn report_days_command(matches: &clap::ArgMatches) {
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

        match timers::get_total_duration(start, end) {
            Ok(duration) => println!(
                "{}: {}",
                if i < 5 {
                    local_start.format("%a").to_string().green()
                } else {
                    local_start.format("%a").to_string().red()
                },
                timers::format_duration(duration)
            ),
            Err(err) => println!("Error computing duration: {}", err)
        }
    }

    if !matches.is_present("no-tot") {
        let week_end = week_start.add(chrono::Duration::weeks(1));

        match timers::get_total_duration(week_start, week_end) {
            Ok(duration) => println!("---\ntotal: {}", timers::format_duration(duration)),
            Err(err) => println!("Error computing duration: {}", err)
        }
    }
}