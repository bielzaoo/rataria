use chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Engagement {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct NewEngagement {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub id: String,
    pub engagement_id: String,
    pub domain: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct NewTarget {
    pub engagement_id: String,
    pub domain: String,
}
