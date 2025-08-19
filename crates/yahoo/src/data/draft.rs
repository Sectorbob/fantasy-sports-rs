use serde::{Deserialize, Deserializer};

use crate::data::{player::PlayerKey, team::TeamKey};

#[derive(Deserialize, Debug, Clone)]
struct DraftResults {
    #[serde(rename = "draft_result")]
    pub draft_results: Vec<DraftResult>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DraftResult {
    pub pick: u32,
    pub round: u32,
    pub team_key: TeamKey,
    pub player_key: Option<PlayerKey>, // e.g. nfl.p.7241
}
impl DraftResult {
    // This method is called when the deserializer encounters a newtype struct
    pub(crate) fn deserialize_draft_results<'de, D>(
        deserializer: D,
    ) -> Result<Vec<DraftResult>, D::Error>
    where
        D: Deserializer<'de>,
    {
        DraftResults::deserialize(deserializer).map(|r| r.draft_results)
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DraftStatus {
    PreDraft,
    PostDraft,
    // TODO: Handle fallback case with value
    // #[serde(untagged)]
    // Other(String),
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::{FantasyContent, data::FantasyContentWrapper};

    #[test]
    fn test_deserilaize_draft_results() {
        let xml_file_reader = BufReader::new(
            File::open("tests/data/fantasy/v2/league/380.l.129397/draftresults.xml")
                .expect("failed to open test data file"),
        );
        let wrapper: FantasyContentWrapper =
            quick_xml::de::from_reader(xml_file_reader).expect("failed to parse test xml file");

        let draft_results = if let FantasyContent::League(league) = wrapper.content {
            league.draft_results
        } else {
            debug_assert!(false, "wrong type");
            return;
        };

        assert_eq!(204, draft_results.len());
    }
}
