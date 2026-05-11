use crate::db::{models::*, queries, Database};
use crate::error::{RatariaError, Result};
use crate::import::report::ImportReport;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RatazanaFile {
    pub rataria_version: String,
    pub target: String,
    pub engagement_name: String,
    pub timestamp: String,
    pub subdomains: Vec<RatazanaSubdomain>,
    pub ips: Vec<String>,
    pub asns: Vec<RatazanaAsn>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RatazanaSubdomain {
    pub subdomain: String,
    pub status_code: Option<i32>,
    pub title: Option<String>,
    pub technologies: Vec<RatazanaTechnology>,
    pub urls: Vec<RatazanaUrl>,
    pub sources: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RatazanaTechnology {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RatazanaUrl {
    pub url: String,
    pub url_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RatazanaAsn {
    pub asn: String,
    pub org: Option<String>,
}

pub fn parse(json: &str) -> Result<RatazanaFile> {
    let file: RatazanaFile = serde_json::from_str(json)
        .map_err(|e| RatariaError::ImportError(format!("JSON inválido: {}", e)))?;

    if file.rataria_version.is_empty() {
        return Err(RatariaError::ImportError(
            "Campo rataria_version ausente".to_string(),
        ));
    }
    if file.target.trim().is_empty() {
        return Err(RatariaError::ImportError(
            "Campo target ausente".to_string(),
        ));
    }
    if file.engagement_name.trim().is_empty() {
        return Err(RatariaError::ImportError(
            "Campo engagement_name ausente".to_string(),
        ));
    }

    Ok(file)
}

pub fn import(db: &Database, file: &RatazanaFile) -> Result<ImportReport> {
    let mut report = ImportReport::default();

    let engagement = match queries::list_engagements(db)?
        .into_iter()
        .find(|e| e.name == file.engagement_name)
    {
        Some(e) => e,
        None => queries::create_engagement(
            db,
            NewEngagement {
                name: file.engagement_name.clone(),
                description: None,
            },
        )?,
    };

    let target = match queries::list_targets(db, &engagement.id)?
        .into_iter()
        .find(|t| t.domain == file.target)
    {
        Some(t) => t,
        None => queries::create_target(
            db,
            NewTarget {
                engagement_id: engagement.id.clone(),
                domain: file.target.clone(),
            },
        )?,
    };

    for ip in &file.ips {
        match queries::create_ip(
            db,
            NewIp {
                target_id: target.id.clone(),
                ip: ip.clone(),
            },
        ) {
            Ok(_) => report.ips_added += 1,
            Err(_) => report.ips_skipped += 1,
        }
    }

    for asn in &file.asns {
        match queries::create_asn(
            db,
            NewAsn {
                target_id: target.id.clone(),
                asn: asn.asn.clone(),
                org: asn.org.clone(),
            },
        ) {
            Ok(_) => report.asns_added += 1,
            Err(_) => report.asns_skipped += 1,
        }
    }

    for sub in &file.subdomains {
        let subdomain = match queries::create_subdomain(
            db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: sub.subdomain.clone(),
                status_code: sub.status_code,
                title: sub.title.clone(),
            },
        ) {
            Ok(s) => {
                report.subdomains_added += 1;
                s
            }
            Err(_) => {
                report.subdomains_skipped += 1;
                match queries::list_subdomains(db, &target.id)?
                    .into_iter()
                    .find(|s| s.subdomain == sub.subdomain)
                {
                    Some(s) => s,
                    None => continue,
                }
            }
        };

        for tech in &sub.technologies {
            match queries::create_technology(
                db,
                NewTechnology {
                    subdomain_id: subdomain.id.clone(),
                    name: tech.name.clone(),
                    version: tech.version.clone(),
                },
            ) {
                Ok(_) => report.technologies_added += 1,
                Err(_) => report.technologies_skipped += 1,
            }
        }

        for url in &sub.urls {
            let url_type = match url.url_type.as_str() {
                "parameter" => UrlType::Parameter,
                "javascript" => UrlType::JavaScript,
                "endpoint" => UrlType::Endpoint,
                _ => UrlType::Other,
            };
            match queries::create_url(
                db,
                NewUrl {
                    subdomain_id: subdomain.id.clone(),
                    url: url.url.clone(),
                    url_type,
                },
            ) {
                Ok(_) => report.urls_added += 1,
                Err(_) => report.urls_skipped += 1,
            }
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn sample_json() -> &'static str {
        r#"{
            "rataria_version": "1.0",
            "target": "empresa.com",
            "engagement_name": "Bug Bounty Q1",
            "timestamp": "2025-01-01T00:00:00Z",
            "subdomains": [
                {
                    "subdomain": "api.empresa.com",
                    "status_code": 200,
                    "title": "API",
                    "technologies": [{"name": "Nginx", "version": "1.24"}],
                    "urls": [{"url": "https://api.empresa.com/v1", "url_type": "endpoint"}],
                    "sources": ["subfinder"]
                }
            ],
            "ips": ["1.1.1.1"],
            "asns": [{"asn": "AS12345", "org": "XPTO"}]
        }"#
    }

    fn setup() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_parse_valido() {
        let f = parse(sample_json()).unwrap();
        assert_eq!(f.rataria_version, "1.0");
        assert_eq!(f.target, "empresa.com");
    }

    #[test]
    fn test_parse_json_invalido() {
        assert!(parse("não é json").is_err());
    }

    #[test]
    fn test_parse_versao_vazia_falha() {
        let json = r#"{"rataria_version":"","target":"x.com","engagement_name":"X","timestamp":"","subdomains":[],"ips":[],"asns":[]}"#;
        assert!(parse(json).is_err());
    }

    #[test]
    fn test_parse_target_vazio_falha() {
        let json = r#"{"rataria_version":"1.0","target":"","engagement_name":"X","timestamp":"","subdomains":[],"ips":[],"asns":[]}"#;
        assert!(parse(json).is_err());
    }

    #[test]
    fn test_import_cria_engagement_e_target() {
        let db = setup();
        let f = parse(sample_json()).unwrap();
        import(&db, &f).unwrap();
        let engs = queries::list_engagements(&db).unwrap();
        assert_eq!(engs.len(), 1);
        assert_eq!(engs[0].name, "Bug Bounty Q1");
    }

    #[test]
    fn test_import_report_correto() {
        let db = setup();
        let f = parse(sample_json()).unwrap();
        let r = import(&db, &f).unwrap();
        assert_eq!(r.subdomains_added, 1);
        assert_eq!(r.ips_added, 1);
        assert_eq!(r.asns_added, 1);
        assert_eq!(r.technologies_added, 1);
        assert_eq!(r.urls_added, 1);
    }

    #[test]
    fn test_import_deduplicacao() {
        let db = setup();
        let f = parse(sample_json()).unwrap();
        import(&db, &f).unwrap();
        let r2 = import(&db, &f).unwrap();
        assert_eq!(r2.subdomains_skipped, 1);
        assert_eq!(r2.ips_skipped, 1);
        assert_eq!(r2.asns_skipped, 1);
    }

    #[test]
    fn test_import_nao_duplica_engagement() {
        let db = setup();
        let f = parse(sample_json()).unwrap();
        import(&db, &f).unwrap();
        import(&db, &f).unwrap();
        assert_eq!(queries::list_engagements(&db).unwrap().len(), 1);
    }
}
