use crate::db::{models::Engagement, Database};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Password,
    Home,
    CreateEngagement,
    ListEngagements,
    Dashboard,
    Targets,
    Subdomains,
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

    // Estado do dashboard
    pub dashboard_selected: usize,

    // Estado de targets
    pub targets: Vec<crate::db::models::Target>,
    pub target_selected: usize,
    pub current_target: Option<crate::db::models::Target>,

    // Estado do formulário de target (reutiliza form_name)
    pub creating_target: bool,
    // Estado de subdomains
    pub subdomains: Vec<crate::db::models::Subdomain>,
    pub subdomain_selected: usize,
    pub current_subdomain: Option<crate::db::models::Subdomain>,
    pub creating_subdomain: bool,
    pub subdomain_filter: Option<crate::db::models::SubdomainStatus>,

    // Edição inline de subdomain
    pub editing_notes: bool,
    pub form_notes: String,
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
            dashboard_selected: 0,
            targets: Vec::new(),
            target_selected: 0,
            current_target: None,
            creating_target: false,
            subdomains: Vec::new(),
            subdomain_selected: 0,
            current_subdomain: None,
            creating_subdomain: false,
            subdomain_filter: None,
            editing_notes: false,
            form_notes: String::new(),
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

    pub fn dashboard_menu_items() -> Vec<&'static str> {
        vec![
            "Targets",
            "Subdomains",
            "IPs",
            "ASNs",
            "URLs",
            "Technologies",
            "Screenshots",
        ]
    }

    pub fn dashboard_next(&mut self) {
        let len = Self::dashboard_menu_items().len();
        self.dashboard_selected = (self.dashboard_selected + 1) % len;
    }

    pub fn dashboard_previous(&mut self) {
        let len = Self::dashboard_menu_items().len();
        if self.dashboard_selected == 0 {
            self.dashboard_selected = len - 1;
        } else {
            self.dashboard_selected -= 1;
        }
    }

    pub fn targets_next(&mut self) {
        if self.targets.is_empty() {
            return;
        }
        self.target_selected = (self.target_selected + 1) % self.targets.len();
    }

    pub fn targets_previous(&mut self) {
        if self.targets.is_empty() {
            return;
        }
        if self.target_selected == 0 {
            self.target_selected = self.targets.len() - 1;
        } else {
            self.target_selected -= 1;
        }
    }

    pub fn selected_target(&self) -> Option<&crate::db::models::Target> {
        self.targets.get(self.target_selected)
    }

    pub fn subdomains_next(&mut self) {
        if self.subdomains.is_empty() {
            return;
        }
        self.subdomain_selected = (self.subdomain_selected + 1) % self.subdomains.len();
    }

    pub fn subdomains_previous(&mut self) {
        if self.subdomains.is_empty() {
            return;
        }
        if self.subdomain_selected == 0 {
            self.subdomain_selected = self.subdomains.len() - 1;
        } else {
            self.subdomain_selected -= 1;
        }
    }

    pub fn selected_subdomain(&self) -> Option<&crate::db::models::Subdomain> {
        self.subdomains.get(self.subdomain_selected)
    }

    pub fn subdomains_filtered(&self) -> Vec<&crate::db::models::Subdomain> {
        match &self.subdomain_filter {
            None => self.subdomains.iter().collect(),
            Some(status) => self
                .subdomains
                .iter()
                .filter(|s| &s.status == status)
                .collect(),
        }
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

    // ── helpers de target ────────────────────────────────────────────────────

    fn make_target(domain: &str) -> crate::db::models::Target {
        crate::db::models::Target {
            id: uuid::Uuid::new_v4().to_string(),
            engagement_id: uuid::Uuid::new_v4().to_string(),
            domain: domain.to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    // ── testes de dashboard ───────────────────────────────────────────────────

    #[test]
    fn test_dashboard_inicia_no_primeiro_item() {
        let app = App::new();
        assert_eq!(app.dashboard_selected, 0);
    }

    #[test]
    fn test_dashboard_menu_tem_7_itens() {
        assert_eq!(App::dashboard_menu_items().len(), 7);
    }

    #[test]
    fn test_dashboard_next_navega() {
        let mut app = App::new();
        app.dashboard_next();
        assert_eq!(app.dashboard_selected, 1);
        app.dashboard_next();
        assert_eq!(app.dashboard_selected, 2);
    }

    #[test]
    fn test_dashboard_next_wrap() {
        let mut app = App::new();
        let total = App::dashboard_menu_items().len();
        for _ in 0..total {
            app.dashboard_next();
        }
        assert_eq!(app.dashboard_selected, 0);
    }

    #[test]
    fn test_dashboard_previous_navega() {
        let mut app = App::new();
        app.dashboard_selected = 3;
        app.dashboard_previous();
        assert_eq!(app.dashboard_selected, 2);
        app.dashboard_previous();
        assert_eq!(app.dashboard_selected, 1);
    }

    #[test]
    fn test_dashboard_previous_wrap() {
        let mut app = App::new();
        assert_eq!(app.dashboard_selected, 0);
        app.dashboard_previous();
        assert_eq!(
            app.dashboard_selected,
            App::dashboard_menu_items().len() - 1
        );
    }

    // ── testes de targets ─────────────────────────────────────────────────────

    #[test]
    fn test_targets_next_navega() {
        let mut app = App::new();
        app.targets = vec![
            make_target("empresa.com"),
            make_target("subsidiaria.com"),
            make_target("outro.com"),
        ];
        assert_eq!(app.target_selected, 0);
        app.targets_next();
        assert_eq!(app.target_selected, 1);
        app.targets_next();
        assert_eq!(app.target_selected, 2);
    }

    #[test]
    fn test_targets_next_wrap() {
        let mut app = App::new();
        app.targets = vec![make_target("a.com"), make_target("b.com")];
        app.target_selected = 1;
        app.targets_next();
        assert_eq!(app.target_selected, 0);
    }

    #[test]
    fn test_targets_previous_navega() {
        let mut app = App::new();
        app.targets = vec![
            make_target("a.com"),
            make_target("b.com"),
            make_target("c.com"),
        ];
        app.target_selected = 2;
        app.targets_previous();
        assert_eq!(app.target_selected, 1);
        app.targets_previous();
        assert_eq!(app.target_selected, 0);
    }

    #[test]
    fn test_targets_previous_wrap() {
        let mut app = App::new();
        app.targets = vec![make_target("a.com"), make_target("b.com")];
        app.target_selected = 0;
        app.targets_previous();
        assert_eq!(app.target_selected, 1);
    }

    #[test]
    fn test_targets_next_vazio_nao_crasha() {
        let mut app = App::new();
        app.targets_next();
        assert_eq!(app.target_selected, 0);
    }

    #[test]
    fn test_targets_previous_vazio_nao_crasha() {
        let mut app = App::new();
        app.targets_previous();
        assert_eq!(app.target_selected, 0);
    }

    #[test]
    fn test_selected_target_retorna_correto() {
        let mut app = App::new();
        let t = make_target("alvo.com");
        app.targets = vec![make_target("a.com"), t.clone(), make_target("b.com")];
        app.target_selected = 1;
        let selected = app.selected_target().unwrap();
        assert_eq!(selected.domain, "alvo.com");
        assert_eq!(selected.id, t.id);
    }

    #[test]
    fn test_selected_target_vazio_retorna_none() {
        let app = App::new();
        assert!(app.selected_target().is_none());
    }

    #[test]
    fn test_creating_target_inicia_false() {
        let app = App::new();
        assert!(!app.creating_target);
    }

    // ── helpers de subdomain ──────────────────────────────────────────────────

    fn make_subdomain(
        sub: &str,
        status: crate::db::models::SubdomainStatus,
    ) -> crate::db::models::Subdomain {
        crate::db::models::Subdomain {
            id: uuid::Uuid::new_v4().to_string(),
            target_id: uuid::Uuid::new_v4().to_string(),
            subdomain: sub.to_string(),
            status,
            notes: None,
            status_code: None,
            title: None,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }

    // ── testes de subdomains ──────────────────────────────────────────────────

    #[test]
    fn test_subdomains_next_navega() {
        let mut app = App::new();
        app.subdomains = vec![
            make_subdomain("api.x.com", crate::db::models::SubdomainStatus::NotVisited),
            make_subdomain(
                "admin.x.com",
                crate::db::models::SubdomainStatus::NotVisited,
            ),
            make_subdomain("dev.x.com", crate::db::models::SubdomainStatus::NotVisited),
        ];
        assert_eq!(app.subdomain_selected, 0);
        app.subdomains_next();
        assert_eq!(app.subdomain_selected, 1);
        app.subdomains_next();
        assert_eq!(app.subdomain_selected, 2);
    }

    #[test]
    fn test_subdomains_next_wrap() {
        let mut app = App::new();
        app.subdomains = vec![
            make_subdomain("a.x.com", crate::db::models::SubdomainStatus::NotVisited),
            make_subdomain("b.x.com", crate::db::models::SubdomainStatus::NotVisited),
        ];
        app.subdomain_selected = 1;
        app.subdomains_next();
        assert_eq!(app.subdomain_selected, 0);
    }

    #[test]
    fn test_subdomains_previous_navega() {
        let mut app = App::new();
        app.subdomains = vec![
            make_subdomain("a.x.com", crate::db::models::SubdomainStatus::NotVisited),
            make_subdomain("b.x.com", crate::db::models::SubdomainStatus::NotVisited),
            make_subdomain("c.x.com", crate::db::models::SubdomainStatus::NotVisited),
        ];
        app.subdomain_selected = 2;
        app.subdomains_previous();
        assert_eq!(app.subdomain_selected, 1);
        app.subdomains_previous();
        assert_eq!(app.subdomain_selected, 0);
    }

    #[test]
    fn test_subdomains_previous_wrap() {
        let mut app = App::new();
        app.subdomains = vec![
            make_subdomain("a.x.com", crate::db::models::SubdomainStatus::NotVisited),
            make_subdomain("b.x.com", crate::db::models::SubdomainStatus::NotVisited),
        ];
        app.subdomain_selected = 0;
        app.subdomains_previous();
        assert_eq!(app.subdomain_selected, 1);
    }

    #[test]
    fn test_subdomains_next_vazio_nao_crasha() {
        let mut app = App::new();
        app.subdomains_next();
        assert_eq!(app.subdomain_selected, 0);
    }

    #[test]
    fn test_subdomains_previous_vazio_nao_crasha() {
        let mut app = App::new();
        app.subdomains_previous();
        assert_eq!(app.subdomain_selected, 0);
    }

    #[test]
    fn test_selected_subdomain_retorna_correto() {
        let mut app = App::new();
        let s = make_subdomain("api.x.com", crate::db::models::SubdomainStatus::Vulnerable);
        app.subdomains = vec![
            make_subdomain("a.x.com", crate::db::models::SubdomainStatus::NotVisited),
            s.clone(),
        ];
        app.subdomain_selected = 1;
        let selected = app.selected_subdomain().unwrap();
        assert_eq!(selected.subdomain, "api.x.com");
        assert_eq!(selected.id, s.id);
    }

    #[test]
    fn test_selected_subdomain_vazio_retorna_none() {
        let app = App::new();
        assert!(app.selected_subdomain().is_none());
    }

    #[test]
    fn test_subdomains_filtered_sem_filtro_retorna_todos() {
        let mut app = App::new();
        app.subdomains = vec![
            make_subdomain("a.x.com", crate::db::models::SubdomainStatus::NotVisited),
            make_subdomain("b.x.com", crate::db::models::SubdomainStatus::Vulnerable),
            make_subdomain("c.x.com", crate::db::models::SubdomainStatus::Reviewed),
        ];
        assert_eq!(app.subdomains_filtered().len(), 3);
    }

    #[test]
    fn test_subdomains_filtered_com_filtro() {
        let mut app = App::new();
        app.subdomains = vec![
            make_subdomain("a.x.com", crate::db::models::SubdomainStatus::NotVisited),
            make_subdomain("b.x.com", crate::db::models::SubdomainStatus::Vulnerable),
            make_subdomain("c.x.com", crate::db::models::SubdomainStatus::Vulnerable),
        ];
        app.subdomain_filter = Some(crate::db::models::SubdomainStatus::Vulnerable);
        assert_eq!(app.subdomains_filtered().len(), 2);
    }

    #[test]
    fn test_subdomains_filter_none_inicial() {
        let app = App::new();
        assert!(app.subdomain_filter.is_none());
    }

    #[test]
    fn test_editing_notes_inicia_false() {
        let app = App::new();
        assert!(!app.editing_notes);
    }
}
