use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, Utc, serde::ts_milliseconds};
use fantasy_sports_cache::Error as CachedClientError;
use fantasy_sports_sleeper as sleeper;
use serde::{Deserialize, Serialize};
use serenity::all::{
    Colour, EditMember, EditRole, GuildId, Http, Member, Permissions, Role, RoleId, UserId,
};

use crate::{
    ecr::keepers::{Keepable, KeepablesReport, get_keeper_status, get_player_history},
    fantasypros,
    store::{self, DataStore, DataStoreError, ExternalId, League, Platform},
};

pub const SPORT: sleeper::Sport = sleeper::Sport::NFL;

pub struct Service {
    sleeper_cli: fantasy_sports_sleeper::Client,
    fantasypros_cli: fantasypros::Client,
    store: Arc<DataStore>,
}
impl Service {
    pub async fn new(store_arc: &Arc<DataStore>) -> Result<Self, ECRError> {
        Ok(Service::with_clients(
            fantasy_sports_sleeper::Client::new(),
            fantasypros::Client::new(),
            store_arc,
        ))
    }

    /// A service over the given clients, for another Sleeper or FantasyPros server (or a test one).
    pub fn with_clients(
        sleeper_cli: fantasy_sports_sleeper::Client,
        fantasypros_cli: fantasypros::Client,
        store_arc: &Arc<DataStore>,
    ) -> Self {
        Service {
            sleeper_cli,
            fantasypros_cli,
            store: Arc::clone(store_arc),
        }
    }

    pub async fn fetch_keepable_context<T: Into<String>>(
        &self,
        league_id: T,
        sport: &sleeper::Sport,
    ) -> Result<KeepablesContext, ECRError> {
        let ecr_response_future = self.fantasypros_cli.fetch_ecr_half_ppr();
        let league_id: String = league_id.into();
        let players_future = self.sleeper_cli.fetch_all_players(sport);
        let league_future = self.sleeper_cli.get_league(league_id.clone());
        let rosters_future = self.sleeper_cli.get_rosters_in_league(league_id.clone());
        let users_future = self.sleeper_cli.get_users_for_league(league_id.clone());
        let league = league_future.await?;
        let draft_history = self.fetch_all_draft_history_sleeper(league.clone()).await?;
        let mut players = players_future.await?;
        let rosters = rosters_future.await?;
        let users = users_future.await?;
        let ecr_response = ecr_response_future.await?;

        // look through drafts and rosters (later transactions?????)
        // and get a set of player_ids to include in players_map
        reduce_players_map(&mut players, Some(&draft_history), Some(&rosters));

        Ok(KeepablesContext {
            fetched_at: Utc::now(), //.with_timezone(&chrono_tz::America::New_York),
            current_league: league,
            draft_history: draft_history
                .into_iter()
                .map(|e| (e.draft.season.clone(), e))
                .collect(),
            players,
            rosters,
            users,
            rankings: ecr_response,
        })
    }

    pub fn get_keepables(&self, ctx: &KeepablesContext) -> Result<KeepablesReport, ECRError> {
        Self::compute_keepables(ctx)
    }

    /// Pure calculation over an already-fetched context; needs no clients or datastore.
    pub fn compute_keepables(ctx: &KeepablesContext) -> Result<KeepablesReport, ECRError> {
        let num_teams = ctx.current_league.settings.num_teams;
        let mut keepables = vec![];
        let current_season = ctx.current_league.season.clone();
        let last_season: i32 = current_season
            .parse()
            .map_err(|e| {
                ECRError::Unknown(format!(
                    "league.season is invalid ({}): {e}",
                    ctx.current_league.season
                ))
            })
            .map(|v: i32| v - 1)?;

        let draft_result = ctx
            .draft_history
            .get(last_season.to_string().as_str())
            .ok_or(ECRError::NotFound(format!(
                "unable to find draft_history for {}",
                last_season
            )))?;
        for roster in &ctx.rosters {
            let user = ctx
                .users
                .iter()
                .find(|u| roster.owner_id.is_some() && roster.owner_id.clone().unwrap() == u.user_id).cloned()
                .unwrap_or_else(|| {
                    log::warn!("for some reason, we could not find the user info {user_id:?} for roster({roster_id})",
                        user_id = roster.owner_id,
                        roster_id = roster.roster_id,
                    );
                    sleeper::User {
                        username: None,
                        user_id: roster.owner_id.clone().unwrap_or_default(),
                        display_name: format!("Roster({:?})", roster.owner_id),
                        avatar: None,
                        league_details: None,
                        other_data: HashMap::new(),
                    }
                });
            log::debug!("User: {}", user.display_name);
            for player_id in &roster.players {
                let (player_name, player_pos, player_team) =
                    ctx.players.get(player_id.as_str()).map_or_else(
                        || {
                            log::warn!("Unable to find player {}", player_id);
                            (
                                format!("Unknown Player ({player_id})"),
                                String::from(""),
                                String::from(""),
                            )
                        },
                        |player| {
                            (
                                player.to_string(),
                                player
                                    .position
                                    .clone()
                                    .map_or(String::new(), |v| v.to_string()),
                                player.team.clone().unwrap_or_default(),
                            )
                        },
                    );
                let keeper_status = get_keeper_status(player_id.as_str(), &draft_result.picks);
                if keeper_status.is_keeper || keeper_status.was_drafted {
                    let player_history = get_player_history(player_id.as_str(), &ctx.draft_history);
                    log::debug!(
                        "{name} [${position}] [{team}] Keeper[{is_keeper}] Rd[{draft_round}]",
                        name = player_name,
                        position = player_pos,
                        team = player_team,
                        is_keeper = keeper_status.is_keeper,
                        draft_round = keeper_status.draft_round.unwrap_or_default(),
                    );
                    let player_rank = ctx
                        .rankings
                        .players
                        .iter()
                        .find(|p| p.player_name == player_name)
                        .map(|pr| {
                            let round = pr.rank_ecr / num_teams + 1;
                            let pick_in_round = pr.rank_ecr % num_teams + 1;
                            format!("{round}.{pick_in_round}")
                        });
                    keepables.push(Keepable {
                        owner_id: user.user_id.clone(),
                        owner_name: user.display_name.clone(),
                        was_keeper_last_year: keeper_status.is_keeper,
                        player_id: player_id.clone(),
                        player_name,
                        position: player_pos,
                        round: keeper_status.draft_round.unwrap_or_default(),
                        history: player_history,
                        player_rank,
                    });
                }
            }
        }

        // try fancy sort of keepables
        keepables.sort_by_key(|k| {
            k.owner_id
                .parse::<u64>()
                .unwrap_or(u64::MIN)
                .saturating_add(k.round as u64)
        });

        Ok(KeepablesReport {
            season: current_season,
            keepables,
        })
    }

    async fn fetch_all_draft_history_sleeper(
        &self,
        league: sleeper::League,
    ) -> Result<Vec<DraftResult>, ECRError> {
        let mut drafts: Vec<DraftResult> = vec![];
        let mut draft_futures = vec![];
        let mut current_league = league;
        while let Some(league_id) = current_league.previous_league_id {
            if league_id.is_empty() || league_id == "0" {
                // end of the line
                break;
            }
            log::debug!("Getting league details for {league_id}");
            current_league = self.sleeper_cli.get_league(league_id).await?;
            log::debug!("Getting draft details for {}", current_league.league_id);
            draft_futures.push(self.fetch_draft_picks_for_league(current_league.league_id));
        }
        for f in draft_futures {
            drafts.push(f.await?);
        }
        drafts.sort_by_key(|d| d.draft.season.to_string());

        Ok(drafts)
    }

    async fn fetch_draft_picks_for_league<T: Into<String>>(
        &self,
        league_id: T,
    ) -> Result<DraftResult, ECRError> {
        let league_id: String = league_id.into();
        let mut drafts: Vec<sleeper::Draft> = self
            .sleeper_cli
            .get_drafts_for_league(league_id.clone())
            .await?
            .into_iter()
            .filter(|d| d.status == "complete" && d.start_time.is_some() && d.last_picked.is_some())
            .collect();
        drafts.sort_by_key(|d| d.last_picked.unwrap());
        drafts.reverse();

        match drafts.len() {
            0 => Err(ECRError::NotFound(format!(
                "draft results for league: {league_id}",
            ))),
            1 => {
                let draft = drafts.remove(0);
                let picks = self.sleeper_cli.get_draft_picks(&draft.draft_id).await?;
                Ok(DraftResult { draft, picks })
            }
            _ => {
                let draft = drafts.remove(0);
                let picks = self.sleeper_cli.get_draft_picks(&draft.draft_id).await?;
                Ok(DraftResult { draft, picks })
            } //Err(ECRError::UndeterminableDraft(league_id)),
        }
    }

    pub fn fetch_league_metadata<T: Into<String>>(
        &self,
        league_id: T,
    ) -> Result<League, DataStoreError> {
        self.store.get_league(league_id.into()).inspect_err(|err| {
            log::warn!("can't find league for guild: {err}");
        })
    }

    pub async fn fetch_external_league_info<T: Into<ExternalId>>(
        &self,
        league_id: T,
    ) -> Result<sleeper::League, ECRError> {
        let league_id: ExternalId = league_id.into();
        match &league_id.platform {
            Some(platform) => match platform {
                // crate::store::Platform::Discord => todo!(),
                crate::store::Platform::Sleeper => {
                    Ok(self.sleeper_cli.get_league(league_id.id.as_str()).await?)
                }
                crate::store::Platform::Yahoo => todo!(),
                other => Err(ECRError::Unknown(format!(
                    "league_id platform not implemented: {other}"
                ))),
            },
            None => Err(ECRError::Unknown("league_id platform not set".into())),
        }
    }

    pub async fn fetch_external_league_owners<T: Into<ExternalId>>(
        &self,
        league_id: T,
    ) -> Result<Vec<sleeper::User>, ECRError> {
        let league_id: ExternalId = league_id.into();
        match &league_id.platform {
            Some(platform) => match platform {
                // crate::store::Platform::Discord => todo!(),
                crate::store::Platform::Sleeper => Ok(self
                    .sleeper_cli
                    .get_users_for_league(league_id.id.as_str())
                    .await?),
                // TODO: crate::store::Platform::Yahoo => todo!(),
                other => Err(ECRError::Unknown(format!(
                    "league_id platform not implemented: {other}"
                ))),
            },
            None => Err(ECRError::Unknown("league_id platform not set".into())),
        }
    }

    pub fn save_user_mapping<T: Into<String>, U: Into<String>>(
        &self,
        guild_id: T,
        user_discord_id: U,
        external_id: ExternalId,
    ) -> Result<(), DataStoreError> {
        self.store.set_league_user_mapping(
            guild_id.into(),
            user_discord_id.into().as_str(),
            external_id,
        )
    }

    pub fn save_league_mapping<T: Into<String>>(
        &self,
        guild_id: T,
        season: i32,
        external_id: ExternalId,
    ) -> Result<(), DataStoreError> {
        self.store
            .set_league_season_mapping(guild_id.into(), season, external_id)
    }

    pub async fn synchronize_manager_role(
        &self,
        guild_id: GuildId,
        http: &Http,
    ) -> Result<(), ECRError> {
        // grab league context
        let league: League = self.fetch_league_metadata(guild_id.to_string())?;
        let current_league_id = league
            .current_league_id()
            .ok_or(ECRError::Unknown("no current league found".to_string()))?;
        let current_platform: Platform = current_league_id.clone().platform.ok_or(
            ECRError::Unknown("current league id platform not set".to_string()),
        )?;
        let external_league_managers = self.fetch_external_league_owners(current_league_id).await?;

        // ensure manager role is setup
        let mut existing_roles = guild_id.roles(&http).await?;
        let manager_role = self
            .get_existing_or_create(
                guild_id,
                http,
                &mut existing_roles,
                "manager",
                0x7E8C54,
                Permissions::READ_MESSAGE_HISTORY |
                // Permissions::CHANGE_NICKNAME |
                Permissions::SEND_MESSAGES |
                Permissions::SEND_MESSAGES_IN_THREADS,
            )
            .await?;
        let mut non_bot_members = guild_id
            .members(http, Some(1000), None)
            .await?
            .into_iter()
            .filter(|m| !m.user.bot)
            .map(|m| (m.user.id, m))
            .collect::<HashMap<_, _>>();

        // check existing member mappings
        for user_mapping in league.member_map.iter() {
            let discord_user_id = UserId::new(user_mapping.0.parse().unwrap());
            // first check to see if member is a manager according to the external league
            let external_manager = external_league_managers.iter().find(|u| {
                user_mapping
                    .1
                    .get(&current_platform)
                    .map(|platform_user_id| platform_user_id == &u.user_id)
                    .unwrap_or(false)
            });

            // match to active discord member ref
            let member = non_bot_members.get_mut(&discord_user_id);
            if member.is_none() {
                // discord user is no longer in server/guild
                continue;
            }
            let member: &mut Member = member.unwrap();

            // check if manager role is applied
            if external_manager.is_some()
                && member.roles.iter().find(|r| *r == &manager_role).is_none()
            {
                // add manager role to member
                member.add_role(http, &manager_role).await?;
                log::info!(
                    "added {member} to managers",
                    member = member.user.display_name()
                )
            }

            // make sure that the usaer's nickname matches what their team display name is in the remote league
            if let Some(external_manager) = external_manager
                && member.display_name() != external_manager.display_name
            {
                let nickname = external_manager.display_name.as_str();
                // let's make a nickname
                match member
                    .edit(http, EditMember::new().nickname(nickname))
                    .await
                {
                    Ok(_) => {
                        log::info!(
                            "set {member}'s nickname to {nickname}",
                            member = member.user.display_name(),
                        );
                    }
                    Err(err) => {
                        let err_msg = format!(
                            "failed to set {member}'s nickname to {nickname}: {err}",
                            member = member.user.display_name(),
                        );
                        log::info!("{err_msg}");
                        // return Err(ECRError::Unknown(err_msg)); don't fail it
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn get_existing_or_create<C: Into<Colour>, R: Into<String>>(
        &self,
        guild_id: GuildId,
        http: &Http,
        existing_roles: &mut HashMap<RoleId, Role>,
        role_name: R,
        role_color: C,
        permissions: Permissions,
    ) -> Result<RoleId, ECRError> {
        let role_name = role_name.into();
        let role_id = if let Some(existing_role) =
            existing_roles.iter().find(|(_id, r)| r.name == role_name)
        {
            *existing_role.0
        } else {
            // create it
            let new_role = guild_id
                .create_role(
                    http,
                    EditRole::new()
                        .name(role_name)
                        .hoist(false)
                        .colour(role_color) // Grey Color
                        .permissions(permissions),
                )
                .await?;
            let k = new_role.id;
            existing_roles.insert(new_role.id, new_role);
            k
        };
        Ok(role_id)
    }
}

/// This function will look through provided sleeper league data structures and
/// find all player_ids referenced, place them into a HashSet and then reduce
/// the provided player_map to reduce memory footprint and such.
///
/// In my limited testing I was able to reduce the overall storage footprint of
/// a 5 year league history context from 14MB to 1MB on disk. This was
/// effectively a player_map size of 11388 reduced down to 476 entries.
fn reduce_players_map(
    players: &mut HashMap<String, sleeper::Player>,
    draft_results: Option<&Vec<DraftResult>>,
    rosters: Option<&Vec<sleeper::Roster>>,
    // transactions: Option<&Vec<Transaction>>,
) {
    let mut player_ids: HashSet<String> = HashSet::new();
    if let Some(draft_results) = draft_results {
        player_ids.extend(
            draft_results
                .iter()
                .flat_map(|d| d.picks.iter().map(|p| p.player_id.clone())),
        );
    }
    if let Some(rosters) = rosters {
        player_ids.extend(rosters.iter().flat_map(|r| r.players.clone()));
    }
    // if let Some(transactions) = transactions {
    //     player_ids.extend(transactions.iter().flat_map(|t| t.adds.keys().));
    // }

    // reduce the player map
    players.retain(|key, _| player_ids.contains(key));
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DraftResult {
    pub draft: sleeper::Draft,
    pub picks: Vec<sleeper::DraftPick>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeepablesContext {
    #[serde(with = "ts_milliseconds")]
    pub fetched_at: DateTime<Utc>,
    pub current_league: sleeper::League,
    pub draft_history: HashMap<String, DraftResult>,
    pub players: HashMap<String, sleeper::Player>,
    pub rosters: Vec<sleeper::Roster>,
    pub users: Vec<sleeper::User>,
    pub rankings: fantasypros::ECRResponse,
}

#[derive(thiserror::Error, Debug)]
pub enum ECRError {
    #[error("failed to fetch from sleeper {0}")]
    SleeperError(#[from] CachedClientError),

    #[error("Sleeper element not found {0}")]
    NotFound(String),

    #[error("error fetching from fantasypros: {0}")]
    FantasyProsError(#[from] fantasypros::FantasyProsError),

    #[error("error performing discord operation: {0}")]
    DiscordError(#[from] serenity::Error),

    #[error("error performing discord operation: {0}")]
    DatastoreError(#[from] store::DataStoreError),

    #[error("Unknown error {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::File;
    use std::io::{self, BufReader};
    use tokio::fs::{self};

    use crate::ecr::keepers::Keepable;
    use crate::ecr::service::{KeepablesContext, SPORT, Service};

    #[test]
    fn test_compute_keepables_report() {
        let keepables_context =
            load_keepables_context_testdata().expect("failed to load context test data file");

        assert_eq!(3, keepables_context.draft_history.len());
        assert_eq!(10, keepables_context.players.len());
        assert_eq!(4, keepables_context.rosters.len());
        assert_eq!(4, keepables_context.users.len());

        let keepables =
            Service::compute_keepables(&keepables_context).expect("report calculation failed");

        // Dana Dasher (9005) is on a roster but was never drafted, so she is not a keepable
        assert_eq!(
            4,
            keepables.keepables.len(),
            "expecting 4 keepables in the league"
        );
        let find = |owner: &str, player: &str| -> &Keepable {
            keepables
                .keepables
                .iter()
                .find(|k| k.owner_name == owner && k.player_name == player)
                .unwrap_or_else(|| panic!("{player} should be a keepable for {owner}"))
        };
        assert_eq!(
            3,
            keepables
                .keepables
                .iter()
                .filter(|k| k.owner_name == "SampleOwner")
                .count()
        );

        assert_eq!(
            Some(String::from("ADP")),
            find("SampleOwner", "Casey Catcher").cost("2025"),
            "kept three straight seasons: contract is renegotiated at ADP"
        );
        assert_eq!(
            Some(String::from("2")),
            find("SampleOwner", "Nico Nimble").cost("2025"),
            "drafted in the second round last year"
        );
        assert_eq!(
            Some(String::from("10")),
            find("SampleOwner", "Will Wheeler").cost("2025"),
            "drafted in the tenth round last year"
        );
        assert_eq!(
            Some(String::from("4")),
            find("ManagerBeta", "Rex Runner").cost("2025"),
            "first kept last year: costs the round he was kept in"
        );
        // ECR rank 1 in a 4 team league => round 1, pick 2
        assert_eq!(
            Some(String::from("1.2")),
            find("SampleOwner", "Casey Catcher").player_rank
        );
    }

    fn load_keepables_context_testdata() -> Result<KeepablesContext, io::Error> {
        let file = File::open("testdata/ecr/keepables_ctx.json")?;
        serde_json::from_reader(BufReader::new(file)).map_err(|json_error| json_error.into())
    }

    #[allow(dead_code)]
    async fn fetch_and_save_ctx<T: Into<String>>(
        svc: &Service,
        league_id: &str,
        outfile: T,
    ) -> Result<KeepablesContext, Box<dyn Error>> {
        let keepables_context: KeepablesContext =
            svc.fetch_keepable_context(league_id, &SPORT).await?;
        let outfile = outfile.into();
        fs::write(outfile, serde_json::to_vec(&keepables_context)?).await?;
        Ok(keepables_context)
    }

    // --- gathering the data from (mock) Sleeper and FantasyPros servers -------------------------

    use super::{DataStore, ECRError};
    use crate::fantasypros;
    use fantasy_sports_sleeper as sleeper;
    use std::sync::Arc;
    use tempfile::TempDir;
    use wiremock::{
        Mock, MockServer, Request, Respond, ResponseTemplate,
        matchers::{method, path_regex},
    };

    /// the 2019 league recorded in the sleeper crate's test data, which plays the previous season
    const PREVIOUS: &str = "164974877262287888";
    /// a made up league that follows it
    const CURRENT: &str = "2000000000000000009";
    const DRAFT: &str = "685822547406541191";

    fn sleeper_data(rel: &str) -> String {
        std::fs::read_to_string(format!(
            "{}/../../crates/sleeper/testdata/{rel}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap()
    }

    struct Sleeper {
        /// whether the current league links back to the recorded one
        has_previous_season: bool,
    }
    impl Respond for Sleeper {
        fn respond(&self, req: &Request) -> ResponseTemplate {
            let json =
                |body: String| ResponseTemplate::new(200).set_body_raw(body, "application/json");
            match req.url.path() {
                p if p == format!("/league/{CURRENT}") => {
                    let mut league: serde_json::Value =
                        serde_json::from_str(&sleeper_data(&format!("v1/league/{PREVIOUS}.json")))
                            .unwrap();
                    league["league_id"] = CURRENT.into();
                    league["season"] = "2020".into();
                    league["previous_league_id"] = if self.has_previous_season {
                        PREVIOUS.into()
                    } else {
                        "0".into()
                    };
                    json(league.to_string())
                }
                p if p == format!("/league/{CURRENT}/rosters") => {
                    json(sleeper_data(&format!("v1/league/{PREVIOUS}/rosters.json")))
                }
                p if p == format!("/league/{CURRENT}/users") => {
                    json(sleeper_data(&format!("v1/league/{PREVIOUS}/users.json")))
                }
                p if p == format!("/league/{PREVIOUS}") => {
                    json(sleeper_data(&format!("v1/league/{PREVIOUS}.json")))
                }
                p if p == format!("/league/{PREVIOUS}/drafts") => {
                    json(sleeper_data(&format!("v1/league/{PREVIOUS}/drafts.json")))
                }
                p if p == format!("/draft/{DRAFT}/picks") => {
                    json(sleeper_data(&format!("v1/draft/{DRAFT}/picks.json")))
                }
                "/players/nfl" => ResponseTemplate::new(200)
                    .set_body_raw(
                        std::fs::read(format!(
                            "{}/../../crates/sleeper/testdata/players_nfl.json.gz",
                            env!("CARGO_MANIFEST_DIR")
                        ))
                        .unwrap(),
                        "application/json",
                    )
                    .insert_header("Content-Encoding", "gzip"),
                _ => ResponseTemplate::new(404),
            }
        }
    }

    async fn service(has_previous_season: bool) -> (Service, MockServer, MockServer, TempDir) {
        let sleeper_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(".*"))
            .respond_with(Sleeper {
                has_previous_season,
            })
            .mount(&sleeper_server)
            .await;
        let pros_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex("half-point-ppr-cheatsheets"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(
                        std::fs::read(format!(
                            "{}/testdata/fantasypros/synthetic_rankings_half_ppr.html.gz",
                            env!("CARGO_MANIFEST_DIR")
                        ))
                        .unwrap(),
                    )
                    .insert_header("content-encoding", "gzip")
                    .insert_header("content-type", "text/html; charset=UTF-8"),
            )
            .mount(&pros_server)
            .await;
        let dir = TempDir::new().unwrap();
        let store = Arc::new(DataStore::open(dir.path()).unwrap());
        let service = Service::with_clients(
            sleeper::Client::new().with_base_url(sleeper_server.uri()),
            fantasypros::Client::new_with_base_url(pros_server.uri()),
            &store,
        );
        (service, sleeper_server, pros_server, dir)
    }

    #[tokio::test]
    async fn the_keepable_context_gathers_the_league_its_history_and_the_rankings() {
        let (service, _sleeper, _pros, _dir) = service(true).await;

        let ctx = service
            .fetch_keepable_context(CURRENT, &SPORT)
            .await
            .unwrap();

        assert_eq!("2020", ctx.current_league.season);
        assert_eq!(12, ctx.rosters.len());
        assert_eq!(12, ctx.users.len());
        assert_eq!(10, ctx.rankings.players.len(), "the FantasyPros page");
        // the history is the previous seasons' drafts, keyed by season; the second recorded
        // draft never started, so only the real one is used
        assert_eq!(vec!["2019"], ctx.draft_history.keys().collect::<Vec<_>>());
        let draft = &ctx.draft_history["2019"];
        assert_eq!(DRAFT, draft.draft.draft_id);
        assert_eq!(204, draft.picks.len());
        // only the players that matter to the league are kept, not the whole player universe
        assert!(ctx.players.len() < 1000, "{}", ctx.players.len());
        assert!(
            ctx.rosters
                .iter()
                .flat_map(|r| &r.players)
                .all(|id| ctx.players.contains_key(id))
        );
    }

    #[tokio::test]
    async fn keepables_are_the_players_drafted_last_season_that_are_still_on_a_roster() {
        let (service, _sleeper, _pros, _dir) = service(true).await;
        let ctx = service
            .fetch_keepable_context(CURRENT, &SPORT)
            .await
            .unwrap();

        let report = service.get_keepables(&ctx).unwrap();

        assert_eq!("2020", report.season);
        assert!(!report.keepables.is_empty());
        for keepable in &report.keepables {
            // everyone in the report was drafted in 2019, so the cost is a draft round
            let cost = keepable.cost("2020").expect("a drafted player has a cost");
            assert!(
                cost.parse::<u8>().is_ok() || cost == "ADP",
                "{} costs {cost}",
                keepable.player_name
            );
        }
        assert!(report.text_report().contains("COST="));
    }

    #[tokio::test]
    async fn a_league_without_a_previous_season_has_no_keepables() {
        let (service, _sleeper, _pros, _dir) = service(false).await;
        let ctx = service
            .fetch_keepable_context(CURRENT, &SPORT)
            .await
            .unwrap();
        assert!(ctx.draft_history.is_empty());

        let err = service.get_keepables(&ctx).unwrap_err();

        assert!(matches!(err, ECRError::NotFound(_)), "{err}");
        assert!(err.to_string().contains("draft_history for 2019"), "{err}");
    }

    #[tokio::test]
    async fn an_unknown_league_is_an_error() {
        let (service, _sleeper, _pros, _dir) = service(true).await;
        assert!(service.fetch_keepable_context("999", &SPORT).await.is_err());
    }
}
