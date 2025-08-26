use fantasy_core_rs::ExternalId;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ConfigModel {
    pub landing_page: DefaultLandingPage,
}
impl Default for ConfigModel {
    fn default() -> Self {
        ConfigModel {
            landing_page: DefaultLandingPage::default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum DefaultLandingPage {
    WelcomePage,
    LeaguesPage,
    LeagueSummary(ExternalId),
}
impl Default for DefaultLandingPage {
    fn default() -> Self {
        DefaultLandingPage::WelcomePage
    }
}
