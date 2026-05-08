use crate::db::Database;

/// Qual tela está sendo exibida
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Password,
    Home,
}

/// Estado global da aplicação
pub struct App {
    pub screen: Screen,
    pub db: Option<Database>,
    pub should_quit: bool,

    // Estado da tela de senha
    pub password_input: String,
    pub password_error: Option<String>,

    // Estado do menu home
    pub home_selected: usize,
}

impl App {
    pub fn new() -> Self {
        App {
            screen: Screen::Password,
            db: None,
            should_quit: false,
            password_input: String::new(),
            password_error: None,
            home_selected: 0,
        }
    }

    pub fn home_menu_items() -> Vec<&'static str> {
        vec![
            "Abrir engagement existente",
            "Criar novo engagement",
            "Sair",
        ]
    }

    pub fn home_next(&mut self) {
        let len = Self::home_menu_items().len();
        self.home_selected = (self.home_selected + 1) % len;
    }

    pub fn home_previous(&mut self) {
        let len = Self::home_menu_items().len();
        if self.home_selected == 0 {
            self.home_selected = len - 1;
        } else {
            self.home_selected -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_inicia_na_tela_de_senha() {
        let app = App::new();
        assert_eq!(app.screen, Screen::Password);
    }

    #[test]
    fn test_app_inicia_sem_erro() {
        let app = App::new();
        assert!(app.password_error.is_none());
    }

    #[test]
    fn test_app_inicia_sem_db() {
        let app = App::new();
        assert!(app.db.is_none());
    }

    #[test]
    fn test_home_next_navega_para_frente() {
        let mut app = App::new();
        assert_eq!(app.home_selected, 0);
        app.home_next();
        assert_eq!(app.home_selected, 1);
        app.home_next();
        assert_eq!(app.home_selected, 2);
    }

    #[test]
    fn test_home_next_wrap_no_final() {
        let mut app = App::new();
        let total = App::home_menu_items().len();
        for _ in 0..total {
            app.home_next();
        }
        assert_eq!(app.home_selected, 0);
    }

    #[test]
    fn test_home_previous_navega_para_tras() {
        let mut app = App::new();
        app.home_selected = 2;
        app.home_previous();
        assert_eq!(app.home_selected, 1);
        app.home_previous();
        assert_eq!(app.home_selected, 0);
    }

    #[test]
    fn test_home_previous_wrap_no_inicio() {
        let mut app = App::new();
        assert_eq!(app.home_selected, 0);
        app.home_previous();
        assert_eq!(app.home_selected, App::home_menu_items().len() - 1);
    }

    #[test]
    fn test_home_menu_tem_3_itens() {
        assert_eq!(App::home_menu_items().len(), 3);
    }
}
