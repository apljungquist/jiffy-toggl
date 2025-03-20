use anyhow::bail;
use chrono::{DateTime, MappedLocalTime, TimeDelta, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

fn datetime_utc(millis: i64) -> anyhow::Result<DateTime<Utc>> {
    if millis < 0 {
        bail!("milliseconds must be positive")
    }
    match Utc.timestamp_millis_opt(millis) {
        MappedLocalTime::Single(dt) => Ok(dt),
        MappedLocalTime::Ambiguous(_, _) => bail!("Time is ambiguous"),
        MappedLocalTime::None => bail!("Time does not exist"),
    }
}

fn datetime_local(millis: i64, tz: &str) -> anyhow::Result<DateTime<Tz>> {
    let tz: Tz = tz.parse()?;
    Ok(datetime_utc(millis)?.with_timezone(&tz))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Backup {
    pub time_entries: Vec<TimeEntry>,
    pub time_owners: Vec<TimeOwner>,
    base_work_times: Vec<BaseWorkTime>,
    day_starts: Vec<DayStart>,
    meta: Meta,
    preferences: Preferences,
    purchases: Vec<Purchase>,
    settings: Vec<Setting>,
    time_entry_locations: Vec<TimeEntryLocation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BaseWorkTime {
    duration: i64,
    id: String,
    last_changed: i64,
    status: String,
    weekday: Weekday,
    work_time_group_id: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DayStart {
    day_id: u32,
    last_changed: i64,
    zone_name: String,
    zone_offset: i64,
    start_of_day: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    db_version: u32,
    partial_backup: bool,
    last_used_sync: i64,
    version_code: u32,
    version_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Preferences {
    balance_enabled: bool,
    duration_presentation: String,
    first_day_of_week: String,
    paused_notifiation_enabled: bool,
    sort_order: String,
    start_of_day: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Purchase {
    price: String,
    purchase_time: Option<i64>,
    order_id: Option<String>,
    sku: String,
    status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Setting {
    name: String,
    value: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Active,
    Archived,
    Deleted,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimeEntry {
    pub id: String,
    pub owner_id: String,
    pub(crate) start_time: i64,
    start_time_zone: String,
    stop_time: i64,
    stop_time_zone: String,
    last_changed: i64,
    locked: bool,
    note: Option<String>,
    pub status: Status,
}

impl TimeEntry {
    pub fn start(&self) -> anyhow::Result<DateTime<Tz>> {
        datetime_local(self.start_time, self.start_time_zone.as_str())
    }

    fn stop(&self) -> anyhow::Result<DateTime<Tz>> {
        datetime_local(self.stop_time, self.stop_time_zone.as_str())
    }

    pub fn duration(&self) -> Option<TimeDelta> {
        if -1 == self.stop_time {
            return None;
        }
        let d = TimeDelta::milliseconds(self.stop_time - self.start_time);
        debug_assert_eq!(
            d,
            datetime_utc(self.stop_time).unwrap() - datetime_utc(self.start_time).unwrap()
        );
        debug_assert_eq!(d, self.stop().unwrap() - self.start().unwrap());
        Some(d)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimeEntryLocation {
    id: String,
    entry_id: String,
    accuracy: f32,
    status: String,
    last_changed: i64,
    latitude: f64,
    longitude: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimeOwner {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    color: String,
    last_changed: i64,
    local: bool,
    sort_value: i64,
    status: Status,
    work_time_group: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
