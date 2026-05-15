use crate::db::{queries, Database};
use crate::error::Result;
use chrono::Utc;

/// Gera o relatório Markdown completo de um engagement
pub fn export_engagement_markdown(db: &Database, engagement_id: &str) -> Result<String> {
    let engagement = queries::get_engagement(db, engagement_id)?.ok_or_else(|| {
        crate::error::RatariaError::NotFound("Engagement não encontrado".to_string())
    })?;

    let mut md = String::new();

    md.push_str(&format!("# Engagement: {}\n\n", engagement.name));

    if let Some(desc) = &engagement.description {
        md.push_str(&format!("**Descrição:** {}\n\n", desc));
    }

    md.push_str(&format!(
        "**Gerado em:** {}\n\n",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));
    md.push_str("---\n\n");

    let targets = queries::list_targets(db, &engagement.id)?;

    if targets.is_empty() {
        md.push_str("*Nenhum target cadastrado.*\n");
        return Ok(md);
    }

    for target in &targets {
        md.push_str(&format!("## Target: {}\n\n", target.domain));

        let ips = queries::list_ips(db, &target.id)?;
        if !ips.is_empty() {
            md.push_str("### IPs\n\n");
            for ip in &ips {
                md.push_str(&format!("- {}\n", ip.ip));
            }
            md.push('\n');
        }

        let asns = queries::list_asns(db, &target.id)?;
        if !asns.is_empty() {
            md.push_str("### ASNs\n\n");
            for asn in &asns {
                match &asn.org {
                    Some(org) => md.push_str(&format!("- {} — {}\n", asn.asn, org)),
                    None => md.push_str(&format!("- {}\n", asn.asn)),
                }
            }
            md.push('\n');
        }

        let subdomains = queries::list_subdomains(db, &target.id)?;
        if !subdomains.is_empty() {
            md.push_str("### Subdomains\n\n");

            for sub in &subdomains {
                md.push_str(&format!("#### {}\n\n", sub.subdomain));
                md.push_str(&format!("- **Status:** {}\n", sub.status.as_str()));

                if let Some(code) = sub.status_code {
                    let title = sub.title.as_deref().unwrap_or("");
                    md.push_str(&format!("- **HTTP:** {} {}\n", code, title));
                }

                if let Some(notes) = &sub.notes {
                    md.push_str(&format!("- **Notas:** {}\n", notes));
                }

                let techs = queries::list_technologies(db, &sub.id)?;
                if !techs.is_empty() {
                    md.push('\n');
                    md.push_str("**Technologies:**\n\n");
                    for tech in &techs {
                        match &tech.version {
                            Some(v) => md.push_str(&format!("- {} v{}\n", tech.name, v)),
                            None => md.push_str(&format!("- {}\n", tech.name)),
                        }
                    }
                }

                let urls = queries::list_urls(db, &sub.id)?;
                if !urls.is_empty() {
                    md.push('\n');
                    md.push_str("**URLs:**\n\n");
                    for url in &urls {
                        md.push_str(&format!("- {}: {}\n", url.url_type.as_str(), url.url));
                    }
                }

                md.push('\n');
            }
        }

        md.push_str("---\n\n");
    }

    Ok(md)
}

/// Salva o relatório em um arquivo
pub fn save_report(content: &str, path: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}

/// Gera o caminho padrão para o relatório
pub fn default_report_path(engagement_name: &str) -> String {
    let safe_name = engagement_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase();

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    format!(
        "{}/rataria_{}_{}.md",
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .display(),
        safe_name,
        timestamp,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    use crate::db::models::{
        Engagement, NewAsn, NewEngagement, NewIp, NewSubdomain, NewTarget, NewTechnology, NewUrl,
        SubdomainStatus, UpdateSubdomain, UrlType,
    };

    fn setup() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn create_full_engagement(db: &Database) -> Engagement {
        let eng = queries::create_engagement(
            db,
            NewEngagement {
                name: "Bug Bounty Q1".to_string(),
                description: Some("Programa de bug bounty".to_string()),
            },
        )
        .unwrap();

        let target = queries::create_target(
            db,
            NewTarget {
                engagement_id: eng.id.clone(),
                domain: "empresa.com".to_string(),
            },
        )
        .unwrap();

        queries::create_ip(
            db,
            NewIp {
                target_id: target.id.clone(),
                ip: "192.168.1.1".to_string(),
            },
        )
        .unwrap();

        queries::create_asn(
            db,
            NewAsn {
                target_id: target.id.clone(),
                asn: "AS12345".to_string(),
                org: Some("Empresa XPTO".to_string()),
            },
        )
        .unwrap();

        let sub = queries::create_subdomain(
            db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "api.empresa.com".to_string(),
                status_code: Some(200),
                title: Some("API Principal".to_string()),
            },
        )
        .unwrap();

        queries::update_subdomain(
            db,
            &sub.id,
            UpdateSubdomain {
                status: Some(SubdomainStatus::Vulnerable),
                notes: Some("SQLi encontrado".to_string()),
                status_code: None,
                title: None,
                subdomain: None,
            },
        )
        .unwrap();

        queries::create_technology(
            db,
            NewTechnology {
                subdomain_id: sub.id.clone(),
                name: "Nginx".to_string(),
                version: Some("1.24".to_string()),
            },
        )
        .unwrap();

        queries::create_url(
            db,
            NewUrl {
                subdomain_id: sub.id.clone(),
                url: "https://api.empresa.com/v1/users".to_string(),
                url_type: UrlType::Endpoint,
            },
        )
        .unwrap();

        eng
    }

    #[test]
    fn test_export_contem_nome_engagement() {
        let db = setup();
        let eng = create_full_engagement(&db);
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("Bug Bounty Q1"));
    }

    #[test]
    fn test_export_contem_descricao() {
        let db = setup();
        let eng = create_full_engagement(&db);
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("Programa de bug bounty"));
    }

    #[test]
    fn test_export_contem_target() {
        let db = setup();
        let eng = create_full_engagement(&db);
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("empresa.com"));
    }

    #[test]
    fn test_export_contem_ip() {
        let db = setup();
        let eng = create_full_engagement(&db);
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("192.168.1.1"));
    }

    #[test]
    fn test_export_contem_asn() {
        let db = setup();
        let eng = create_full_engagement(&db);
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("AS12345"));
        assert!(md.contains("Empresa XPTO"));
    }

    #[test]
    fn test_export_contem_subdomain() {
        let db = setup();
        let eng = create_full_engagement(&db);
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("api.empresa.com"));
    }

    #[test]
    fn test_export_contem_status_subdomain() {
        let db = setup();
        let eng = create_full_engagement(&db);
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("vulnerable"));
    }

    #[test]
    fn test_export_contem_notas() {
        let db = setup();
        let eng = create_full_engagement(&db);
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("SQLi encontrado"));
    }

    #[test]
    fn test_export_contem_technology() {
        let db = setup();
        let eng = create_full_engagement(&db);
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("Nginx"));
        assert!(md.contains("1.24"));
    }

    #[test]
    fn test_export_contem_url() {
        let db = setup();
        let eng = create_full_engagement(&db);
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("https://api.empresa.com/v1/users"));
        assert!(md.contains("endpoint"));
    }

    #[test]
    fn test_export_engagement_inexistente_falha() {
        let db = setup();
        let result = export_engagement_markdown(&db, "id-inexistente");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_engagement_sem_targets() {
        let db = setup();
        let eng = queries::create_engagement(
            &db,
            NewEngagement {
                name: "Vazio".to_string(),
                description: None,
            },
        )
        .unwrap();
        let md = export_engagement_markdown(&db, &eng.id).unwrap();
        assert!(md.contains("Nenhum target cadastrado"));
    }

    #[test]
    fn test_default_report_path_contem_nome() {
        let path = default_report_path("Bug Bounty Q1");
        assert!(path.contains("bug_bounty_q1"));
        assert!(path.ends_with(".md"));
    }

    #[test]
    fn test_default_report_path_sanitiza_caracteres() {
        let path = default_report_path("Empresa & Cia!");
        assert!(!path.contains('&'));
        assert!(!path.contains('!'));
    }

    #[test]
    fn test_save_report() {
        let path = "/tmp/rataria_test_export.md";
        save_report("# Test\n\nConteúdo", path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("# Test"));
        std::fs::remove_file(path).ok();
    }
}
