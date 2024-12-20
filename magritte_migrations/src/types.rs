use magritte::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FlexibleDateTime {
    #[cfg(feature = "with-chrono")]
    Chrono(String),
    #[cfg(feature = "with-time")]
    Time(String),
    Std(String),
}
impl FlexibleDateTime {
    pub fn now() -> FlexibleDateTime {
        #[cfg(feature = "chrono")]
        {
            FlexibleDateTime::Chrono(chrono::Utc::now().format("%Y%m%d%H%M%S").to_string())
        }
        #[cfg(all(feature = "time", not(feature = "chrono")))]
        {
            let format =
                time::macros::format_description!("[year][month][day][hour][minute][second]");
            let utc = time::OffsetDateTime::now_utc()
                .format(&format)
                .unwrap_or_else(|_| "19700101000000".to_string());
            FlexibleDateTime::Time(utc)
        }
        #[cfg(not(any(feature = "chrono", feature = "time")))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now();
            let duration = now.duration_since(UNIX_EPOCH).unwrap();
            let secs = duration.as_secs();

            // Convert seconds to a datetime and format it
            let (year, month, day, hour, minute, second) = Self::seconds_to_ymd_hms(secs);
            FlexibleDateTime::Std(format!(
                "{:04}{:02}{:02}{:02}{:02}{:02}",
                year, month, day, hour, minute, second
            ))
        }
    }

    #[cfg(not(any(feature = "with-chrono", feature = "with-time")))]
    fn seconds_to_ymd_hms(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
        const SECS_PER_MINUTE: u64 = 60;
        const SECS_PER_HOUR: u64 = 3600;
        const SECS_PER_DAY: u64 = 86400;

        let days_since_epoch = secs / SECS_PER_DAY;
        let secs_of_day = secs % SECS_PER_DAY;

        // Compute the actual year
        let mut year = 1970;
        let mut days_left = days_since_epoch as i64;
        while days_left >= Self::days_in_year(year) as i64 {
            days_left -= Self::days_in_year(year) as i64;
            year += 1;
        }

        // Compute the month and day
        let mut month = 1;
        while days_left >= Self::days_in_month(year, month) as i64 {
            days_left -= Self::days_in_month(year, month) as i64;
            month += 1;
        }
        let day = (days_left + 1) as u32;

        let hour = (secs_of_day / SECS_PER_HOUR) as u32;
        let minute = ((secs_of_day % SECS_PER_HOUR) / SECS_PER_MINUTE) as u32;
        let second = (secs_of_day % SECS_PER_MINUTE) as u32;

        (year as u32, month as u32, day, hour, minute, second)
    }
    #[cfg(not(any(feature = "with-chrono", feature = "with-time")))]
    fn is_leap_year(y: i32) -> bool {
        (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
    }
    #[cfg(not(any(feature = "with-chrono", feature = "with-time")))]
    fn days_in_year(y: i32) -> u32 {
        if Self::is_leap_year(y) {
            366
        } else {
            365
        }
    }
    #[cfg(not(any(feature = "with-chrono", feature = "with-time")))]
    fn days_in_month(y: i32, m: i32) -> u32 {
        match m {
            1 => 31,
            2 => {
                if Self::is_leap_year(y) {
                    29
                } else {
                    28
                }
            }
            3 => 31,
            4 => 30,
            5 => 31,
            6 => 30,
            7 => 31,
            8 => 31,
            9 => 30,
            10 => 31,
            11 => 30,
            12 => 31,
            _ => unreachable!(),
        }
    }
}

impl Display for FlexibleDateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "chrono")]
            FlexibleDateTime::Chrono(d) => write!(f, "{}", d),
            #[cfg(all(feature = "time", not(feature = "chrono")))]
            FlexibleDateTime::Time(d) => write!(f, "{}", d),
            #[cfg(not(any(feature = "chrono", feature = "time")))]
            FlexibleDateTime::Std(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum MigrationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Clone, Serialize, Deserialize, Table)]
#[table(name = "_migrations", if_not_exists)]
pub struct MigrationRecord {
    pub id: SurrealId<Self>,
    pub name: String,     // Human readable name
    pub version: String,  // Semantic version or timestamp
    pub checksum: String, // Hash of migration content
    pub status: MigrationStatus,
    #[column(type = "datetime")]
    pub applied_at: Option<FlexibleDateTime>,
    pub execution_time_ms: Option<i64>,
    pub error_message: Option<String>,
}
impl HasId for MigrationRecord {
    fn id(&self) -> SurrealId<Self> {
        self.id.clone()
    }
}

#[derive(Index, strum::EnumIter, Serialize, Deserialize)]
pub enum MigrationRecordIndexes {

}
#[derive(Event, strum::EnumIter, Serialize, Deserialize)]
pub enum MigrationRecordEvents {

}
impl MigrationRecord {
    pub fn new(
        id: impl Into<SurrealId<Self>>,
        name: String,
        version: String,
        checksum: String,
    ) -> Self {
        MigrationRecord {
            id: id.into(),
            name,
            version,
            checksum,
            status: MigrationStatus::Pending,
            applied_at: None,
            execution_time_ms: None,
            error_message: None,
        }
    }

    pub fn set_applied(&mut self, execution_time_ms: i64) {
        self.status = MigrationStatus::Completed;
        self.execution_time_ms = Some(execution_time_ms);
        self.applied_at = Some(FlexibleDateTime::now());
    }
}

#[derive(Debug, Clone)]
pub struct MigrationContext {
    pub db: SurrealDB,
    pub namespace: String,
    pub database: String,
}
