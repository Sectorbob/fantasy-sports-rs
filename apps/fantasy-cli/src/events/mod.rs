use fantasy_sports_core as core;

#[allow(clippy::large_enum_variant)] // one variant holds a whole league/settings payload; boxing it would complicate the public API
pub enum Event {
    LeagueRefreshComplete(core::League),
    Error(String),
}

pub enum BackendAction {
    RefreshLeague(core::ExternalId),
    FetchAllLeagues,
}
