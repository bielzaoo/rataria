use crate::db::{models::*, queries, Database};
use crate::error::{RatariaError, Result};
use crate::import::report::ImportReport;
use serde::{Deserialize, Serialize};

// ─── Estruturas do formato genérico ──────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenericFile {
    pub target: String,
    pub engagement: Option<String>,
    #[serde(default)]
    pub subdomains: Vec<GenericSubdomain>,
    #[serde(default)]
    pub ips: Vec<String>,
    #[serde(default)]
    pub asns: Vec<GenericAsn>,
}

/// Subdomains aceita tanto string simples quanto objeto completo
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum GenericSubdomain {
    Simple(String),
    Full(GenericSubdomainFull),
}

impl GenericSubdomain {
    pub fn subdomain_str(&self) -> &str {
        match self {
            GenericSubdomain::Simple(s) => s.as_str(),
            GenericSubdomain::Full(f) => f.subdomain.as_str(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenericSubdomainFull {
    pub subdomain: String,
    pub status_code: Option<i32>,
    pub title: Option<String>,
    #[serde(default)]
    pub technologies: Vec<GenericTechnology>,
    #[serde(default)]
    pub urls: Vec<GenericUrl>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenericTechnology {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenericUrl {
    pub url: String,
    pub url_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenericAsn {
    pub asn: String,
    pub org: Option<String>,
}

// ─── Parsers ──────────────────────────────────────────────────────────────────

/// Parseia JSON genérico
pub fn parse_json(json: &str) -> Result<GenericFile> {
    let file: GenericFile = serde_json::from_str(json)
        .map_err(|e| RatariaError::ImportError(format!("JSON inválido: {}", e)))?;

    if file.target.trim().is_empty() {
        return Err(RatariaError::ImportError(
            "Campo target ausente".to_string(),
        ));
    }
    if file.subdomains.is_empty() {
        return Err(RatariaError::ImportError(
            "Nenhum subdomain encontrado".to_string(),
        ));
    }

    Ok(file)
}

/// Parseia arquivo TXT — uma linha por subdomain
pub fn parse_txt(content: &str, target: &str, engagement: Option<&str>) -> Result<GenericFile> {
    if target.trim().is_empty() {
        return Err(RatariaError::ImportError(
            "Target é obrigatório para import TXT".to_string(),
        ));
    }

    let subdomains: Vec<GenericSubdomain> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| GenericSubdomain::Simple(l.to_string()))
        .collect();

    if subdomains.is_empty() {
        return Err(RatariaError::ImportError(
            "Nenhum subdomain encontrado no arquivo".to_string(),
        ));
    }

    Ok(GenericFile {
        target: target.to_string(),
        engagement: engagement.map(|s| s.to_string()),
        subdomains,
        ips: Vec::new(),
        asns: Vec::new(),
    })
}

/// Importa um GenericFile para o banco
/// Se engagement_name for None, usa o campo do arquivo ou "Imported"
pub fn import(
    db: &Database,
    file: &GenericFile,
    engagement_name: Option<&str>,
) -> Result<ImportReport> {
    let mut report = ImportReport::default();

    let eng_name = engagement_name
        .or(file.engagement.as_deref())
        .unwrap_or("Imported");

    let engagement = match queries::list_engagements(db)?
        .into_iter()
        .find(|e| e.name == eng_name)
    {
        Some(e) => e,
        None => queries::create_engagement(
            db,
            NewEngagement {
                name: eng_name.to_string(),
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
        let (status_code, title, technologies, urls) = match sub {
            GenericSubdomain::Simple(_) => (None, None, vec![], vec![]),
            GenericSubdomain::Full(f) => (
                f.status_code,
                f.title.clone(),
                f.technologies.clone(),
                f.urls.clone(),
            ),
        };

        let subdomain = match queries::create_subdomain(
            db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: sub.subdomain_str().to_string(),
                status_code,
                title,
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
                    .find(|s| s.subdomain == sub.subdomain_str())
                {
                    Some(s) => s,
                    None => continue,
                }
            }
        };

        for tech in &technologies {
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

        for url in &urls {
            let url_type = match url.url_type.as_deref() {
                Some("parameter") => UrlType::Parameter,
                Some("javascript") => UrlType::JavaScript,
                Some("endpoint") => UrlType::Endpoint,
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

    fn setup() -> Database {
        Database::open_in_memory().unwrap()
    }

    // ── parse_json ────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_json_minimo() {
        let json = r#"{
            "target": "empresa.com",
            "subdomains": ["api.empresa.com", "admin.empresa.com"]
        }"#;
        let f = parse_json(json).unwrap();
        assert_eq!(f.target, "empresa.com");
        assert_eq!(f.subdomains.len(), 2);
    }

    #[test]
    fn test_parse_json_completo() {
        let json = r#"{
            "target": "empresa.com",
            "engagement": "Bug Bounty",
            "subdomains": [
                {
                    "subdomain": "api.empresa.com",
                    "status_code": 200,
                    "title": "API",
                    "technologies": [{"name": "Nginx", "version": "1.24"}],
                    "urls": [{"url": "https://api.empresa.com/v1", "url_type": "endpoint"}]
                }
            ],
            "ips": ["1.1.1.1"],
            "asns": [{"asn": "AS123", "org": "XPTO"}]
        }"#;
        let f = parse_json(json).unwrap();
        assert_eq!(f.ips.len(), 1);
        assert_eq!(f.asns.len(), 1);
    }

    #[test]
    fn test_parse_json_target_vazio_falha() {
        let json = r#"{"target": "", "subdomains": ["api.x.com"]}"#;
        assert!(parse_json(json).is_err());
    }

    #[test]
    fn test_parse_json_sem_subdomains_falha() {
        let json = r#"{"target": "x.com", "subdomains": []}"#;
        assert!(parse_json(json).is_err());
    }

    #[test]
    fn test_parse_json_subdomains_mistos() {
        // Aceita strings simples e objetos no mesmo array
        let json = r#"{
            "target": "empresa.com",
            "subdomains": [
                "api.empresa.com",
                {"subdomain": "admin.empresa.com", "status_code": 403, "title": null, "technologies": [], "urls": []}
            ]
        }"#;
        let f = parse_json(json).unwrap();
        assert_eq!(f.subdomains.len(), 2);
        assert_eq!(f.subdomains[0].subdomain_str(), "api.empresa.com");
        assert_eq!(f.subdomains[1].subdomain_str(), "admin.empresa.com");
    }

    // ── parse_txt ─────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_txt_basico() {
        let txt = "api.empresa.com\nadmin.empresa.com\ndev.empresa.com";
        let f = parse_txt(txt, "empresa.com", None).unwrap();
        assert_eq!(f.subdomains.len(), 3);
        assert_eq!(f.target, "empresa.com");
    }

    #[test]
    fn test_parse_txt_ignora_linhas_vazias() {
        let txt = "api.empresa.com\n\n\nadmin.empresa.com\n";
        let f = parse_txt(txt, "empresa.com", None).unwrap();
        assert_eq!(f.subdomains.len(), 2);
    }

    #[test]
    fn test_parse_txt_ignora_comentarios() {
        let txt = "# gerado pelo subfinder\napi.empresa.com\n# outro comentário\nadmin.empresa.com";
        let f = parse_txt(txt, "empresa.com", None).unwrap();
        assert_eq!(f.subdomains.len(), 2);
    }

    #[test]
    fn test_parse_txt_target_vazio_falha() {
        let txt = "api.empresa.com";
        assert!(parse_txt(txt, "", None).is_err());
    }

    #[test]
    fn test_parse_txt_vazio_falha() {
        assert!(parse_txt("", "empresa.com", None).is_err());
    }

    #[test]
    fn test_parse_txt_com_engagement() {
        let txt = "api.empresa.com";
        let f = parse_txt(txt, "empresa.com", Some("Meu Engagement")).unwrap();
        assert_eq!(f.engagement, Some("Meu Engagement".to_string()));
    }

    // ── import ────────────────────────────────────────────────────────────────

    #[test]
    fn test_import_json_minimo() {
        let db = setup();
        let json = r#"{"target":"x.com","subdomains":["api.x.com","admin.x.com"]}"#;
        let f = parse_json(json).unwrap();
        let r = import(&db, &f, Some("Test")).unwrap();
        assert_eq!(r.subdomains_added, 2);
    }

    #[test]
    fn test_import_txt() {
        let db = setup();
        let txt = "api.x.com\nadmin.x.com\ndev.x.com";
        let f = parse_txt(txt, "x.com", None).unwrap();
        let r = import(&db, &f, Some("Test")).unwrap();
        assert_eq!(r.subdomains_added, 3);
    }

    #[test]
    fn test_import_deduplicacao() {
        let db = setup();
        let json = r#"{"target":"x.com","subdomains":["api.x.com"]}"#;
        let f = parse_json(json).unwrap();
        import(&db, &f, Some("Test")).unwrap();
        let r2 = import(&db, &f, Some("Test")).unwrap();
        assert_eq!(r2.subdomains_skipped, 1);
        assert_eq!(r2.subdomains_added, 0);
    }

    #[test]
    fn test_import_usa_engagement_do_arquivo() {
        let db = setup();
        let json = r#"{"target":"x.com","engagement":"Do Arquivo","subdomains":["api.x.com"]}"#;
        let f = parse_json(json).unwrap();
        import(&db, &f, None).unwrap();
        let engs = queries::list_engagements(&db).unwrap();
        assert_eq!(engs[0].name, "Do Arquivo");
    }

    #[test]
    fn test_import_engagement_parametro_tem_prioridade() {
        let db = setup();
        let json = r#"{"target":"x.com","engagement":"Do Arquivo","subdomains":["api.x.com"]}"#;
        let f = parse_json(json).unwrap();
        import(&db, &f, Some("Prioritário")).unwrap();
        let engs = queries::list_engagements(&db).unwrap();
        assert_eq!(engs[0].name, "Prioritário");
    }

    #[test]
    fn test_import_completo_com_techs_e_urls() {
        let db = setup();
        let json = r#"{
            "target": "x.com",
            "subdomains": [{
                "subdomain": "api.x.com",
                "status_code": 200,
                "title": "API",
                "technologies": [{"name": "Rails", "version": "7.0"}],
                "urls": [{"url": "https://api.x.com/v1", "url_type": "endpoint"}]
            }]
        }"#;
        let f = parse_json(json).unwrap();
        let r = import(&db, &f, Some("Test")).unwrap();
        assert_eq!(r.subdomains_added, 1);
        assert_eq!(r.technologies_added, 1);
        assert_eq!(r.urls_added, 1);
    }
}
