use crate::db::models::{
    Engagement, NewEngagement, NewSubdomain, NewTag, NewTarget, NewTechnology, Subdomain,
    SubdomainStatus, Tag, Target, Technology, UpdateSubdomain,
};

use crate::db::Database;
use crate::error::{RatariaError, Result};
use chrono::Utc;
use uuid::Uuid;

pub fn create_engagement(db: &Database, new: NewEngagement) -> Result<Engagement> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().naive_utc();

    db.conn.execute(
        "INSERT INTO engagements (id, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            id,
            new.name,
            new.description,
            now.to_string(),
            now.to_string()
        ],
    )?;

    Ok(Engagement {
        id,
        name: new.name,
        description: new.description,
        created_at: now,
        updated_at: now,
    })
}

pub fn list_engagements(db: &Database) -> Result<Vec<Engagement>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, name, description, created_at, updated_at
         FROM engagements ORDER BY created_at DESC",
    )?;

    let engagements = stmt
        .query_map([], |row| {
            let created_str: String = row.get(3)?;
            let updated_str: String = row.get(4)?;
            Ok(Engagement {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: chrono::NaiveDateTime::parse_from_str(
                    &created_str,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap_or_default(),
                updated_at: chrono::NaiveDateTime::parse_from_str(
                    &updated_str,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap_or_default(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(engagements)
}

pub fn get_engagement(db: &Database, id: &str) -> Result<Option<Engagement>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, name, description, created_at, updated_at
         FROM engagements WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map([id], |row| {
        let created_str: String = row.get(3)?;
        let updated_str: String = row.get(4)?;
        Ok(Engagement {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: chrono::NaiveDateTime::parse_from_str(&created_str, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap_or_default(),
            updated_at: chrono::NaiveDateTime::parse_from_str(&updated_str, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap_or_default(),
        })
    })?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn update_engagement(
    db: &Database,
    id: &str,
    name: &str,
    description: Option<&str>,
) -> Result<Engagement> {
    let now = Utc::now().naive_utc();

    let rows_affected = db.conn.execute(
        "UPDATE engagements SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![name, description, now.to_string(), id],
    )?;

    if rows_affected == 0 {
        return Err(RatariaError::NotFound(
            "Engagement não encontrado".to_string(),
        ));
    }

    get_engagement(db, id)?
        .ok_or_else(|| RatariaError::NotFound("Engagement não encontrado".to_string()))
}

pub fn delete_engagement(db: &Database, id: &str) -> Result<()> {
    let rows_affected = db.conn.execute(
        "DELETE FROM engagements WHERE id = ?1",
        rusqlite::params![id],
    )?;

    if rows_affected == 0 {
        return Err(RatariaError::NotFound(
            "Engagement não encontrado".to_string(),
        ));
    }

    Ok(())
}

// ─── Targets ─────────────────────────────────────────────────────────────────

// ─── Targets ─────────────────────────────────────────────────────────────────

pub fn create_target(db: &Database, new: NewTarget) -> Result<Target> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().naive_utc();

    db.conn.execute(
        "INSERT INTO targets (id, engagement_id, domain, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, new.engagement_id, new.domain, now.to_string()],
    )?;

    Ok(Target {
        id,
        engagement_id: new.engagement_id,
        domain: new.domain,
        created_at: now,
    })
}

pub fn list_targets(db: &Database, engagement_id: &str) -> Result<Vec<Target>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, engagement_id, domain, created_at
         FROM targets WHERE engagement_id = ?1 ORDER BY created_at ASC",
    )?;

    let targets = stmt
        .query_map([engagement_id], |row| {
            let created_str: String = row.get(3)?;
            Ok(Target {
                id: row.get(0)?,
                engagement_id: row.get(1)?,
                domain: row.get(2)?,
                created_at: chrono::NaiveDateTime::parse_from_str(
                    &created_str,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap_or_default(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(targets)
}

pub fn get_target(db: &Database, id: &str) -> Result<Option<Target>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, engagement_id, domain, created_at
         FROM targets WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map([id], |row| {
        let created_str: String = row.get(3)?;
        Ok(Target {
            id: row.get(0)?,
            engagement_id: row.get(1)?,
            domain: row.get(2)?,
            created_at: chrono::NaiveDateTime::parse_from_str(&created_str, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap_or_default(),
        })
    })?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn delete_target(db: &Database, id: &str) -> Result<()> {
    let rows_affected = db
        .conn
        .execute("DELETE FROM targets WHERE id = ?1", rusqlite::params![id])?;

    if rows_affected == 0 {
        return Err(RatariaError::NotFound("Target não encontrado".to_string()));
    }

    Ok(())
}

// ─── Subdomains ───────────────────────────────────────────────────────────────

// ─── Subdomains ───────────────────────────────────────────────────────────────

pub fn create_subdomain(db: &Database, new: NewSubdomain) -> Result<Subdomain> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().naive_utc();
    let status = SubdomainStatus::NotVisited;

    db.conn.execute(
        "INSERT INTO subdomains (id, target_id, subdomain, status, notes, status_code, title, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            id,
            new.target_id,
            new.subdomain,
            status.as_str(),
            new.status_code,
            new.title,
            now.to_string(),
            now.to_string(),
        ],
    )?;

    Ok(Subdomain {
        id,
        target_id: new.target_id,
        subdomain: new.subdomain,
        status,
        notes: None,
        status_code: new.status_code,
        title: new.title,
        created_at: now,
        updated_at: now,
    })
}

pub fn list_subdomains(db: &Database, target_id: &str) -> Result<Vec<Subdomain>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, target_id, subdomain, status, notes, status_code, title, created_at, updated_at
         FROM subdomains WHERE target_id = ?1 ORDER BY subdomain ASC",
    )?;

    let items = stmt
        .query_map([target_id], |row| {
            let status_str: String = row.get(3)?;
            let created_str: String = row.get(7)?;
            let updated_str: String = row.get(8)?;
            Ok(Subdomain {
                id: row.get(0)?,
                target_id: row.get(1)?,
                subdomain: row.get(2)?,
                status: SubdomainStatus::from_str(&status_str),
                notes: row.get(4)?,
                status_code: row.get(5)?,
                title: row.get(6)?,
                created_at: chrono::NaiveDateTime::parse_from_str(
                    &created_str,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap_or_default(),
                updated_at: chrono::NaiveDateTime::parse_from_str(
                    &updated_str,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap_or_default(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(items)
}

pub fn get_subdomain(db: &Database, id: &str) -> Result<Option<Subdomain>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, target_id, subdomain, status, notes, status_code, title, created_at, updated_at
         FROM subdomains WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map([id], |row| {
        let status_str: String = row.get(3)?;
        let created_str: String = row.get(7)?;
        let updated_str: String = row.get(8)?;
        Ok(Subdomain {
            id: row.get(0)?,
            target_id: row.get(1)?,
            subdomain: row.get(2)?,
            status: SubdomainStatus::from_str(&status_str),
            notes: row.get(4)?,
            status_code: row.get(5)?,
            title: row.get(6)?,
            created_at: chrono::NaiveDateTime::parse_from_str(&created_str, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap_or_default(),
            updated_at: chrono::NaiveDateTime::parse_from_str(&updated_str, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap_or_default(),
        })
    })?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn update_subdomain(db: &Database, id: &str, update: UpdateSubdomain) -> Result<Subdomain> {
    let now = Utc::now().naive_utc();

    // Busca o estado atual para aplicar apenas os campos fornecidos
    let current = get_subdomain(db, id)?
        .ok_or_else(|| RatariaError::NotFound("Subdomain não encontrado".to_string()))?;

    let new_status = update.status.unwrap_or(current.status);
    let new_notes = update.notes.or(current.notes);
    let new_status_code = update.status_code.or(current.status_code);
    let new_title = update.title.or(current.title);

    db.conn.execute(
        "UPDATE subdomains SET status = ?1, notes = ?2, status_code = ?3, title = ?4, updated_at = ?5
         WHERE id = ?6",
        rusqlite::params![
            new_status.as_str(),
            new_notes,
            new_status_code,
            new_title,
            now.to_string(),
            id,
        ],
    )?;

    get_subdomain(db, id)?
        .ok_or_else(|| RatariaError::NotFound("Subdomain não encontrado".to_string()))
}

pub fn delete_subdomain(db: &Database, id: &str) -> Result<()> {
    let rows_affected = db.conn.execute(
        "DELETE FROM subdomains WHERE id = ?1",
        rusqlite::params![id],
    )?;

    if rows_affected == 0 {
        return Err(RatariaError::NotFound(
            "Subdomain não encontrado".to_string(),
        ));
    }

    Ok(())
}

pub fn list_subdomains_by_status(
    db: &Database,
    target_id: &str,
    status: SubdomainStatus,
) -> Result<Vec<Subdomain>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, target_id, subdomain, status, notes, status_code, title, created_at, updated_at
         FROM subdomains WHERE target_id = ?1 AND status = ?2 ORDER BY subdomain ASC",
    )?;

    let items = stmt
        .query_map(rusqlite::params![target_id, status.as_str()], |row| {
            let status_str: String = row.get(3)?;
            let created_str: String = row.get(7)?;
            let updated_str: String = row.get(8)?;
            Ok(Subdomain {
                id: row.get(0)?,
                target_id: row.get(1)?,
                subdomain: row.get(2)?,
                status: SubdomainStatus::from_str(&status_str),
                notes: row.get(4)?,
                status_code: row.get(5)?,
                title: row.get(6)?,
                created_at: chrono::NaiveDateTime::parse_from_str(
                    &created_str,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap_or_default(),
                updated_at: chrono::NaiveDateTime::parse_from_str(
                    &updated_str,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap_or_default(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(items)
}

// ─── Tags ─────────────────────────────────────────────────────────────────────

// ─── Tags ─────────────────────────────────────────────────────────────────────

pub fn create_tag(db: &Database, new: NewTag) -> Result<Tag> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().naive_utc();

    db.conn.execute(
        "INSERT INTO tags (id, subdomain_id, name, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, new.subdomain_id, new.name, now.to_string()],
    )?;

    Ok(Tag {
        id,
        subdomain_id: new.subdomain_id,
        name: new.name,
        created_at: now,
    })
}

pub fn list_tags(db: &Database, subdomain_id: &str) -> Result<Vec<Tag>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, subdomain_id, name, created_at
         FROM tags WHERE subdomain_id = ?1 ORDER BY name ASC",
    )?;

    let items = stmt
        .query_map([subdomain_id], |row| {
            let created_str: String = row.get(3)?;
            Ok(Tag {
                id: row.get(0)?,
                subdomain_id: row.get(1)?,
                name: row.get(2)?,
                created_at: chrono::NaiveDateTime::parse_from_str(
                    &created_str,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap_or_default(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(items)
}

pub fn delete_tag(db: &Database, id: &str) -> Result<()> {
    let rows_affected = db
        .conn
        .execute("DELETE FROM tags WHERE id = ?1", rusqlite::params![id])?;

    if rows_affected == 0 {
        return Err(RatariaError::NotFound("Tag não encontrada".to_string()));
    }

    Ok(())
}

pub fn delete_tag_by_name(db: &Database, subdomain_id: &str, name: &str) -> Result<()> {
    let rows_affected = db.conn.execute(
        "DELETE FROM tags WHERE subdomain_id = ?1 AND name = ?2",
        rusqlite::params![subdomain_id, name],
    )?;

    if rows_affected == 0 {
        return Err(RatariaError::NotFound("Tag não encontrada".to_string()));
    }

    Ok(())
}

// ─── Technologies ─────────────────────────────────────────────────────────────

// ─── Technologies ─────────────────────────────────────────────────────────────

pub fn create_technology(db: &Database, new: NewTechnology) -> Result<Technology> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().naive_utc();

    db.conn.execute(
        "INSERT INTO technologies (id, subdomain_id, name, version, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, new.subdomain_id, new.name, new.version, now.to_string()],
    )?;

    Ok(Technology {
        id,
        subdomain_id: new.subdomain_id,
        name: new.name,
        version: new.version,
        created_at: now,
    })
}

pub fn list_technologies(db: &Database, subdomain_id: &str) -> Result<Vec<Technology>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, subdomain_id, name, version, created_at
         FROM technologies WHERE subdomain_id = ?1 ORDER BY name ASC",
    )?;

    let items = stmt
        .query_map([subdomain_id], |row| {
            let created_str: String = row.get(4)?;
            Ok(Technology {
                id: row.get(0)?,
                subdomain_id: row.get(1)?,
                name: row.get(2)?,
                version: row.get(3)?,
                created_at: chrono::NaiveDateTime::parse_from_str(
                    &created_str,
                    "%Y-%m-%d %H:%M:%S%.f",
                )
                .unwrap_or_default(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(items)
}

pub fn delete_technology(db: &Database, id: &str) -> Result<()> {
    let rows_affected = db.conn.execute(
        "DELETE FROM technologies WHERE id = ?1",
        rusqlite::params![id],
    )?;

    if rows_affected == 0 {
        return Err(RatariaError::NotFound(
            "Technology não encontrada".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Database {
        Database::open_in_memory().expect("falhou ao abrir banco em memória")
    }

    #[test]
    fn test_create_engagement() {
        let db = setup();
        let new = NewEngagement {
            name: "Empresa XPTO".to_string(),
            description: Some("Bug bounty Q1".to_string()),
        };

        let engagement = create_engagement(&db, new).unwrap();

        assert!(!engagement.id.is_empty());
        assert_eq!(engagement.name, "Empresa XPTO");
        assert_eq!(engagement.description, Some("Bug bounty Q1".to_string()));
    }

    #[test]
    fn test_create_engagement_sem_descricao() {
        let db = setup();
        let new = NewEngagement {
            name: "Alvo Simples".to_string(),
            description: None,
        };

        let engagement = create_engagement(&db, new).unwrap();
        assert_eq!(engagement.description, None);
    }

    #[test]
    fn test_list_engagements_vazio() {
        let db = setup();
        let lista = list_engagements(&db).unwrap();
        assert!(lista.is_empty());
    }

    #[test]
    fn test_list_engagements() {
        let db = setup();

        create_engagement(
            &db,
            NewEngagement {
                name: "Empresa A".to_string(),
                description: None,
            },
        )
        .unwrap();

        create_engagement(
            &db,
            NewEngagement {
                name: "Empresa B".to_string(),
                description: None,
            },
        )
        .unwrap();

        let lista = list_engagements(&db).unwrap();
        assert_eq!(lista.len(), 2);
    }

    #[test]
    fn test_get_engagement_existente() {
        let db = setup();
        let criado = create_engagement(
            &db,
            NewEngagement {
                name: "Alvo X".to_string(),
                description: None,
            },
        )
        .unwrap();

        let encontrado = get_engagement(&db, &criado.id).unwrap();
        assert!(encontrado.is_some());
        assert_eq!(encontrado.unwrap().name, "Alvo X");
    }

    #[test]
    fn test_get_engagement_inexistente() {
        let db = setup();
        let resultado = get_engagement(&db, "id-que-nao-existe").unwrap();
        assert!(resultado.is_none());
    }

    #[test]
    fn test_update_engagement() {
        let db = setup();
        let criado = create_engagement(
            &db,
            NewEngagement {
                name: "Nome Antigo".to_string(),
                description: None,
            },
        )
        .unwrap();

        let atualizado =
            update_engagement(&db, &criado.id, "Nome Novo", Some("Descrição nova")).unwrap();

        assert_eq!(atualizado.name, "Nome Novo");
        assert_eq!(atualizado.description, Some("Descrição nova".to_string()));
    }

    #[test]
    fn test_delete_engagement() {
        let db = setup();
        let criado = create_engagement(
            &db,
            NewEngagement {
                name: "Para Deletar".to_string(),
                description: None,
            },
        )
        .unwrap();

        delete_engagement(&db, &criado.id).unwrap();

        let resultado = get_engagement(&db, &criado.id).unwrap();
        assert!(resultado.is_none());
    }

    #[test]
    fn test_nome_duplicado_falha() {
        let db = setup();

        create_engagement(
            &db,
            NewEngagement {
                name: "Nome Único".to_string(),
                description: None,
            },
        )
        .unwrap();

        let resultado = create_engagement(
            &db,
            NewEngagement {
                name: "Nome Único".to_string(),
                description: None,
            },
        );

        assert!(resultado.is_err());
    }

    // ─── Testes de Target ─────────────────────────────────────────────────────

    fn create_test_engagement(db: &Database) -> Engagement {
        create_engagement(
            db,
            NewEngagement {
                name: "Engagement Teste".to_string(),
                description: None,
            },
        )
        .unwrap()
    }

    #[test]
    fn test_create_target() {
        let db = setup();
        let eng = create_test_engagement(&db);

        let target = create_target(
            &db,
            NewTarget {
                engagement_id: eng.id.clone(),
                domain: "empresa.com".to_string(),
            },
        )
        .unwrap();

        assert!(!target.id.is_empty());
        assert_eq!(target.domain, "empresa.com");
        assert_eq!(target.engagement_id, eng.id);
    }

    #[test]
    fn test_list_targets_vazio() {
        let db = setup();
        let eng = create_test_engagement(&db);

        let lista = list_targets(&db, &eng.id).unwrap();
        assert!(lista.is_empty());
    }

    #[test]
    fn test_list_targets() {
        let db = setup();
        let eng = create_test_engagement(&db);

        create_target(
            &db,
            NewTarget {
                engagement_id: eng.id.clone(),
                domain: "empresa.com".to_string(),
            },
        )
        .unwrap();

        create_target(
            &db,
            NewTarget {
                engagement_id: eng.id.clone(),
                domain: "subsidiaria.com".to_string(),
            },
        )
        .unwrap();

        let lista = list_targets(&db, &eng.id).unwrap();
        assert_eq!(lista.len(), 2);
    }

    #[test]
    fn test_list_targets_isolado_por_engagement() {
        let db = setup();

        // Dois engagements distintos
        let eng1 = create_engagement(
            &db,
            NewEngagement {
                name: "Eng 1".to_string(),
                description: None,
            },
        )
        .unwrap();

        let eng2 = create_engagement(
            &db,
            NewEngagement {
                name: "Eng 2".to_string(),
                description: None,
            },
        )
        .unwrap();

        create_target(
            &db,
            NewTarget {
                engagement_id: eng1.id.clone(),
                domain: "alvo-do-eng1.com".to_string(),
            },
        )
        .unwrap();

        create_target(
            &db,
            NewTarget {
                engagement_id: eng2.id.clone(),
                domain: "alvo-do-eng2.com".to_string(),
            },
        )
        .unwrap();

        // Cada engagement deve ver apenas seus próprios targets
        let targets_eng1 = list_targets(&db, &eng1.id).unwrap();
        let targets_eng2 = list_targets(&db, &eng2.id).unwrap();

        assert_eq!(targets_eng1.len(), 1);
        assert_eq!(targets_eng2.len(), 1);
        assert_eq!(targets_eng1[0].domain, "alvo-do-eng1.com");
        assert_eq!(targets_eng2[0].domain, "alvo-do-eng2.com");
    }

    #[test]
    fn test_get_target_existente() {
        let db = setup();
        let eng = create_test_engagement(&db);

        let criado = create_target(
            &db,
            NewTarget {
                engagement_id: eng.id.clone(),
                domain: "alvo.com".to_string(),
            },
        )
        .unwrap();

        let encontrado = get_target(&db, &criado.id).unwrap();
        assert!(encontrado.is_some());
        assert_eq!(encontrado.unwrap().domain, "alvo.com");
    }

    #[test]
    fn test_get_target_inexistente() {
        let db = setup();
        let resultado = get_target(&db, "id-inexistente").unwrap();
        assert!(resultado.is_none());
    }

    #[test]
    fn test_delete_target() {
        let db = setup();
        let eng = create_test_engagement(&db);

        let criado = create_target(
            &db,
            NewTarget {
                engagement_id: eng.id.clone(),
                domain: "deletar.com".to_string(),
            },
        )
        .unwrap();

        delete_target(&db, &criado.id).unwrap();

        let resultado = get_target(&db, &criado.id).unwrap();
        assert!(resultado.is_none());
    }

    #[test]
    fn test_delete_engagement_cascata_targets() {
        let db = setup();
        let eng = create_test_engagement(&db);

        create_target(
            &db,
            NewTarget {
                engagement_id: eng.id.clone(),
                domain: "alvo1.com".to_string(),
            },
        )
        .unwrap();

        create_target(
            &db,
            NewTarget {
                engagement_id: eng.id.clone(),
                domain: "alvo2.com".to_string(),
            },
        )
        .unwrap();

        // Deletar o engagement deve deletar os targets por CASCADE
        delete_engagement(&db, &eng.id).unwrap();

        let lista = list_targets(&db, &eng.id).unwrap();
        assert!(lista.is_empty());
    }

    // ─── Helpers para subdomains ──────────────────────────────────────────────

    fn create_test_target(db: &Database, eng_id: &str) -> Target {
        create_target(
            db,
            NewTarget {
                engagement_id: eng_id.to_string(),
                domain: "empresa.com".to_string(),
            },
        )
        .unwrap()
    }

    // ─── Testes de Subdomain ──────────────────────────────────────────────────

    #[test]
    fn test_create_subdomain() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);

        let sub = create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "api.empresa.com".to_string(),
                status_code: Some(200),
                title: Some("API Principal".to_string()),
            },
        )
        .unwrap();

        assert!(!sub.id.is_empty());
        assert_eq!(sub.subdomain, "api.empresa.com");
        assert_eq!(sub.status, SubdomainStatus::NotVisited);
        assert_eq!(sub.status_code, Some(200));
        assert_eq!(sub.title, Some("API Principal".to_string()));
        assert_eq!(sub.notes, None);
    }

    #[test]
    fn test_create_subdomain_minimo() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);

        let sub = create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "mail.empresa.com".to_string(),
                status_code: None,
                title: None,
            },
        )
        .unwrap();

        assert_eq!(sub.status_code, None);
        assert_eq!(sub.title, None);
        assert_eq!(sub.status, SubdomainStatus::NotVisited);
    }

    #[test]
    fn test_subdomain_duplicado_falha() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);

        create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "api.empresa.com".to_string(),
                status_code: None,
                title: None,
            },
        )
        .unwrap();

        let resultado = create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "api.empresa.com".to_string(),
                status_code: None,
                title: None,
            },
        );

        assert!(resultado.is_err());
    }

    #[test]
    fn test_list_subdomains_vazio() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);

        let lista = list_subdomains(&db, &target.id).unwrap();
        assert!(lista.is_empty());
    }

    #[test]
    fn test_list_subdomains() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);

        create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "api.empresa.com".to_string(),
                status_code: Some(200),
                title: None,
            },
        )
        .unwrap();

        create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "admin.empresa.com".to_string(),
                status_code: Some(403),
                title: None,
            },
        )
        .unwrap();

        let lista = list_subdomains(&db, &target.id).unwrap();
        assert_eq!(lista.len(), 2);
    }

    #[test]
    fn test_list_subdomains_isolado_por_target() {
        let db = setup();
        let eng = create_test_engagement(&db);

        let target1 = create_target(
            &db,
            NewTarget {
                engagement_id: eng.id.clone(),
                domain: "empresa1.com".to_string(),
            },
        )
        .unwrap();

        let target2 = create_target(
            &db,
            NewTarget {
                engagement_id: eng.id.clone(),
                domain: "empresa2.com".to_string(),
            },
        )
        .unwrap();

        create_subdomain(
            &db,
            NewSubdomain {
                target_id: target1.id.clone(),
                subdomain: "api.empresa1.com".to_string(),
                status_code: None,
                title: None,
            },
        )
        .unwrap();

        create_subdomain(
            &db,
            NewSubdomain {
                target_id: target2.id.clone(),
                subdomain: "api.empresa2.com".to_string(),
                status_code: None,
                title: None,
            },
        )
        .unwrap();

        let subs1 = list_subdomains(&db, &target1.id).unwrap();
        let subs2 = list_subdomains(&db, &target2.id).unwrap();

        assert_eq!(subs1.len(), 1);
        assert_eq!(subs2.len(), 1);
        assert_eq!(subs1[0].subdomain, "api.empresa1.com");
        assert_eq!(subs2[0].subdomain, "api.empresa2.com");
    }

    #[test]
    fn test_get_subdomain_existente() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);

        let criado = create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "dev.empresa.com".to_string(),
                status_code: Some(200),
                title: Some("Dev".to_string()),
            },
        )
        .unwrap();

        let encontrado = get_subdomain(&db, &criado.id).unwrap();
        assert!(encontrado.is_some());
        assert_eq!(encontrado.unwrap().subdomain, "dev.empresa.com");
    }

    #[test]
    fn test_get_subdomain_inexistente() {
        let db = setup();
        let resultado = get_subdomain(&db, "id-inexistente").unwrap();
        assert!(resultado.is_none());
    }

    #[test]
    fn test_update_subdomain_status_e_notes() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);

        let criado = create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "login.empresa.com".to_string(),
                status_code: Some(200),
                title: None,
            },
        )
        .unwrap();

        let atualizado = update_subdomain(
            &db,
            &criado.id,
            UpdateSubdomain {
                status: Some(SubdomainStatus::Vulnerable),
                notes: Some("Tela de login com SQLi".to_string()),
                status_code: None,
                title: None,
            },
        )
        .unwrap();

        assert_eq!(atualizado.status, SubdomainStatus::Vulnerable);
        assert_eq!(atualizado.notes, Some("Tela de login com SQLi".to_string()));
        // status_code não mudou
        assert_eq!(atualizado.status_code, Some(200));
    }

    #[test]
    fn test_delete_subdomain() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);

        let criado = create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "old.empresa.com".to_string(),
                status_code: None,
                title: None,
            },
        )
        .unwrap();

        delete_subdomain(&db, &criado.id).unwrap();

        let resultado = get_subdomain(&db, &criado.id).unwrap();
        assert!(resultado.is_none());
    }

    #[test]
    fn test_list_subdomains_por_status() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);

        let sub1 = create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "api.empresa.com".to_string(),
                status_code: None,
                title: None,
            },
        )
        .unwrap();

        create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "admin.empresa.com".to_string(),
                status_code: None,
                title: None,
            },
        )
        .unwrap();

        // Marca apenas o primeiro como vulnerável
        update_subdomain(
            &db,
            &sub1.id,
            UpdateSubdomain {
                status: Some(SubdomainStatus::Vulnerable),
                notes: None,
                status_code: None,
                title: None,
            },
        )
        .unwrap();

        let vulneraveis =
            list_subdomains_by_status(&db, &target.id, SubdomainStatus::Vulnerable).unwrap();

        assert_eq!(vulneraveis.len(), 1);
        assert_eq!(vulneraveis[0].subdomain, "api.empresa.com");
    }

    #[test]
    fn test_delete_target_cascata_subdomains() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);

        create_subdomain(
            &db,
            NewSubdomain {
                target_id: target.id.clone(),
                subdomain: "api.empresa.com".to_string(),
                status_code: None,
                title: None,
            },
        )
        .unwrap();

        delete_target(&db, &target.id).unwrap();

        let lista = list_subdomains(&db, &target.id).unwrap();
        assert!(lista.is_empty());
    }

    // ─── Helper para subdomain ────────────────────────────────────────────────

    fn create_test_subdomain(db: &Database, target_id: &str, subdomain: &str) -> Subdomain {
        create_subdomain(
            db,
            NewSubdomain {
                target_id: target_id.to_string(),
                subdomain: subdomain.to_string(),
                status_code: None,
                title: None,
            },
        )
        .unwrap()
    }

    // ─── Testes de Tag ────────────────────────────────────────────────────────

    #[test]
    fn test_create_tag() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "api.empresa.com");

        let tag = create_tag(
            &db,
            NewTag {
                subdomain_id: sub.id.clone(),
                name: "login-page".to_string(),
            },
        )
        .unwrap();

        assert!(!tag.id.is_empty());
        assert_eq!(tag.name, "login-page");
        assert_eq!(tag.subdomain_id, sub.id);
    }

    #[test]
    fn test_tag_duplicada_falha() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "api.empresa.com");

        create_tag(
            &db,
            NewTag {
                subdomain_id: sub.id.clone(),
                name: "api-endpoint".to_string(),
            },
        )
        .unwrap();

        let resultado = create_tag(
            &db,
            NewTag {
                subdomain_id: sub.id.clone(),
                name: "api-endpoint".to_string(),
            },
        );

        assert!(resultado.is_err());
    }

    #[test]
    fn test_mesma_tag_em_subdomains_diferentes() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub1 = create_test_subdomain(&db, &target.id, "api.empresa.com");
        let sub2 = create_test_subdomain(&db, &target.id, "admin.empresa.com");

        // A mesma tag pode existir em subdomains diferentes
        create_tag(
            &db,
            NewTag {
                subdomain_id: sub1.id.clone(),
                name: "login-page".to_string(),
            },
        )
        .unwrap();

        let resultado = create_tag(
            &db,
            NewTag {
                subdomain_id: sub2.id.clone(),
                name: "login-page".to_string(),
            },
        );

        assert!(resultado.is_ok());
    }

    #[test]
    fn test_list_tags_vazio() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "api.empresa.com");

        let lista = list_tags(&db, &sub.id).unwrap();
        assert!(lista.is_empty());
    }

    #[test]
    fn test_list_tags() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "api.empresa.com");

        create_tag(
            &db,
            NewTag {
                subdomain_id: sub.id.clone(),
                name: "login-page".to_string(),
            },
        )
        .unwrap();

        create_tag(
            &db,
            NewTag {
                subdomain_id: sub.id.clone(),
                name: "interesting".to_string(),
            },
        )
        .unwrap();

        let lista = list_tags(&db, &sub.id).unwrap();
        assert_eq!(lista.len(), 2);
    }

    #[test]
    fn test_delete_tag_por_id() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "api.empresa.com");

        let tag = create_tag(
            &db,
            NewTag {
                subdomain_id: sub.id.clone(),
                name: "outdated".to_string(),
            },
        )
        .unwrap();

        delete_tag(&db, &tag.id).unwrap();

        let lista = list_tags(&db, &sub.id).unwrap();
        assert!(lista.is_empty());
    }

    #[test]
    fn test_delete_tag_por_nome() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "api.empresa.com");

        create_tag(
            &db,
            NewTag {
                subdomain_id: sub.id.clone(),
                name: "false-positive".to_string(),
            },
        )
        .unwrap();

        delete_tag_by_name(&db, &sub.id, "false-positive").unwrap();

        let lista = list_tags(&db, &sub.id).unwrap();
        assert!(lista.is_empty());
    }

    #[test]
    fn test_delete_subdomain_cascata_tags() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "api.empresa.com");

        create_tag(
            &db,
            NewTag {
                subdomain_id: sub.id.clone(),
                name: "api-endpoint".to_string(),
            },
        )
        .unwrap();

        delete_subdomain(&db, &sub.id).unwrap();

        // Tags devem ter sido deletadas por CASCADE
        let lista = list_tags(&db, &sub.id).unwrap();
        assert!(lista.is_empty());
    }

    // ─── Testes de Technology ─────────────────────────────────────────────────

    #[test]
    fn test_create_technology() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "app.empresa.com");

        let tech = create_technology(
            &db,
            NewTechnology {
                subdomain_id: sub.id.clone(),
                name: "WordPress".to_string(),
                version: Some("6.4".to_string()),
            },
        )
        .unwrap();

        assert!(!tech.id.is_empty());
        assert_eq!(tech.name, "WordPress");
        assert_eq!(tech.version, Some("6.4".to_string()));
        assert_eq!(tech.subdomain_id, sub.id);
    }

    #[test]
    fn test_create_technology_sem_versao() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "app.empresa.com");

        let tech = create_technology(
            &db,
            NewTechnology {
                subdomain_id: sub.id.clone(),
                name: "React".to_string(),
                version: None,
            },
        )
        .unwrap();

        assert_eq!(tech.version, None);
    }

    #[test]
    fn test_list_technologies_vazio() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "app.empresa.com");

        let lista = list_technologies(&db, &sub.id).unwrap();
        assert!(lista.is_empty());
    }

    #[test]
    fn test_list_technologies() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "app.empresa.com");

        create_technology(
            &db,
            NewTechnology {
                subdomain_id: sub.id.clone(),
                name: "WordPress".to_string(),
                version: Some("6.4".to_string()),
            },
        )
        .unwrap();

        create_technology(
            &db,
            NewTechnology {
                subdomain_id: sub.id.clone(),
                name: "jQuery".to_string(),
                version: Some("3.6".to_string()),
            },
        )
        .unwrap();

        let lista = list_technologies(&db, &sub.id).unwrap();
        assert_eq!(lista.len(), 2);
    }

    #[test]
    fn test_delete_technology() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "app.empresa.com");

        let tech = create_technology(
            &db,
            NewTechnology {
                subdomain_id: sub.id.clone(),
                name: "Apache".to_string(),
                version: None,
            },
        )
        .unwrap();

        delete_technology(&db, &tech.id).unwrap();

        let lista = list_technologies(&db, &sub.id).unwrap();
        assert!(lista.is_empty());
    }

    #[test]
    fn test_delete_subdomain_cascata_technologies() {
        let db = setup();
        let eng = create_test_engagement(&db);
        let target = create_test_target(&db, &eng.id);
        let sub = create_test_subdomain(&db, &target.id, "app.empresa.com");

        create_technology(
            &db,
            NewTechnology {
                subdomain_id: sub.id.clone(),
                name: "Nginx".to_string(),
                version: Some("1.24".to_string()),
            },
        )
        .unwrap();

        delete_subdomain(&db, &sub.id).unwrap();

        let lista = list_technologies(&db, &sub.id).unwrap();
        assert!(lista.is_empty());
    }
}
