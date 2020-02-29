use std::io::prelude::*;
use std::num::ParseIntError;
use std::ops::Add;

use colored::*;
use chrono;
use chrono::{Timelike, TimeZone};
use chrono::offset::LocalResult::Single;

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
}

pub fn parse_time(raw_time: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    if raw_time.starts_with("y") {
        let stripped_time = raw_time.get(1..).unwrap();
        return match parse_time(stripped_time) {
            Some(datetime) => Some(datetime - chrono::Duration::days(1)),
            None => None
        }
    }

    if raw_time.starts_with("-") || raw_time.starts_with("+") {
        let sign = raw_time.get(..1).unwrap();
        let stripped_time = raw_time.get(1..).unwrap();
        return match parse_duration(stripped_time) {
            Some(duration) =>
                Some(chrono::Utc::now() + if sign == "+" {duration} else {-duration}),
            None => None
        }
    }

    if let Some(datetime) = try_parse_time(raw_time, "%H:%M") {
        return Some(datetime)
    }

    if let Some(datetime) = try_parse_time(raw_time, "%H:%M:%S") {
        return Some(datetime)
    }

    if let Some(datetime) = try_parse_datetime(raw_time, "%Y-%m-%d %H:%M") {
        return Some(datetime)
    }

    if let Some(datetime) = try_parse_datetime(raw_time, "%Y-%m-%d %H:%M:%S") {
        return Some(datetime)
    }

    println!("Time format '{}' not understood", raw_time);
    None
}

pub fn try_parse_time(raw_time: &str, fmt: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    match chrono::NaiveTime::parse_from_str(raw_time, fmt) {
        Ok(parsed_time) => {
            let datetime = chrono::Local::now()
                .with_nanosecond(parsed_time.nanosecond()).unwrap()
                .with_second(parsed_time.second()).unwrap()
                .with_minute(parsed_time.minute()).unwrap()
                .with_hour(parsed_time.hour()).unwrap()
                .with_timezone(&chrono::Utc);
            Some(datetime)
        },
        Err(_) => None,
    }
}

pub fn try_parse_datetime(raw_datetime: &str, fmt: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    match chrono::NaiveDateTime::parse_from_str(raw_datetime, fmt) {
        Ok(parsed_datetime) => {
            match chrono::Local.from_local_datetime(&parsed_datetime) {
                Single(datetime) => Some(datetime.with_timezone(&chrono::Utc)),
                _ => None,
            }
        }
        Err(_) => None,
    }
}

pub fn parse_duration(raw_duration: &str) -> Option<chrono::Duration> {
    return if raw_duration.to_string().contains(":") {
        let split: Vec<&str> = raw_duration.splitn(2, ":").collect();
        let raw_hours = split[0];
        let raw_minutes = split[1];

        let mut duration  = chrono::Duration::seconds(0);
        match parse_int(raw_hours) {
            Ok(hours) => duration = duration.add(chrono::Duration::hours(hours)),
            Err(_) => {
                println!("Duration format '{}' not understood", raw_duration);
                return None;
            },
        }

        match parse_int(raw_minutes) {
            Ok(minutes) => duration = duration.add(chrono::Duration::minutes(minutes)),
            Err(_) => {
                println!("Duration format '{}' not understood", raw_duration);
                return None;
            },
        }

        Some(duration)
    } else {
        match parse_int(raw_duration) {
            Ok(minutes) => Some(chrono::Duration::minutes(minutes)),
            Err(_) => {
                println!("Duration format '{}' not understood", raw_duration);
                None
            },
        }
    }
}

fn parse_int(text: &str) -> Result<i64, ParseIntError> {
    Ok(text.trim().parse::<i64>()?)
}