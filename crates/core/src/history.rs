use std::{collections::HashSet, ops::IndexMut};

use crate::{Standings, StandingsEntry};

pub fn build_owner_records(
    leagues_standings: &Vec<Standings>,
    owner_id_mapping: &Vec<(&str, Vec<&str>)>,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(owner_id: &str, owner_name: &str, wins: u64, losses: u64) -> StandingsEntry {
        StandingsEntry {
            roster_id: "1".into(),
            team_name: format!("Team {owner_name}"),
            owner_name: owner_name.into(),
            owner_id: owner_id.into(),
            wins,
            losses,
            ties: 0,
        }
    }

    #[test]
    fn records_of_one_person_with_several_ids_are_merged() {
        let seasons = vec![
            Standings { entries: vec![entry("sleeper-id", "sam", 10, 4), entry("other", "alex", 3, 11)] },
            Standings { entries: vec![entry("yahoo-guid", "Sam S.", 8, 6)] },
        ];
        let mapping = vec![("Sam Sample", vec!["sleeper-id", "yahoo-guid"])];

        let records = build_owner_records(&seasons, &mapping);

        assert_eq!(2, records.len());
        // sorted by wins, descending
        assert_eq!("Sam Sample", records[0].owner_name);
        assert_eq!((18, 10), (records[0].wins, records[0].losses));
        assert_eq!("alex", records[1].owner_name);
    }

    #[test]
    fn unmapped_owners_keep_their_own_records() {
        let seasons = vec![Standings { entries: vec![entry("a", "alex", 5, 9)] }];
        let records = build_owner_records(&seasons, &vec![]);
        assert_eq!(1, records.len());
        assert_eq!("a", records[0].owner_id);
    }

    #[test]
    fn no_standings_means_no_records() {
        assert!(build_owner_records(&vec![], &vec![("Nobody", vec!["x"])]).is_empty());
    }
}
