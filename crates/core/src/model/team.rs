use sleeper_fantasy_rs::{self as sleeper, User};
use std::{collections::HashMap, fmt};
use yahoo_fantasy_rs as yahoo;

#[derive(Debug, Clone)]
pub struct Team {
    pub id: String,
    pub roster_id: String,
    pub team_name: String,
    pub owner_id: Option<String>,
    pub owner_name: Option<String>,
}
impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{name}", name = self.team_name)
    }
}
impl From<&yahoo::Team> for Team {
    fn from(value: &yahoo::Team) -> Self {
        let managers = value.managers();
        let manager = managers.first();
        Team {
            id: value.team_id.to_string(),
            owner_id: manager.map(|m| m.guid.clone()),
            roster_id: value.team_id.to_string(),
            team_name: value.name.clone(),
            owner_name: manager.map(|m| m.nickname.clone()),
        }
    }
}

impl From<(&sleeper::Roster, &HashMap<String, sleeper::User>)> for Team {
    fn from((roster, users): (&sleeper::Roster, &HashMap<String, sleeper::User>)) -> Self {
        let user = if let Some(id) = &roster.owner_id {
            users.get(id)
        } else {
            None
        };

        Team {
            id: roster.roster_id.to_string(),
            owner_id: user.map(|u| u.user_id.clone()),
            roster_id: roster.roster_id.to_string(),
            team_name: user.map_or(format!("Team {}", roster.roster_id), User::team_name),
            owner_name: user.map(|u| u.display_name.clone()),
        }
    }
}
