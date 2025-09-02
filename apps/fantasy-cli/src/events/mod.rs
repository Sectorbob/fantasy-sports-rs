use fantasy_core_rs as core;

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
