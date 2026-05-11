pub mod generic;
pub mod ratazana;
pub mod report;

use crate::db::Database;
use crate::error::{RatariaError, Result};
pub use report::ImportReport;

/// Detecta o formato e importa automaticamente
/// engagement_name é usado apenas no formato genérico quando não há engagement no arquivo
pub fn auto_import(
    db: &Database,
    content: &str,
    filename: &str,
    engagement_name: Option<&str>,
    target_for_txt: Option<&str>,
) -> Result<ImportReport> {
    // TXT — detecta pela extensão
    if filename.ends_with(".txt") {
        let target = target_for_txt.ok_or_else(|| {
            RatariaError::ImportError(
                "Para importar .txt é necessário informar o target".to_string(),
            )
        })?;
        let file = generic::parse_txt(content, target, engagement_name)?;
        return generic::import(db, &file, engagement_name);
    }

    // JSON — detecta pelo campo rataria_version
    if content.contains("\"rataria_version\"") {
        let file = ratazana::parse(content)?;
        return ratazana::import(db, &file);
    }

    // JSON genérico
    let file = generic::parse_json(content)?;
    generic::import(db, &file, engagement_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn setup() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_auto_import_detecta_ratazana() {
        let db = setup();
        let json = r#"{
            "rataria_version": "1.0",
            "target": "x.com",
            "engagement_name": "Test",
            "timestamp": "",
            "subdomains": [{"subdomain":"api.x.com","status_code":null,"title":null,"technologies":[],"urls":[],"sources":[]}],
            "ips": [],
            "asns": []
        }"#;
        let r = auto_import(&db, json, "scan.json", None, None).unwrap();
        assert_eq!(r.subdomains_added, 1);
    }

    #[test]
    fn test_auto_import_detecta_json_generico() {
        let db = setup();
        let json = r#"{"target":"x.com","subdomains":["api.x.com"]}"#;
        let r = auto_import(&db, json, "subs.json", Some("Test"), None).unwrap();
        assert_eq!(r.subdomains_added, 1);
    }

    #[test]
    fn test_auto_import_detecta_txt() {
        let db = setup();
        let txt = "api.x.com\nadmin.x.com";
        let r = auto_import(&db, txt, "subs.txt", Some("Test"), Some("x.com")).unwrap();
        assert_eq!(r.subdomains_added, 2);
    }

    #[test]
    fn test_auto_import_txt_sem_target_falha() {
        let db = setup();
        let txt = "api.x.com";
        let r = auto_import(&db, txt, "subs.txt", Some("Test"), None);
        assert!(r.is_err());
    }
}
