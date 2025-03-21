use crate::jiffy::{Backup, Status, TimeEntry, TimeOwner};
use crate::toggl::Row;
use anyhow::bail;
use chrono::TimeDelta;
use clap::Parser;
use csv::Writer;
use itertools::Itertools;
use log::{info, warn};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;

pub mod jiffy;
pub mod toggl;

fn format_duration(duration: TimeDelta) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

struct TimeOwnersHierarchy {
    clients: HashMap<String, TimeOwner>,
    projects: HashMap<String, TimeOwner>,
    tasks: HashMap<String, TimeOwner>,
}

impl TimeOwnersHierarchy {
    fn new(time_owners: Vec<TimeOwner>) -> anyhow::Result<Self> {
        let clients: HashMap<_, _> = time_owners
            .iter()
            .filter(|v| v.parent_id.is_none())
            .map(|v| (v.id.clone(), v.clone()))
            .collect();

        let projects: HashMap<_, _> = time_owners
            .iter()
            .filter(|v| {
                if let Some(p) = v.parent_id.as_ref() {
                    clients.contains_key(p.as_str())
                } else {
                    false
                }
            })
            .map(|v| (v.id.clone(), v.clone()))
            .collect();

        let tasks: HashMap<_, _> = time_owners
            .iter()
            .filter(|v| {
                if let Some(p) = v.parent_id.as_ref() {
                    projects.contains_key(p.as_str())
                } else {
                    false
                }
            })
            .map(|v| (v.id.clone(), v.clone()))
            .collect();

        if time_owners.len() != clients.len() + projects.len() + tasks.len() {
            bail!("Not all time owners were classified");
        }

        Ok(Self {
            clients,
            projects,
            tasks,
        })
    }

    fn location(&self, subject: &TimeEntry) -> TimeOwnersLocation {
        let task = self.tasks.get(&subject.owner_id);
        let project = if let Some(task) = task {
            if let Some(parent) = &task.parent_id {
                self.projects.get(parent)
            } else {
                None
            }
        } else {
            self.projects.get(&subject.owner_id)
        };
        let client = if let Some(project) = project {
            if let Some(parent) = &project.parent_id {
                self.clients.get(parent)
            } else {
                None
            }
        } else {
            self.clients.get(&subject.owner_id)
        };
        TimeOwnersLocation {
            client,
            project,
            task,
        }
    }
}

struct TimeOwnersLocation<'a> {
    client: Option<&'a TimeOwner>,
    project: Option<&'a TimeOwner>,
    task: Option<&'a TimeOwner>,
}

#[derive(Debug, Parser)]
pub struct Cli {
    /// Input file to read
    backup: String,
    /// Email for email and user columns
    email: String,
    /// Number of rows to skip
    #[arg(short, long)]
    skip: Option<usize>,
    /// Max number of rows to produc
    #[arg(short, long, default_value = "5000")]
    take: usize,
}

impl Cli {
    pub fn exec(&self) -> anyhow::Result<()> {
        let Backup {
            time_entries,
            time_owners,
            ..
        } = serde_json::from_reader(BufReader::new(File::open(self.backup.as_str())?))?;

        let hierarchy = TimeOwnersHierarchy::new(time_owners)?;

        let rows = time_entries
            .into_iter()
            .sorted_by_key(|v| std::cmp::Reverse(v.start_time))
            .filter_map(|entry| {
                match entry.status {
                    Status::Active => {}
                    Status::Archived => {}
                    Status::Deleted => {
                        info!("Skipping deleted {entry:?}");
                        return None;
                    }
                }

                let TimeOwnersLocation {
                    client,
                    project,
                    task,
                } = hierarchy.location(&entry);

                let Ok(start) = entry.start() else {
                    warn!("Invalid start time");
                    return None;
                };
                let Some(duration) = entry.duration() else {
                    warn!("Entry was never stopped");
                    return None;
                };

                Some(Row {
                    user: self.email.to_string(),
                    email: self.email.to_string(),
                    client: client.map(|c| c.name.clone()),
                    project: project.map(|p| p.name.clone()),
                    description: task.map(|t| t.name.clone()).unwrap_or_default(),
                    start_date: start.format("%Y-%m-%d").to_string(),
                    start_time: start.format("%H:%M:%S").to_string(),
                    duration: format_duration(duration),
                })
            })
            .skip(self.skip.unwrap_or(0))
            .take(self.take);

        let mut wtr = Writer::from_writer(vec![]);
        for row in rows {
            wtr.serialize(row).unwrap();
        }
        println!("{}", String::from_utf8(wtr.into_inner().unwrap()).unwrap());
        Ok(())
    }
}
