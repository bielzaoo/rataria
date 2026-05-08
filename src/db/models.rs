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

#[derive(Debug, Clone, PartialEq)]
pub enum SubdomainStatus {
    NotVisited,
    InProgress,
    Reviewed,
    Vulnerable,
    FalsePositive,
}

impl SubdomainStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SubdomainStatus::NotVisited => "not-visited",
            SubdomainStatus::InProgress => "in-progress",
            SubdomainStatus::Reviewed => "reviewed",
            SubdomainStatus::Vulnerable => "vulnerable",
            SubdomainStatus::FalsePositive => "false-positive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "in-progress" => SubdomainStatus::InProgress,
            "reviewed" => SubdomainStatus::Reviewed,
            "vulnerable" => SubdomainStatus::Vulnerable,
            "false-positive" => SubdomainStatus::FalsePositive,
            _ => SubdomainStatus::NotVisited,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Subdomain {
    pub id: String,
    pub target_id: String,
    pub subdomain: String,
    pub status: SubdomainStatus,
    pub notes: Option<String>,
    pub status_code: Option<i32>,
    pub title: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct NewSubdomain {
    pub target_id: String,
    pub subdomain: String,
    pub status_code: Option<i32>,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateSubdomain {
    pub status: Option<SubdomainStatus>,
    pub notes: Option<String>,
    pub status_code: Option<i32>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub id: String,
    pub subdomain_id: String,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct NewTag {
    pub subdomain_id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Technology {
    pub id: String,
    pub subdomain_id: String,
    pub name: String,
    pub version: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct NewTechnology {
    pub subdomain_id: String,
    pub name: String,
    pub version: Option<String>,
}
