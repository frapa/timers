use std::io::prelude::*;
use std::collections::HashMap;
use std::ops::Add;

use crate::errors::{Error, ValueError};

#[derive(Debug)]
pub struct Log {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: Option<chrono::DateTime<chrono::Utc>>,
}

impl Log {
    pub fn duration(&self) -> chrono::Duration {
        if let Some(end) = self.end {
            end.signed_duration_since(self.start)
        } else {
            chrono::Utc::now().signed_duration_since(self.start)
        }
    }

    pub fn duration_between(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>
    ) -> chrono::Duration {
        let task_end = if self.end.is_none() {
            chrono::Utc::now()
        } else {
            self.end.unwrap()
        };

        if start.le(&self.start) && end.ge(&task_end) {
            self.duration()
        } else if start.gt(&self.start) && end.ge(&task_end) {
            let duration = task_end.signed_duration_since(start);
            if duration.num_seconds() >= 0 { duration } else { chrono::Duration::seconds(0) }
        } else if start.le(&self.start) && end.lt(&task_end) {
            let duration = end.signed_duration_since(self.start);
            if duration.num_seconds() >= 0 { duration } else { chrono::Duration::seconds(0) }
        } else {
            end.signed_duration_since(start)
        }
    }
}

#[derive(Debug)]
pub struct Task {
    pub id: u32,
    pub path: std::path::PathBuf,
    pub name: String,
    pub logs: Vec<Log>,
    pub logging: bool,
}

#[derive(Debug)]
pub enum TaskStatus {
    Logging(),
    Stopped(),
}

impl Task {
    pub fn duration(&self) -> chrono::Duration {
        let mut duration = chrono::Duration::seconds(0);
        for log in self.logs.iter() {
            duration = duration + log.duration();
        }

        duration
    }

    pub fn duration_between(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>
    ) -> chrono::Duration {
        let mut total = chrono::Duration::seconds(0);

        for log in self.logs.iter() {
            total = total.add(log.duration_between(start, end));
        }

        total
    }

    pub fn status(&self) -> TaskStatus {
        for log in self.logs.iter() {
            if log.end.is_none() {
                return TaskStatus::Logging()
            }
        }

        TaskStatus::Stopped()
    }

    pub fn status_text(&self) -> &str {
        match self.status() {
            TaskStatus::Logging() => "logging",
            TaskStatus::Stopped() => "stopped",
        }
    }
}

#[derive(Debug)]
pub struct Repo {
    pub path: std::path::PathBuf,
}

impl Repo {
    fn read_task(path: std::path::PathBuf) -> Result<Task, Error> {
        let file = std::fs::File::open(&path)?;
        let mut reader = std::io::BufReader::new(file);

        let mut id_str= String::new();
        reader.read_line(&mut id_str)?;
        let id = id_str.trim().parse::<u32>().expect("Unexpected or corrupt id value in task file");

        let mut name = String::new();
        reader.read_line(&mut name)?;

        let mut logs = Vec::new();
        let mut logging= false;
        loop {
            let mut line = String::new();
            let num_bytes_read = reader.read_line(&mut line)?;
            if num_bytes_read == 0 {
                break
            }

            let split: Vec<&str> = line.split(" ").collect();
            let start = split[0].trim();
            let end = split[1].trim();
            if end.len() == 0 {
                logging = true;
            }

            logs.push(Log {
                start: chrono::DateTime::parse_from_rfc3339(start)
                    .expect("Unexpected or corrupt start date value in task file")
                    .with_timezone(&chrono::Utc),
                end: if logging {
                    None
                } else {
                    Some(chrono::DateTime::parse_from_rfc3339(end)
                        .expect("Unexpected or corrupt end date value in task file")
                        .with_timezone(&chrono::Utc))
                }
            })
        }

        Ok(Task {
            id,
            path,
            name: name.trim().to_string(),
            logs,
            logging,
        })
    }

    fn write_task(task: &Task) -> Result<(), Error> {
        let mut file = std::fs::File::create(&task.path)?;

        write!(file, "{}\n{}\n", task.id, task.name)?;

        for log in task.logs.iter() {
            write!(file, "{} ", log.start.to_rfc3339())?;

            if let Some(end) = log.end {
                write!(file, "{}\n", end.to_rfc3339())?;
            }
        }

        Ok(())
    }

    pub fn list_tasks(&self) -> Result<HashMap<u32, Task>, Error> {
        let paths = std::fs::read_dir(&self.path)?;

        let mut tasks = HashMap::new();
        for path in paths {
            let path = path.unwrap().path();
            let task = Repo::read_task(path)?;
            tasks.insert(task.id, task);
        }

        Ok(tasks)
    }

    pub fn get_task(&self, id: u32) -> Result<Task, Error> {
        let mut path = self.path.clone();
        path.push(id.to_string());

        Ok(Repo::read_task(path)?)
    }

    pub fn create_task(&self, name: &str) -> Result<Task, Error> {
        let id = self.next_id()?;

        let mut path = self.path.clone();
        path.push(id.to_string());

        let task = Task {
            id,
            path,
            name: name.to_string(),
            logs: Vec::new(),
            logging: false,
        };

        Repo::write_task(&task)?;

        Ok(task)
    }

    fn next_id(&self) -> Result<u32, Error> {
        let tasks = self.list_tasks()?;

        let mut max_id = 0u32;
        for id in tasks.keys() {
            if id > &max_id {
                max_id = *id;
            }
        }

        Ok(max_id + 1)
    }

    pub fn log_task(
        &self,
        task: &mut Task,
        time: chrono::DateTime<chrono::Utc>
    ) -> Result<(), Error> {
        task.logging = true;
        task.logs.push(Log {
            start: time,
            end: None,
        });

        Repo::write_task(&task)?;

        Ok(())
    }

    pub fn stop_task(
        &self,
        task: &mut Task,
        time: chrono::DateTime<chrono::Utc>
    ) -> Result<(), Error> {
        task.logging = false;

        match task.logs.last_mut() {
            Some(log) => {
                if log.end.is_some() {
                    return Err(Error::Value(
                        ValueError::new("Task was not started, cannot stop logging.")
                    ))
                }

                log.end = Some(time)
            },
            None => return Err(Error::Value(
                ValueError::new("Task was not started, cannot stop logging.")
            ))
        }

        Repo::write_task(&task)?;

        Ok(())
    }
}


