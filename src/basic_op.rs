use colored::*;
use chrono;

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

pub fn status_command() {
    match timers::get_current_log_task() {
        Ok(task) => match task {
            Some(task) => print_status(&task),
            None => println!("You are not logging on any task.")
        },
        Err(err) => println!("Error finding current task: {}", err),
    }
}

pub fn stop_command() {
    match timers::stop_current_task() {
        Ok(task) => print_status(&task),
        Err(timers::Error::Value(_)) => println!(
            "Cannot stop because you're not logging on any task."
        ),
        Err(err) => println!("An stopping task: {}", err)
    }
}