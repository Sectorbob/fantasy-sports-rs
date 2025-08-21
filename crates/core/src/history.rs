use std::{collections::HashSet, ops::IndexMut};

use crate::{Standings, StandingsEntry};

pub fn build_owner_records(
    leagues_standings: &Vec<Standings>,
    owner_id_mapping: &Vec<(&'static str, Vec<&'static str>)>,
) -> Vec<StandingsEntry> {
    let owner_ids = leagues_standings
        .iter()
        .flat_map(|l| l.entries.iter().map(|e| e.owner_id.clone()))
        .collect::<HashSet<_>>();

    // initial pass
    let owner_records = owner_ids
        .iter()
        .map(|owner_id| {
            let entries = leagues_standings
                .iter()
                .filter_map(|s| s.entries.iter().find(|e| &e.owner_id == owner_id))
                .collect::<Vec<_>>();
            StandingsEntry {
                roster_id: entries
                    .first()
                    .map_or(String::new(), |e| e.roster_id.clone()),
                team_name: entries
                    .first()
                    .map_or(String::new(), |e| e.team_name.clone()),
                owner_name: entries
                    .first()
                    .map_or(String::new(), |e| e.owner_name.clone()),
                owner_id: owner_id.to_string(),
                wins: entries.iter().map(|e| e.wins).sum(),
                losses: entries.iter().map(|e| e.losses).sum(),
                ties: entries.iter().map(|e| e.ties).sum(),
            }
        })
        .collect::<Vec<_>>();

    // now try to ajoin mapped users
    // try to do a lookup by id for known owner mapping

    let mut new_owner_records: Vec<StandingsEntry> = vec![];

    owner_records.into_iter().for_each(|e| {
        let m = owner_id_mapping
            .iter()
            .find(|(_, external_ids)| external_ids.contains(&e.owner_id.as_str()));
        if let Some(f) = m {
            if let Some(existing_index) = new_owner_records.iter().position(|r| r.owner_name == f.0)
            {
                let existing: &mut StandingsEntry = new_owner_records.index_mut(existing_index);
                existing.wins = existing.wins + e.wins;
                existing.losses = existing.losses + e.losses;
                existing.ties = existing.ties + e.ties;
            } else {
                new_owner_records.push(StandingsEntry {
                    roster_id: e.roster_id,
                    team_name: e.team_name,
                    owner_name: f.0.to_string(),
                    owner_id: String::new(),
                    wins: e.wins,
                    losses: e.losses,
                    ties: e.ties,
                });
            }
        } else {
            new_owner_records.push(e);
        }
    });

    new_owner_records.sort_by_key(|e| e.wins);
    new_owner_records.reverse();
    new_owner_records
}
