use fantasy_sports_core as core;

pub enum Event {
    LeagueRefreshComplete(core::League),
    Error(String),
}

pub enum BackendAction {
    RefreshLeague(core::ExternalId),
    FetchAllLeagues,
}
