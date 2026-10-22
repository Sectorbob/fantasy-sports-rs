use fantasy_sports_core::ExternalId;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ConfigModel {
    pub landing_page: DefaultLandingPage,
    /// Sleeper user whose leagues are listed; Sleeper is skipped when unset.
    #[serde(default)]
    pub sleeper_user_id: Option<String>,
    /// Settings for the league history / owner records page.
    #[serde(default)]
    pub history: HistoryConfig,
}
impl Default for ConfigModel {
    fn default() -> Self {
        ConfigModel {
            landing_page: DefaultLandingPage::default(),
            sleeper_user_id: None,
            history: HistoryConfig::default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default, PartialEq)]
pub struct HistoryConfig {
    /// Only leagues with this exact name are included; all leagues are used when unset.
    #[serde(default)]
    pub league_name: Option<String>,
    /// People who appear under different ids across seasons and platforms.
    #[serde(default)]
    pub owners: Vec<OwnerAlias>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct OwnerAlias {
    pub name: String,
    /// every Sleeper user id / Yahoo guid this person has used
    pub ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_config_files_without_the_new_fields_still_load() {
        let cfg: ConfigModel = serde_json::from_str(r#"{"landing_page":"WelcomePage"}"#).unwrap();
        assert_eq!(None, cfg.sleeper_user_id);
        assert_eq!(HistoryConfig::default(), cfg.history);
    }

    #[test]
    fn example_config_parses() {
        let text = std::fs::read_to_string("config.example.json").unwrap();
        let cfg: ConfigModel = serde_json::from_str(&text).unwrap();
        assert!(cfg.sleeper_user_id.is_some());
        assert_eq!(2, cfg.history.owners.len());
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
