use std::io::prelude::*;

use colored::*;
use chrono;
use chrono::{Timelike, Datelike};

pub fn user_input(prompt: &str) -> String {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();

    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer).unwrap();

    answer
}

pub fn print_status(task: &timers::Task) {
    println!(
        "{} {}\nstatus: {}\ntime: {}",
        format!("@{}:", task.id).yellow().bold(),
        task.name.red().bold(),
        task.status_text().bold(),
        timers::format_duration(task.duration()).bold()
    );
//    println!(
//        "[{}] @{}: {}\n---\nlogs: {}\ntotal: {}",
//        task.status_text(),
//        task.id,
//        task.name,
//        task.logs.len(),
//        timers::format_duration(task.duration())
//    )
}

pub fn parse_time(raw_time: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let mut d = chrono::NaiveDate::from_ymd(2020, 1, 1);
    d = d.with_day(d.day()-1).unwrap();
    println!("{}", d);

    if raw_time.starts_with("y") {
        let stripped_time = raw_time.get(1..).unwrap();
        return match parse_time(stripped_time) {
            Some(datetime) =>
                Some(datetime.with_day(datetime.day()-1).unwrap()),
            None => None
        }
    }

    if let Some(datetime) = try_parse_time(raw_time, "%H:%M") {
        return Some(datetime)
    }

    if let Some(datetime) = try_parse_time(raw_time, "%H:%M:%S") {
        return Some(datetime)
    }

    println!("Time format '{}' not understood", raw_time);
    None
}

pub fn try_parse_time(raw_time: &str, fmt: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    match chrono::NaiveTime::parse_from_str(raw_time, fmt) {
        Ok(parsed_time) => {
            let datetime = chrono::Local::now()
                .with_hour(parsed_time.hour()).unwrap()
                .with_minute(parsed_time.minute()).unwrap()
                .with_second(parsed_time.second()).unwrap()
                .with_nanosecond(parsed_time.nanosecond()).unwrap()
                .with_timezone(&chrono::Utc);
            println!("{}", datetime.to_rfc3339());
            Some(datetime)
        },
        Err(_) => None,
    }
}