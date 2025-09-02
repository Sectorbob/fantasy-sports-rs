use std::io::{self, Stdout};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Constraint,
    style::Style,
    terminal::CompletedFrame,
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

use fantasy_core_rs::{self as core};

use crate::constants;

pub struct LeagueHistoryOwnerRecordsPage {
    title_override: Option<String>,
    owner_records: Vec<core::StandingsEntry>,
}
impl LeagueHistoryOwnerRecordsPage {
    pub fn new() -> Self {
        LeagueHistoryOwnerRecordsPage {
            title_override: None,
            owner_records: vec![],
        }
    }
    pub fn from_leagues(leagues: &Vec<core::League>) -> Self {
        let ecr_leagues = leagues
            .iter()
            .filter(|l| l.name() == "Rapid League 17")
            .collect::<Vec<_>>();
        let leagues_standings = ecr_leagues
            .iter()
            .map(|l| l.standings())
            .collect::<Vec<_>>();
        let owner_id_mapping = vec![
            (
                "Kyle Heide",
                vec!["GGQ52IUUT34FUMAXVLVIZ6V2QP", "727598467304811923"],
            ),
            (
                "ManagerGolden74",
                vec!["REIST2RMJAWUUGJITSTEBVQVXI", "484883887154522157"],
            ),
            (
                "ManagerGolden89",
                vec!["ENEDP3KTK5BRSZV2IUIMX33DJN", "309251795699782577"],
            ),
            (
                "ManagerRapid69",
                vec!["OBHLQUILSDMZP5V2VN43JWFM6D", "314885799278160790"],
            ),
            (
                "ManagerStormy75",
                vec!["SQMZ2IRSENZ62CKW2N2WEPHVI4", "774067826420676378"],
            ),
            (
                "ManagerGolden82",
                vec!["X7WQ2EOSE73UD27GVPUEJWAQ7G", "529096176700349825"],
            ),
            ("ManagerWild47", vec!["592316633898074561"]),
            ("ManagerMighty50", vec!["109948207775581082", "--"]),
            (
                "ManagerMighty19",
                vec!["YKTFV3WMRHTM6CUQ75IWHKRVMC", "193976103221158331"],
            ),
            (
                "ManagerBrave87",
                vec![
                    "ZFV3OZMWADA67PA45EBPDHWBBR",
                    "252139931714386691",
                    "749786951287232737",
                ], //David has 2 sleeper accounts???
            ),
            (
                "ManagerWild06",
                vec!["MSXOLV3TPG4YWIRHLTTLH7CTKR", "669649552185855627"],
            ),
            (
                "ManagerIron05",
                vec!["Q6M3SMGVOHFXMOBJLGR5ZKXKDL", "355028255152716972"],
            ),
            ("ManagerIron36", vec!["891257881093878073"]),
            ("ManagerIron21", vec!["5641843776073125157"]),
            ("ManagerCrimson51", vec!["789478417584731319"]),
            ("ManagerStormy63", vec!["ELMPJBYWZGDEE3NOVWLVNE6EPP"]),
            ("ManagerRapid88", vec!["AQGCI2KPCEFMP5D6IEJ32UHGCG"]),
            ("ManagerSilent14", vec!["1511255259875654879"]),
            ("ManagerStormy98", vec!["433867313387100781"]),
            ("ManagerBrave42", vec!["744482347784160106"]),
            ("ManagerCosmic96", vec!["FNQWFKXSGS6CQ5C4TLSHGYOZBP"]),
            ("ManagerSilent03", vec!["641365149273056286"]),
            ("ManagerSwift83", vec!["158638755124059567"]),
        ];
        let owner_records =
            core::history::build_owner_records(&leagues_standings, &owner_id_mapping);
        LeagueHistoryOwnerRecordsPage {
            title_override: None,
            owner_records,
        }
    }

    pub fn draw<'a>(
        &'a mut self,
        terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    ) -> io::Result<CompletedFrame<'a>> {
        terminal.draw(|f| {
            let size = f.size();
            let title: String = if let Some(title_override) = self.title_override.clone() {
                title_override.trim().to_string()
            } else {
                String::from("League History Owner Records")
            };

            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(constants::COLOR_BORDER))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(constants::COLOR_BACKGROUND));

            let mut rows = vec![];
            for (_, e) in self.owner_records.iter().enumerate() {
                // let standings = &self.leagues_standings[i];
                rows.push(Row::new(vec![
                    // Cell::from("Rapid League 17"),
                    Cell::from(e.team_name.clone()),
                    Cell::from(e.roster_id.clone()),
                    Cell::from(e.owner_name.clone()),
                    // Cell::from(e.owner_id.clone()),
                    Cell::from(e.wins.to_string()),
                    Cell::from(e.losses.to_string()),
                    Cell::from(e.ties.to_string()),
                ]));
            }

            let table = Table::new(rows)
                .widths(&[
                    // Constraint::Percentage(20),
                    Constraint::Percentage(30),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    // Constraint::Percentage(20),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                ])
                .header(Row::new(vec![
                    // "League Name",
                    "Team Name",
                    "Roster",
                    "Owner Name",
                    // "OwnerId",
                    "W",
                    "L",
                    "T",
                ]));

            f.render_widget(table, block.inner(size));
            f.render_widget(block.clone(), size);
        })
    }
}
