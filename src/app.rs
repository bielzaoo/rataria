use crate::db::{models::Engagement, Database};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Password,
    Home,
    CreateEngagement,
    ListEngagements,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormField {
    Name,
    Description,
}

pub struct App {
    pub screen: Screen,
    pub db: Option<Database>,
    pub should_quit: bool,

    // Estado da tela de senha
    pub password_input: String,
    pub password_error: Option<String>,

    // Estado do menu home
    pub home_selected: usize,

    // Estado do formulário de criação
    pub form_name: String,
    pub form_description: String,
    pub form_field: FormField,
    pub form_error: Option<String>,

    // Estado da lista de engagements
    pub engagements: Vec<Engagement>,
    pub engagement_selected: usize,
    pub current_engagement: Option<Engagement>,
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
            form_name: String::new(),
            form_description: String::new(),
            form_field: FormField::Name,
            form_error: None,
            engagements: Vec::new(),
            engagement_selected: 0,
            current_engagement: None,
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

    /// Limpa o formulário de criação
    pub fn reset_form(&mut self) {
        self.form_name.clear();
        self.form_description.clear();
        self.form_field = FormField::Name;
        self.form_error = None;
    }

    /// Alterna entre os campos do formulário
    pub fn form_next_field(&mut self) {
        self.form_field = match self.form_field {
            FormField::Name => FormField::Description,
            FormField::Description => FormField::Name,
        };
    }

    /// Navega para cima na lista de engagements
    pub fn engagements_previous(&mut self) {
        if self.engagements.is_empty() {
            return;
        }
        if self.engagement_selected == 0 {
            self.engagement_selected = self.engagements.len() - 1;
        } else {
            self.engagement_selected -= 1;
        }
    }

    /// Navega para baixo na lista de engagements
    pub fn engagements_next(&mut self) {
        if self.engagements.is_empty() {
            return;
        }
        self.engagement_selected = (self.engagement_selected + 1) % self.engagements.len();
    }

    /// Retorna o engagement atualmente selecionado na lista
    pub fn selected_engagement(&self) -> Option<&Engagement> {
        self.engagements.get(self.engagement_selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn make_engagement(name: &str) -> Engagement {
        Engagement {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: None,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }

    // ── testes originais ─────────────────────────────────────────────────────

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

    // ── testes de formulário ─────────────────────────────────────────────────

    #[test]
    fn test_form_inicia_vazio() {
        let app = App::new();
        assert!(app.form_name.is_empty());
        assert!(app.form_description.is_empty());
        assert_eq!(app.form_field, FormField::Name);
        assert!(app.form_error.is_none());
    }

    #[test]
    fn test_reset_form_limpa_tudo() {
        let mut app = App::new();
        app.form_name = "Empresa X".to_string();
        app.form_description = "Descrição".to_string();
        app.form_field = FormField::Description;
        app.form_error = Some("erro".to_string());

        app.reset_form();

        assert!(app.form_name.is_empty());
        assert!(app.form_description.is_empty());
        assert_eq!(app.form_field, FormField::Name);
        assert!(app.form_error.is_none());
    }

    #[test]
    fn test_form_next_field_alterna() {
        let mut app = App::new();
        assert_eq!(app.form_field, FormField::Name);
        app.form_next_field();
        assert_eq!(app.form_field, FormField::Description);
        app.form_next_field();
        assert_eq!(app.form_field, FormField::Name);
    }

    // ── testes de lista de engagements ────────────────────────────────────────

    #[test]
    fn test_engagements_next_navega() {
        let mut app = App::new();
        app.engagements = vec![
            make_engagement("A"),
            make_engagement("B"),
            make_engagement("C"),
        ];

        assert_eq!(app.engagement_selected, 0);
        app.engagements_next();
        assert_eq!(app.engagement_selected, 1);
        app.engagements_next();
        assert_eq!(app.engagement_selected, 2);
    }

    #[test]
    fn test_engagements_next_wrap() {
        let mut app = App::new();
        app.engagements = vec![make_engagement("A"), make_engagement("B")];
        app.engagement_selected = 1;

        app.engagements_next();
        assert_eq!(app.engagement_selected, 0);
    }

    #[test]
    fn test_engagements_previous_navega() {
        let mut app = App::new();
        app.engagements = vec![
            make_engagement("A"),
            make_engagement("B"),
            make_engagement("C"),
        ];
        app.engagement_selected = 2;

        app.engagements_previous();
        assert_eq!(app.engagement_selected, 1);
        app.engagements_previous();
        assert_eq!(app.engagement_selected, 0);
    }

    #[test]
    fn test_engagements_previous_wrap() {
        let mut app = App::new();
        app.engagements = vec![make_engagement("A"), make_engagement("B")];
        app.engagement_selected = 0;

        app.engagements_previous();
        assert_eq!(app.engagement_selected, 1);
    }

    #[test]
    fn test_engagements_next_vazio_nao_crasha() {
        let mut app = App::new();
        app.engagements_next();
        assert_eq!(app.engagement_selected, 0);
    }

    #[test]
    fn test_engagements_previous_vazio_nao_crasha() {
        let mut app = App::new();
        app.engagements_previous();
        assert_eq!(app.engagement_selected, 0);
    }

    #[test]
    fn test_selected_engagement_retorna_correto() {
        let mut app = App::new();
        let eng_b = make_engagement("B");
        app.engagements = vec![make_engagement("A"), eng_b.clone(), make_engagement("C")];
        app.engagement_selected = 1;

        let selected = app.selected_engagement().unwrap();
        assert_eq!(selected.name, "B");
        assert_eq!(selected.id, eng_b.id);
    }

    #[test]
    fn test_selected_engagement_vazio_retorna_none() {
        let app = App::new();
        assert!(app.selected_engagement().is_none());
    }
}
