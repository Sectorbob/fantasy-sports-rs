use fantasy_sports_core as core;

pub enum Event {
    // Input(char),
    Tick,
    LeagueRefreshComplete(core::League),
    Error(String),
}

pub enum BackendAction {
    Quit,
    RefreshLeague(core::ExternalId),
    FetchAllLeagues,
}
