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
pub struct UpdateTarget {
    pub domain: String,
}

#[derive(Debug, Clone)]
pub struct UpdateIp {
    pub ip: String,
}

#[derive(Debug, Clone)]
pub struct UpdateAsn {
    pub asn: String,
    pub org: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateUrl {
    pub url: String,
    pub url_type: UrlType,
}

#[derive(Debug, Clone)]
pub struct UpdateTechnology {
    pub name: String,
    pub version: Option<String>,
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
    pub subdomain: Option<String>, // ← adiciona
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub id: String,
    pub subdomain_id: String,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum UrlType {
    Parameter,
    JavaScript,
    Endpoint,
    Other,
}

impl UrlType {
    pub fn as_str(&self) -> &str {
        match self {
            UrlType::Parameter => "parameter",
            UrlType::JavaScript => "javascript",
            UrlType::Endpoint => "endpoint",
            UrlType::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "parameter" => UrlType::Parameter,
            "javascript" => UrlType::JavaScript,
            "endpoint" => UrlType::Endpoint,
            _ => UrlType::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    pub id: String,
    pub subdomain_id: String,
    pub url: String,
    pub url_type: UrlType,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct NewUrl {
    pub subdomain_id: String,
    pub url: String,
    pub url_type: UrlType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ip {
    pub id: String,
    pub target_id: String,
    pub ip: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct NewIp {
    pub target_id: String,
    pub ip: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Asn {
    pub id: String,
    pub target_id: String,
    pub asn: String,
    pub org: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct NewAsn {
    pub target_id: String,
    pub asn: String,
    pub org: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Screenshot {
    pub id: String,
    pub subdomain_id: String,
    pub file_path: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NewScreenshot {
    pub subdomain_id: String,
    pub file_path: String,
}
