use std::collections::HashMap;
use std::collections::BTreeMap;
use std::ops::Add;
use std::path::PathBuf;

use chrono;
use dirs;
use fs_extra;

mod errors;
pub use errors::{Error, ValueError};
mod repo;
pub use repo::{Log, Repo, Task, TaskStatus};

fn data_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap();
    path.push("timers_time_logs");
    path
}

fn get_repo() -> Result<Repo, Error> {
    let path = data_path();

    // Temporary code: migrate old folder if it exists
    // -------------
    let mut old_path = dirs::home_dir().unwrap();
    old_path.push(".timers");
    if old_path.exists() {
        println!("Warning: Migrating files to new data location.");

        // move to newpath
        fs_extra::dir::move_dir(
            &old_path,
            dirs::data_dir().unwrap(),
            &fs_extra::dir::CopyOptions::new(),
        )
        .unwrap();

        let mut moved_path = dirs::data_dir().unwrap();
        moved_path.push(".timers");
        std::fs::rename(&moved_path, &path).unwrap();
    }
    // -------------

    // ensure folder exists
    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(Repo { path })
}

pub fn create_task(name: &str) -> Result<Task, Error> {
    let repo = get_repo()?;
    Ok(repo.create_task(name)?)
}

pub fn log_task_at(id: u32, at: chrono::DateTime<chrono::Utc>) -> Result<Task, Error> {
    let repo = get_repo()?;
    let mut task = repo.get_task(id)?;
    repo.log_task(&mut task, at)?;
    Ok(task)
}

pub fn log_task(id: u32) -> Result<Task, Error> {
    Ok(log_task_at(id, chrono::Utc::now())?)
}

pub fn create_log_task_at(name: &str, at: chrono::DateTime<chrono::Utc>) -> Result<Task, Error> {
    let repo = get_repo()?;
    let mut task = repo.create_task(name)?;
    repo.log_task(&mut task, at)?;
    Ok(task)
}

pub fn create_log_task(name: &str) -> Result<Task, Error> {
    Ok(create_log_task_at(name, chrono::Utc::now())?)
}

pub fn get_current_log_task() -> Result<Option<Task>, Error> {
    let repo = get_repo()?;
    let mut tasks = repo.list_tasks()?;

    let mut found_id: Option<u32> = None;
    for (id, task) in tasks.iter() {
        if task.logging {
            found_id = Some(*id);
            break;
        }
    }

    match found_id {
        Some(id) => Ok(tasks.remove(&id)),
        None => Ok(None),
    }
}

pub fn stop_current_task_at(at: chrono::DateTime<chrono::Utc>) -> Result<Task, Error> {
    let repo = get_repo()?;

    match get_current_log_task()? {
        Some(mut task) => {
            repo.stop_task(&mut task, at)?;
            Ok(task)
        }
        None => Err(Error::Value(ValueError::new(
            "Not task currently being logged.",
        ))),
    }
}

pub fn stop_current_task() -> Result<Task, Error> {
    Ok(stop_current_task_at(chrono::Utc::now())?)
}

pub fn get_all_tasks() -> Result<HashMap<u32, Task>, Error> {
    let repo = get_repo()?;
    Ok(repo.list_tasks()?)
}

pub fn get_all_tasks_between(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) -> Result<HashMap<u32, Task>, Error> {
    let repo = get_repo()?;
    let tasks = repo.list_tasks()?;

    let mut filtered = HashMap::new();
    for (id, task) in tasks {
        for log in task.logs.iter() {
            if log.start.ge(&start) && log.start.le(&end) {
                filtered.insert(id, task);
                break;
            }

            let log_end = match log.end {
                Some(end) => end,
                None => chrono::Utc::now(),
            };
            if log_end.ge(&start) && log_end.le(&end) {
                filtered.insert(id, task);
                break;
            }
        }
    }

    Ok(filtered)
}

// Returns all logs between start and end,
// sorted chronologically
pub fn get_all_logs_between(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<(Task, Log)>, Error> {
    let tasks = get_all_tasks_between(start, end)?;

    let mut logs = BTreeMap::new();
    for task in tasks.values() {
        for log in task.logs.iter() {
            if log.start > end || log.end() < start {
                continue
            }

            logs.insert(log.start, (task.clone(), *log));
        }
    }

    Ok(logs.values().map(|log| log.clone()).collect())
}

pub fn get_total_duration(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) -> Result<chrono::Duration, Error> {
    let repo = get_repo()?;

    let mut total_duration = chrono::Duration::seconds(0);

    for task in repo.list_tasks()?.values() {
        total_duration = total_duration.add(task.duration_between(start, end));
    }

    Ok(total_duration)
}

pub fn format_duration(duration: chrono::Duration) -> String {
    let mut formatted_duration = String::new();

    if duration.num_hours() >= 24 {
        formatted_duration.push_str(format!("{}d ", duration.num_days()).as_str());
    }

    if duration.num_minutes() >= 60 {
        formatted_duration.push_str(format!("{}h ", duration.num_hours() % 24).as_str());
    }

    if duration.num_seconds() >= 60 {
        formatted_duration.push_str(format!("{}m ", duration.num_minutes() % 60).as_str());
    }

    if duration.num_seconds() < 60 {
        formatted_duration.push_str(format!("{}s ", duration.num_seconds() % 60).as_str());
    }

    formatted_duration.trim().to_string()
}

pub fn format_duration_hours(duration: chrono::Duration) -> String {
    let mut formatted_duration = String::new();

    if duration.num_minutes() >= 60 {
        formatted_duration.push_str(format!("{}h ", duration.num_hours()).as_str());
    }

    if duration.num_seconds() >= 60 {
        formatted_duration.push_str(format!("{}m ", duration.num_minutes() % 60).as_str());
    }

    if duration.num_seconds() < 60 {
        formatted_duration.push_str(format!("{}s ", duration.num_seconds() % 60).as_str());
    }

    formatted_duration.trim().to_string()
}

pub fn find_start(tasks: &HashMap<u32, Task>) -> Result<chrono::DateTime<chrono::Utc>, Error> {
    if tasks.len() == 0 {
        return Err(Error::Value(ValueError::new("There are no tasks.")));
    }

    let mut start = tasks
        .values().next().unwrap()
        .logs.first().unwrap()
        .start
    ;
    for task in tasks.values() {
        let log = task.logs.first().unwrap();

        if log.start < start {
            start = log.start;
        }
    }
    Ok(start)
}

pub fn find_end(tasks: &HashMap<u32, Task>) -> Result<chrono::DateTime<chrono::Utc>, Error> {
    if tasks.len() == 0 {
        return Err(Error::Value(ValueError::new("There are no tasks.")));
    }

    let mut end = tasks
        .values().next().unwrap()
        .logs.last().unwrap()
        .end()
    ;
    for task in tasks.values() {
        let log = task.logs.last().unwrap();

        if log.end() < end {
            end = log.end();
        }
    }
    Ok(end)
}

pub fn task_path(task: u32) -> PathBuf {
    let mut path = data_path();
    path.push(task.to_string());
    path
}