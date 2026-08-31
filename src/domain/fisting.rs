use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const DEFENSE_MINUTES: i64 = 30;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FistingInfo {
    pub user: String,
    pub fisting_defense_data: DateTime<Utc>,
}

impl FistingInfo {
    pub fn new(user: String) -> Self {
        FistingInfo {
            user,
            fisting_defense_data: Utc::now(),
        }
    }
}

pub fn is_protected(target: &str, records: &[FistingInfo]) -> bool {
    records.iter().any(|r| {
        r.user == target
            && i64::abs(
                (Utc::now().time() - r.fisting_defense_data.time()).num_minutes(),
            ) <= DEFENSE_MINUTES
    })
}

pub fn update_defense(user: String, records: &mut Vec<FistingInfo>) {
    if let Some(r) = records.iter_mut().find(|r| r.user == user) {
        r.fisting_defense_data = Utc::now();
    } else {
        records.push(FistingInfo::new(user));
    }
}
