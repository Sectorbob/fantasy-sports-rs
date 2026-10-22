use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct User {
    pub username: Option<String>, // present for get_user, but not get_users_in_league
    pub user_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
    #[serde(flatten)]
    pub league_details: Option<LeagueUserDetails>,
    #[serde(flatten)]
    pub other_data: HashMap<String, Value>,
}
impl User {
    /// Returns the team name of this [`User`].
    pub fn team_name(&self) -> String {
        match &self.league_details {
            Some(league_user_details) => match league_user_details.metadata.get("team_name") {
                Some(raw_team_name) => match raw_team_name.as_str() {
                    Some(team_name) => String::from(team_name),
                    None => todo!(), // data error
                },
                None => format!("Team {}", &self.display_name.as_str()),
            },
            None => format!("Team {}", &self.display_name.as_str()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LeagueUserDetails {
    pub is_bot: bool,
    pub is_owner: bool,
    pub league_id: String,
    pub metadata: HashMap<String, Value>,
    // #[serde(default = "empty_map")]
    pub settings: Option<HashMap<String, Value>>,
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_team_name() {
        use super::{LeagueUserDetails, User};
        use serde_json::json;
        use std::collections::HashMap;

        let user_1 = User {
            username: Some("bob_barker_123".to_string()),
            user_id: "3969562440090994".to_string(),
            display_name: "Bob Barker".to_string(),
            avatar: None,
            league_details: Some(LeagueUserDetails {
                is_bot: false,
                is_owner: true,
                league_id: "".to_string(),
                metadata: HashMap::from([("team_name".to_string(), json!("The Glorious One"))]),
                settings: None,
            }),
            other_data: HashMap::with_capacity(0),
        };

        assert_eq!("The Glorious One", user_1.team_name());
    }
}
