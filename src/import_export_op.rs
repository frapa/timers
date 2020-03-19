use std::path::Path;

use csv;
use timers;
use chrono;
use itertools::Itertools;
use chrono::TimeZone;
use crate::util::parse_time;

pub fn export_command(matches: &clap::ArgMatches) {
    let object = matches.value_of("OBJECT").unwrap();

    let output: Box<dyn std::io::Write> = match matches.value_of("output") {
        Some(output_path_str) => {
            let output_path = Path::new(output_path_str);
            match std::fs::File::create(output_path) {
                Ok(file) => Box::new(file),
                Err(err) => {
                    println!("Impossible to write file '{}': {}", output_path_str, err);
                    return;
                }
            }
        },
        None => Box::new(std::io::stdout()),
    };

    let delimiter = matches.value_of("delimiter").unwrap();

    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter.bytes().next().unwrap())
        .from_writer(output);

    // Header
    if object == "logs" {
        writer.write_record(
            &["Task ID", "Task name", "Begin (UTC)", "End (UTC)", "Duration (hours)"]
        ).unwrap();
    } else {
        writer.write_record(
            &["Task ID", "Task name", "Logs", "Duration (hours)"]
        ).unwrap();
    }

    let from = match matches.value_of("from") {
        Some(from) => match parse_time(from) {
            Some(from) => from,
            None => return,
        },
        None => chrono::Utc.ymd(1900, 1, 1).and_hms(0, 0, 0),
    };
    let to = match matches.value_of("to") {
        Some(to) => match parse_time(to) {
            Some(to) => to,
            None => return,
        }
        None => chrono::Utc.ymd(2100, 1, 1).and_hms(0, 0, 0),
    };

    match timers::get_all_tasks_between(from, to) {
        Ok(tasks) => for id in tasks.keys().sorted() {
            let task = tasks.get(id).unwrap();
            if object == "logs" {
                write_task_logs(&mut writer, task);
            } else {
                write_task(&mut writer, task);
            }
        },
        Err(err) => println!("Error retrieving tasks: {}", err)
    }

    writer.flush().unwrap();
}

fn write_task<T>(writer: &mut csv::Writer<T>, task: &timers::Task)
    where
        T: std::io::Write,
{
    writer.write_record(&[
        task.id.to_string().as_str(),
        task.name.as_str(),
        task.logs.len().to_string().as_str(),
        (task.duration().num_seconds() as f64 / 3600.).to_string().as_str(),
    ]).unwrap();
}

fn write_task_logs<T>(writer: &mut csv::Writer<T>, task: &timers::Task)
    where
        T: std::io::Write
{
    for log in task.logs.iter() {
        write_log(writer, task, log);
    }
}

fn write_log<T>(writer: &mut csv::Writer<T>, task: &timers::Task, log: &timers::Log)
    where
        T: std::io::Write,
{
    let end_str = match log.end {
        Some(end) => end.to_rfc3339(),
        None => String::new(),
    };

    writer.write_record(&[
        task.id.to_string().as_str(),
        task.name.as_str(),
        log.start.to_rfc3339().as_str(),
        end_str.as_str(),
        (log.duration().num_seconds() as f64 / 3600.).to_string().as_str(),
    ]).unwrap();
}