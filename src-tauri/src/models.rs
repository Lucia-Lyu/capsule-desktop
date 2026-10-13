use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    pub task_date: String,
    pub status: String,
    pub sort_order: i64,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub origin_created_at: Option<String>,
    pub carried_to_date: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderRule {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub interval_minutes: i64,
    pub snooze_minutes: i64,
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderEvent {
    pub id: String,
    pub reminder_id: String,
    pub scheduled_at: String,
    pub triggered_at: Option<String>,
    pub action: String,
    pub completed_at: Option<String>,
    pub snooze_count: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextReminder {
    pub reminder_id: String,
    pub next_trigger_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderSnapshot {
    pub rules: Vec<ReminderRule>,
    pub next: Vec<NextReminder>,
    pub active: Vec<ReminderEvent>,
    pub quiet_start: String,
    pub quiet_end: String,
    pub attention_sound: bool,
    pub attention_repeat: bool,
    pub attention_visible: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderCount {
    pub reminder_id: String,
    pub completed: usize,
    pub total: usize,
    pub completion_rate: u8,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryDay {
    pub date: String,
    pub tasks: Vec<Task>,
    pub tasks_completed: usize,
    pub tasks_total: usize,
    pub task_completion_rate: u8,
    pub reminders: Vec<ReminderCount>,
}
