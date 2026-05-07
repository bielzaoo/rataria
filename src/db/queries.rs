use crate::db::models::{Engagement, NewEngagement};
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
}
