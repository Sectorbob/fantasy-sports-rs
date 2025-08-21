use sleeper_fantasy_rs as sleeper;
use yahoo_fantasy_rs as yahoo;

#[derive(Debug, PartialEq)]
pub enum ScoringSettings {
    Football {
        // Passing Stats
        pass_2pt: f32,
        pass_int: f32,
        pass_yd: f32,
        pass_td: f32,

        // Receiving Stats
        receptions: f32,
        rec_2pt: f32,
        rec_td: f32,
        rec_yd: f32,

        // Rushing Stats
        rush_2pt: f32,
        rush_td: f32,
        rush_yd: f32,

        // Kicking Stats
        kick_fgmiss: f32,
        kick_fgm_0_19: f32,
        kick_fgm_20_29: f32,
        kick_fgm_30_39: f32,
        kick_fgm_40_49: f32,
        kick_fgm_50p: f32,
        kick_xpm: f32,
        kick_xpmiss: f32,

        // Other Offensive Stats
        fum: f32,
        fum_lost: f32,

        // DST Stats
        pts_allow_0: f32,
        pts_allow_1_6: f32,
        pts_allow_7_13: f32,
        pts_allow_14_20: f32,
        pts_allow_21_27: f32,
        pts_allow_28_34: f32,
        pts_allow_35p: f32,
        int: f32,
        sack: f32,
        safe: f32,
        def_td: f32,
        def_kr_td: f32,
        fum_rec: f32,
        fum_rec_td: f32,
        fum_forced: f32,
        pass_int_td: f32,
        def_st_td: f32,
        def_st_fum_rec: f32,
        def_st_ff: f32,
        st_fum_rec: f32,
        st_ff: f32,
        st_td: f32,
        blk_kick: f32,
        kr_td: f32,
        def_pr_td: f32,
        pr_td: f32,

        // Milestones (cumulative stats)
        bonus_rec_yd_200: f32,
        bonus_rush_yd_200: f32,
        bonus_pass_yd_400: f32,

        // Bonus Stats
        bonus_rec_te: f32,       // TODO: What is this?
        bonus_pass_td_40p: f32,  // points per pass_td of 40+ yards
        bonus_rush_td_40p: f32,  // points per rush_td of 40+ yards
        bonus_pass_cmp_40p: f32, // points per pass_cmp of 40+ yards
        bonus_rec_td_40p: f32,   // points per rec_td of 40+ yards
        bonus_rush_40p: f32,     // points per rush of 40+ yards
        bonus_rec_40p: f32,      // points per reception of 40+ yards
    },
    #[allow(dead_code)]
    Unknown,
}
impl ScoringSettings {
    pub fn formatted_settings(&self) -> Vec<(&'static str, String)> {
        match self {
            ScoringSettings::Football {
                pass_2pt,
                pass_int,
                pass_yd,
                pass_td,
                receptions,
                rec_2pt,
                rec_td,
                rec_yd,
                rush_2pt,
                rush_td,
                rush_yd,
                kick_fgmiss,
                kick_fgm_0_19,
                kick_fgm_20_29,
                kick_fgm_30_39,
                kick_fgm_40_49,
                kick_fgm_50p,
                kick_xpm,
                kick_xpmiss,
                fum,
                fum_lost,
                pts_allow_0,
                pts_allow_1_6,
                pts_allow_7_13,
                pts_allow_14_20,
                pts_allow_21_27,
                pts_allow_28_34,
                pts_allow_35p,
                int,
                sack,
                safe,
                def_td,
                def_kr_td,
                fum_rec,
                fum_rec_td,
                fum_forced,
                pass_int_td,
                def_st_td,
                def_st_fum_rec,
                def_st_ff,
                st_fum_rec,
                st_ff,
                st_td,
                blk_kick,
                kr_td,
                def_pr_td,
                pr_td,
                bonus_rec_yd_200,
                bonus_rush_yd_200,
                bonus_pass_yd_400,
                bonus_rec_te,
                bonus_pass_td_40p,
                bonus_rush_td_40p,
                bonus_pass_cmp_40p,
                bonus_rec_td_40p,
                bonus_rush_40p,
                bonus_rec_40p,
            } => [
                ("pass_2pt", pass_2pt),
                ("pass_int", pass_int),
                ("pass_yd", pass_yd),
                ("pass_td", pass_td),
                ("receptions", receptions),
                ("rec_2pt", rec_2pt),
                ("rec_td", rec_td),
                ("rec_yd", rec_yd),
                ("rush_2pt", rush_2pt),
                ("rush_td", rush_td),
                ("rush_yd", rush_yd),
                ("kick_fgmiss", kick_fgmiss),
                ("kick_fgm_0_19", kick_fgm_0_19),
                ("kick_fgm_20_29", kick_fgm_20_29),
                ("kick_fgm_30_39", kick_fgm_30_39),
                ("kick_fgm_40_49", kick_fgm_40_49),
                ("kick_fgm_50p", kick_fgm_50p),
                ("kick_xpm", kick_xpm),
                ("kick_xpmiss", kick_xpmiss),
                ("fum", fum),
                ("fum_lost", fum_lost),
                ("pts_allow_0", pts_allow_0),
                ("pts_allow_1_6", pts_allow_1_6),
                ("pts_allow_7_13", pts_allow_7_13),
                ("pts_allow_14_20", pts_allow_14_20),
                ("pts_allow_21_27", pts_allow_21_27),
                ("pts_allow_28_34", pts_allow_28_34),
                ("pts_allow_35p", pts_allow_35p),
                ("int", int),
                ("sack", sack),
                ("safe", safe),
                ("def_td", def_td),
                ("def_kr_td", def_kr_td),
                ("fum_rec", fum_rec),
                ("fum_rec_td", fum_rec_td),
                ("fum_forced", fum_forced),
                ("pass_int_td", pass_int_td),
                ("def_st_td", def_st_td),
                ("def_st_fum_rec", def_st_fum_rec),
                ("def_st_ff", def_st_ff),
                ("st_fum_rec", st_fum_rec),
                ("st_ff", st_ff),
                ("st_td", st_td),
                ("blk_kick", blk_kick),
                ("kr_td", kr_td),
                ("def_pr_td", def_pr_td),
                ("pr_td", pr_td),
                ("bonus_rec_yd_200", bonus_rec_yd_200),
                ("bonus_rush_yd_200", bonus_rush_yd_200),
                ("bonus_pass_yd_400", bonus_pass_yd_400),
                ("bonus_rec_te", bonus_rec_te),
                ("bonus_pass_td_40p", bonus_pass_td_40p),
                ("bonus_rush_td_40p", bonus_rush_td_40p),
                ("bonus_pass_cmp_40p", bonus_pass_cmp_40p),
                ("bonus_rec_td_40p", bonus_rec_td_40p),
                ("bonus_rush_40p", bonus_rush_40p),
                ("bonus_rec_40p", bonus_rec_40p),
            ]
            .into_iter()
            .filter_map(|(k, v)| match *v != (0.0 as f32) {
                true => Some((k, format!("{:.2}", v))),
                false => None,
            })
            .collect(),
            ScoringSettings::Unknown => vec![],
        }
    }
}
impl From<&sleeper::ScoringSettings> for ScoringSettings {
    fn from(value: &sleeper::ScoringSettings) -> Self {
        {
            match value {
                sleeper_fantasy_rs::ScoringSettings::Football {
                    pass_2pt,
                    pass_int,
                    pass_yd,
                    pass_td,
                    rec,
                    rec_2pt,
                    rec_td,
                    rec_yd,
                    rush_2pt,
                    rush_td,
                    rush_yd,
                    fgmiss,
                    fgm_0_19,
                    fgm_20_29,
                    fgm_30_39,
                    fgm_40_49,
                    fgm_50p,
                    xpm,
                    xpmiss,
                    fum,
                    fum_lost,
                    pts_allow_0,
                    pts_allow_1_6,
                    pts_allow_7_13,
                    pts_allow_14_20,
                    pts_allow_21_27,
                    pts_allow_28_34,
                    pts_allow_35p,
                    int,
                    sack,
                    safe,
                    def_td,
                    def_kr_td,
                    fum_rec,
                    fum_rec_td,
                    ff,
                    pass_int_td,
                    def_st_td,
                    def_st_fum_rec,
                    def_st_ff,
                    st_fum_rec,
                    st_ff,
                    st_td,
                    blk_kick,
                    kr_td,
                    def_pr_td,
                    pr_td,
                    bonus_rec_yd_200,
                    pass_td_40p,
                    bonus_rush_yd_200,
                    bonus_pass_yd_400,
                    rush_td_40p,
                    bonus_rec_te,
                    pass_cmp_40p,
                    rec_td_40p,
                    rush_40p,
                    rec_40p,
                    other: _,
                } => ScoringSettings::Football {
                    pass_2pt: pass_2pt.clone(),
                    pass_int: pass_int.clone(),
                    pass_yd: pass_yd.clone(),
                    pass_td: pass_td.clone(),
                    receptions: rec.clone(),
                    rec_2pt: rec_2pt.clone(),
                    rec_td: rec_td.clone(),
                    rec_yd: rec_yd.clone(),
                    rush_2pt: rush_2pt.clone(),
                    rush_td: rush_td.clone(),
                    rush_yd: rush_yd.clone(),
                    kick_fgmiss: fgmiss.clone(),
                    kick_fgm_0_19: fgm_0_19.clone(),
                    kick_fgm_20_29: fgm_20_29.clone(),
                    kick_fgm_30_39: fgm_30_39.clone(),
                    kick_fgm_40_49: fgm_40_49.clone(),
                    kick_fgm_50p: fgm_50p.clone(),
                    kick_xpm: xpm.clone(),
                    kick_xpmiss: xpmiss.clone(),
                    fum: fum.clone(),
                    fum_lost: fum_lost.clone(),
                    pts_allow_0: pts_allow_0.clone(),
                    pts_allow_1_6: pts_allow_1_6.clone(),
                    pts_allow_7_13: pts_allow_7_13.clone(),
                    pts_allow_14_20: pts_allow_14_20.clone(),
                    pts_allow_21_27: pts_allow_21_27.clone(),
                    pts_allow_28_34: pts_allow_28_34.clone(),
                    pts_allow_35p: pts_allow_35p.clone(),
                    int: int.clone(),
                    sack: sack.clone(),
                    safe: safe.clone(),
                    def_td: def_td.clone(),
                    def_kr_td: def_kr_td.clone(),
                    fum_rec: fum_rec.clone(),
                    fum_rec_td: fum_rec_td.clone(),
                    fum_forced: ff.clone(),
                    pass_int_td: pass_int_td.clone(),
                    def_st_td: def_st_td.clone(),
                    def_st_fum_rec: def_st_fum_rec.clone(),
                    def_st_ff: def_st_ff.clone(),
                    st_fum_rec: st_fum_rec.clone(),
                    st_ff: st_ff.clone(),
                    st_td: st_td.clone(),
                    blk_kick: blk_kick.clone(),
                    kr_td: kr_td.clone(),
                    def_pr_td: def_pr_td.clone(),
                    pr_td: pr_td.clone(),
                    bonus_rec_yd_200: bonus_rec_yd_200.clone(),
                    bonus_rush_yd_200: bonus_rush_yd_200.clone(),
                    bonus_pass_yd_400: bonus_pass_yd_400.clone(),
                    bonus_rec_te: bonus_rec_te.clone(),
                    bonus_pass_td_40p: pass_td_40p.clone(),
                    bonus_rush_td_40p: rush_td_40p.clone(),
                    bonus_pass_cmp_40p: pass_cmp_40p.clone(),
                    bonus_rec_td_40p: rec_td_40p.clone(),
                    bonus_rush_40p: rush_40p.clone(),
                    bonus_rec_40p: rec_40p.clone(),
                },
                sleeper_fantasy_rs::ScoringSettings::Basketball {
                    pts: _,
                    ast: _,
                    reb: _,
                    fgmi: _,
                    ftmi: _,
                    tpm: _,
                    stl: _,
                    blk: _,
                    tf: _,
                    dd: _,
                    td: _,
                    ff: _,
                    to: _,
                    bonus_ast_15p: _,
                    bonus_pt_40p: _,
                    bonus_pt_50p: _,
                    bonus_reb_20p: _,
                    other: _,
                } => todo!(),
            }
        }
    }
}
impl From<sleeper::ScoringSettings> for ScoringSettings {
    fn from(value: sleeper::ScoringSettings) -> Self {
        ScoringSettings::from(&value)
    }
}
impl From<&yahoo::Settings> for ScoringSettings {
    fn from(value: &yahoo::Settings) -> Self {
        let mut pass_2pt = 0.0;
        let mut pass_int = 0.0;
        let mut pass_yd = 0.0;
        let mut pass_td = 0.0;
        let mut receptions = 0.0;
        let mut rec_2pt = 0.0;
        let mut rec_td = 0.0;
        let mut rec_yd = 0.0;
        let mut rush_2pt = 0.0;
        let mut rush_td = 0.0;
        let mut rush_yd = 0.0;
        let kick_fgmiss = 0.0;
        let mut kick_fgm_0_19 = 0.0;
        let mut kick_fgm_20_29 = 0.0;
        let mut kick_fgm_30_39 = 0.0;
        let mut kick_fgm_40_49 = 0.0;
        let mut kick_fgm_50p = 0.0;
        let mut kick_xpm = 0.0;
        let mut kick_xpmiss = 0.0;
        let fum = 0.0;
        let mut fum_lost = 0.0;
        let mut pts_allow_0 = 0.0;
        let mut pts_allow_1_6 = 0.0;
        let mut pts_allow_7_13 = 0.0;
        let mut pts_allow_14_20 = 0.0;
        let mut pts_allow_21_27 = 0.0;
        let mut pts_allow_28_34 = 0.0;
        let mut pts_allow_35p = 0.0;
        let mut int = 0.0;
        let mut sack = 0.0;
        let mut safe = 0.0;
        let mut def_td = 0.0;
        let def_kr_td = 0.0;
        let mut fum_rec = 0.0;
        let fum_rec_td = 0.0;
        let fum_forced = 0.0;
        let pass_int_td = 0.0;
        let def_st_td = 0.0;
        let def_st_fum_rec = 0.0;
        let def_st_ff = 0.0;
        let st_fum_rec = 0.0;
        let st_ff = 0.0;
        let mut st_td = 0.0;
        let mut blk_kick = 0.0;
        let mut kr_td = 0.0;
        let def_pr_td = 0.0;
        let mut pr_td = 0.0;
        let bonus_rec_yd_200 = 0.0;
        let bonus_rush_yd_200 = 0.0;
        let bonus_pass_yd_400 = 0.0;
        let bonus_rec_te = 0.0;
        let bonus_pass_td_40p = 0.0;
        let bonus_rush_td_40p = 0.0;
        let bonus_pass_cmp_40p = 0.0;
        let bonus_rec_td_40p = 0.0;
        let bonus_rush_40p = 0.0;
        let bonus_rec_40p = 0.0;
        for m in &value.stat_modifiers {
            match m.stat_id {
                4 => pass_yd = m.value,  // Passing Yards
                5 => pass_td = m.value,  // Passing Touchdowns
                6 => pass_int = m.value, // Interceptions (passing)
                8 => {
                    // Rushing Attempts
                    // TODO: implement points per rust=hing attempt
                    // rush_att = m.value;
                }
                9 => rush_yd = m.value,     // Rushing Yards
                10 => rush_td = m.value,    // Rushing Touchdowns
                11 => receptions = m.value, // Receptions
                12 => rec_yd = m.value,     // Receiving Yards
                13 => rec_td = m.value,     // Receiving Touchdowns
                15 => {
                    // Return Touchdowns
                    kr_td = m.value;
                    pr_td = m.value;
                }
                16 => {
                    // 2-Point Conversions
                    rush_2pt = m.value;
                    rec_2pt = m.value;
                    pass_2pt = m.value;
                }
                18 => fum_lost = m.value,       // Fumbles Lost
                19 => kick_fgm_0_19 = m.value,  // Field Goals 0-19 Yards
                20 => kick_fgm_20_29 = m.value, // Field Goals 20-29 Yards
                21 => kick_fgm_30_39 = m.value, // Field Goals 30-39 Yards
                22 => kick_fgm_40_49 = m.value, // Field Goals 40-49 Yards
                23 => kick_fgm_50p = m.value,   // Field Goals 50+ Yards
                29 => kick_xpm = m.value,       // Point After Attempt Made
                30 => kick_xpmiss = m.value,    // Point After Attempt Missed
                31 => {
                    // Points Allowed
                    // TODO: needs work, I think this is a multiplier not a band
                    // pts_allow_0 = m.value;
                }
                32 => sack = m.value,            // Sack
                33 => int = m.value,             // Interception (defensive)
                34 => fum_rec = m.value,         // Fumble Recovery
                35 => def_td = m.value,          // Touchdown (defensive)
                36 => safe = m.value,            // Safety
                37 => blk_kick = m.value,        // Block Kick
                49 => st_td = m.value,           // Kickoff and Punt Return Touchdowns
                50 => pts_allow_0 = m.value,     // Points Allowed 0 points
                51 => pts_allow_1_6 = m.value,   // Points Allowed 1-6 points
                52 => pts_allow_7_13 = m.value,  // Points Allowed 7-13 points
                53 => pts_allow_14_20 = m.value, // Points Allowed 14-20 points
                54 => pts_allow_21_27 = m.value, // Points Allowed 21-27 points
                55 => pts_allow_28_34 = m.value, // Points Allowed 28-34 points
                56 => pts_allow_35p = m.value,   // Points Allowed 35+ points
                57 => {
                    // Offensive Fumble Return TD
                    // TODO: is this right?
                    // fum_rec_td = m.value;
                }
                78 => {
                    // Targets
                    // TODO: implement points per target
                    // targets = m.value;
                }
                82 => {
                    // Extra Point Returned
                    // TODO: implement points per xp returned for td
                }
                _ => {
                    debug_assert!(
                        false,
                        "unknown yahoo league scoring setting stat modifier: {m:?}"
                    );
                } // unknown
            };
        }

        ScoringSettings::Football {
            pass_2pt,
            pass_int,
            pass_yd,
            pass_td,
            receptions,
            rec_2pt,
            rec_td,
            rec_yd,
            rush_2pt,
            rush_td,
            rush_yd,
            kick_fgmiss,
            kick_fgm_0_19,
            kick_fgm_20_29,
            kick_fgm_30_39,
            kick_fgm_40_49,
            kick_fgm_50p,
            kick_xpm,
            kick_xpmiss,
            fum,
            fum_lost,
            pts_allow_0,
            pts_allow_1_6,
            pts_allow_7_13,
            pts_allow_14_20,
            pts_allow_21_27,
            pts_allow_28_34,
            pts_allow_35p,
            int,
            sack,
            safe,
            def_td,
            def_kr_td,
            fum_rec,
            fum_rec_td,
            fum_forced,
            pass_int_td,
            def_st_td,
            def_st_fum_rec,
            def_st_ff,
            st_fum_rec,
            st_ff,
            st_td,
            blk_kick,
            kr_td,
            def_pr_td,
            pr_td,
            bonus_rec_yd_200,
            bonus_rush_yd_200,
            bonus_pass_yd_400,
            bonus_rec_te,
            bonus_pass_td_40p,
            bonus_rush_td_40p,
            bonus_pass_cmp_40p,
            bonus_rec_td_40p,
            bonus_rush_40p,
            bonus_rec_40p,
        }
    }
}
impl From<yahoo::Settings> for ScoringSettings {
    fn from(value: yahoo::Settings) -> Self {
        ScoringSettings::from(&value)
    }
}

#[cfg(test)]
mod tests {
    use yahoo_fantasy_rs as yahoo;

    use crate::{LeagueSettings, ScoringSettings};

    #[test]
    fn yahoo_league_division_settings_conversion() {
        let yahoo_settings = yahoo_settings();
        let league_settings: LeagueSettings = LeagueSettings::from_yahoo(&yahoo_settings);
        debug_assert_eq!(3, league_settings.divisions.len(), "expecting 3 divisions");
        debug_assert_eq!(
            vec!["North".to_string(), "South".to_string(), "West".to_string()],
            league_settings.divisions,
            "expecting correct division names"
        );
    }

    #[test]
    fn yahoo_football_scoring_settings_conversion() {
        let yahoo_settings = yahoo_settings();
        let scoring_settings: ScoringSettings = yahoo_settings.into();
        if let ScoringSettings::Football {
            pass_2pt,
            pass_int,
            pass_yd,
            pass_td,
            receptions,
            rec_2pt,
            rec_td,
            rec_yd,
            rush_2pt,
            rush_td,
            rush_yd,
            kick_fgmiss,
            kick_fgm_0_19,
            kick_fgm_20_29,
            kick_fgm_30_39,
            kick_fgm_40_49,
            kick_fgm_50p,
            kick_xpm,
            kick_xpmiss,
            fum,
            fum_lost,
            pts_allow_0,
            pts_allow_1_6,
            pts_allow_7_13,
            pts_allow_14_20,
            pts_allow_21_27,
            pts_allow_28_34,
            pts_allow_35p,
            int,
            sack,
            safe,
            def_td,
            def_kr_td,
            fum_rec,
            fum_rec_td,
            fum_forced,
            pass_int_td,
            def_st_td,
            def_st_fum_rec,
            def_st_ff,
            st_fum_rec,
            st_ff,
            st_td,
            blk_kick,
            kr_td,
            def_pr_td,
            pr_td,
            bonus_rec_yd_200,
            bonus_rush_yd_200,
            bonus_pass_yd_400,
            bonus_rec_te,
            bonus_pass_td_40p,
            bonus_rush_td_40p,
            bonus_pass_cmp_40p,
            bonus_rec_td_40p,
            bonus_rush_40p,
            bonus_rec_40p,
        } = scoring_settings
        {
            debug_assert_eq!(pass_2pt, 2.0, "expected 2 pts/passing 2-pt conversion");
            debug_assert_eq!(pass_int, -2.0, "expected -2.0 pts/passing int");
            debug_assert_eq!(pass_yd, 0.05, "expected 0.05 pts/passing yard");
            debug_assert_eq!(pass_td, 4.0, "expected 4 pts/passing td");
            debug_assert_eq!(receptions, 0.5, "expected 0.5 pts/reception");
            debug_assert_eq!(rec_2pt, 2.0, "expected 2 pts/rec 2-pt conversion");
            debug_assert_eq!(rec_td, 6.0, "expected 6 pts/rec td");
            debug_assert_eq!(rec_yd, 0.1, "expected 0.1 pts/rec yd");
            debug_assert_eq!(rush_2pt, 2.0, "expected 2 pts/rush 2-pt conversion");
            debug_assert_eq!(rush_td, 6.0, "expected 6 pts/rush td");
            debug_assert_eq!(rush_yd, 0.1, "expected 0.1 pts/rushing yard");
            debug_assert_eq!(
                kick_fgmiss, 0.0,
                "expected no negative points per field goal miss"
            );
            debug_assert_eq!(
                kick_fgm_0_19, 3.0,
                "expected 3 pts/field goal less than 20 yards"
            );
            debug_assert_eq!(
                kick_fgm_20_29, 3.0,
                "expected 3 pts/field goal between 20-29 yards"
            );
            debug_assert_eq!(
                kick_fgm_30_39, 3.0,
                "expected 3 pts/field goal between 30-39 yards"
            );
            debug_assert_eq!(
                kick_fgm_40_49, 4.0,
                "expected 4 pts/field goal between 40-49 yards"
            );
            debug_assert_eq!(kick_fgm_50p, 5.0, "expected 5 pts/field goal 50+ yards");
            debug_assert_eq!(kick_xpm, 1.0, "expected 1 pt/xp made");
            debug_assert_eq!(kick_xpmiss, -2.0, "expected -2 pts/missed xp");
            debug_assert_eq!(
                fum, 0.0,
                "expected no negative points for a fumble (not lost)"
            );
            debug_assert_eq!(fum_lost, -2.0, "expected -2 pts/fumble lost");
            debug_assert_eq!(pts_allow_0, 15.0, "expected 15 pts for a defensive shutout");
            debug_assert_eq!(
                pts_allow_1_6, 10.0,
                "expected 10 pts for a defense holding a team to 1-6 pts"
            );
            debug_assert_eq!(
                pts_allow_7_13, 5.0,
                "expected 5 pts for a defense holding a team to 7-13 pts"
            );
            debug_assert_eq!(
                pts_allow_14_20, 3.0,
                "expected 3 pts for a defense holding a team to 14-20 pts"
            );
            debug_assert_eq!(
                pts_allow_21_27, 2.0,
                "expected 2 pts for a defense holding a team to 21-27 pts"
            );
            debug_assert_eq!(
                pts_allow_28_34, 0.0,
                "expected 0 pts for a defense holding a team to 28-34 pts"
            );
            debug_assert_eq!(
                pts_allow_35p, -4.0,
                "expected -4 pts for a defense holding a team 35+ pts"
            );
            debug_assert_eq!(int, 2.0, "expected 2 pts/interception forced");
            debug_assert_eq!(sack, 2.0, "expected 2 pts/sack");
            debug_assert_eq!(safe, 3.0, "expected 2 pts/safety");
            debug_assert_eq!(def_td, 6.0, "expected 6 pts/defensive touchdown");
            debug_assert_eq!(
                def_kr_td, 0.0,
                "expected 0 pts for unsupported defensive kick return td"
            );
            debug_assert_eq!(fum_rec, 2.0, "expected 2 pts/defensive fumble recovery");
            debug_assert_eq!(
                fum_rec_td, 0.0,
                "expected 6 pts/defensive fumble recovery td"
            );
            debug_assert_eq!(fum_forced, 0.0, "expected 0 pts/fumbles forced");
            debug_assert_eq!(pass_int_td, 0.0, "expected 6 pts/pick-6");
            debug_assert_eq!(
                def_st_td, 0.0,
                "expected 0 pts for unsupported defense and special teams td"
            );
            debug_assert_eq!(
                def_st_fum_rec, 0.0,
                "expected 2 pts for a DST fumble recovery"
            );
            debug_assert_eq!(def_st_ff, 0.0, "expected 0.0 def_st_ff");
            debug_assert_eq!(st_fum_rec, 0.0, "expected 0.0 st_fum_rec");
            debug_assert_eq!(st_ff, 0.0, "expected 0.0 st_ff");
            debug_assert_eq!(st_td, 6.0, "expected 6.0 st_td");
            debug_assert_eq!(blk_kick, 2.0, "expected 2.0 blk_kick");
            debug_assert_eq!(kr_td, 6.0, "expected 6.0 kr_td");
            debug_assert_eq!(def_pr_td, 0.0, "expected 0.0 def_pr_td");
            debug_assert_eq!(pr_td, 6.0, "expected 6.0 pr_td");
            debug_assert_eq!(bonus_rec_yd_200, 0.0, "expected 0.0 bonus_rec_yd_200");
            debug_assert_eq!(bonus_rush_yd_200, 0.0, "expected 0.0 bonus_rush_yd_200");
            debug_assert_eq!(bonus_pass_yd_400, 0.0, "expected 0.0 bonus_pass_yd_400");
            debug_assert_eq!(bonus_rec_te, 0.0, "expected 0.0 bonus_rec_te");
            debug_assert_eq!(bonus_pass_td_40p, 0.0, "expected 0.0 bonus_pass_td_40p");
            debug_assert_eq!(bonus_rush_td_40p, 0.0, "expected 0.0 bonus_rush_td_40p");
            debug_assert_eq!(bonus_pass_cmp_40p, 0.0, "expected 0.0 bonus_pass_cmp_40p");
            debug_assert_eq!(bonus_rec_td_40p, 0.0, "expected 0.0 bonus_rec_td_40p");
            debug_assert_eq!(bonus_rush_40p, 0.0, "expected 0.0 bonus_rush_40p");
            debug_assert_eq!(bonus_rec_40p, 0.0, "expected 0.0 bonus_rec_40p");
        } else {
            debug_assert!(false, "expected football scoring settings");
        }
    }

    fn yahoo_settings() -> yahoo::Settings {
        yahoo::Settings {
            draft_type: "live".to_string(),
            scoring_type: "head".to_string(),
            uses_playoff: 1,
            playoff_start_week: 14,
            uses_playoff_reseeding: 0,
            uses_lock_eliminated_teams: 0,
            uses_faab: 1,
            trade_end_date: "2018-11-10".to_string(),
            trade_ratify_type: "commish".to_string(),
            trade_reject_time: 2,
            stat_categories: [
                yahoo::Stat {
                    stat_id: 4,
                    enabled: true,
                    name: "Passing Yards".to_string(),
                    display_name: "Pass Yds".to_string(),
                    group: "passing".to_string(),
                    abbr: "Yds".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 5,
                    enabled: true,
                    name: "Passing Touchdowns".to_string(),
                    display_name: "Pass TD".to_string(),
                    group: "passing".to_string(),
                    abbr: "TD".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 6,
                    enabled: true,
                    name: "Interceptions".to_string(),
                    display_name: "Int".to_string(),
                    group: "passing".to_string(),
                    abbr: "Int".to_string(),
                    sort_order: 0,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 8,
                    enabled: true,
                    name: "Rushing Attempts".to_string(),
                    display_name: "Rush Att".to_string(),
                    group: "rushing".to_string(),
                    abbr: "Att".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 9,
                    enabled: true,
                    name: "Rushing Yards".to_string(),
                    display_name: "Rush Yds".to_string(),
                    group: "rushing".to_string(),
                    abbr: "Yds".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 10,
                    enabled: true,
                    name: "Rushing Touchdowns".to_string(),
                    display_name: "Rush TD".to_string(),
                    group: "rushing".to_string(),
                    abbr: "TD".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 11,
                    enabled: true,
                    name: "Receptions".to_string(),
                    display_name: "Rec".to_string(),
                    group: "receiving".to_string(),
                    abbr: "Rec".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 12,
                    enabled: true,
                    name: "Receiving Yards".to_string(),
                    display_name: "Rec Yds".to_string(),
                    group: "receiving".to_string(),
                    abbr: "Yds".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 13,
                    enabled: true,
                    name: "Receiving Touchdowns".to_string(),
                    display_name: "Rec TD".to_string(),
                    group: "receiving".to_string(),
                    abbr: "TD".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 15,
                    enabled: true,
                    name: "Return Touchdowns".to_string(),
                    display_name: "Ret TD".to_string(),
                    group: "return".to_string(),
                    abbr: "TD".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 16,
                    enabled: true,
                    name: "2-Point Conversions".to_string(),
                    display_name: "2-PT".to_string(),
                    group: "misc".to_string(),
                    abbr: "2-PT".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 18,
                    enabled: true,
                    name: "Fumbles Lost".to_string(),
                    display_name: "Fum Lost".to_string(),
                    group: "fumbles".to_string(),
                    abbr: "Lost".to_string(),
                    sort_order: 0,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 78,
                    enabled: true,
                    name: "Targets".to_string(),
                    display_name: "Targets".to_string(),
                    group: "receiving".to_string(),
                    abbr: "Targets".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 57,
                    enabled: true,
                    name: "Offensive Fumble Return TD".to_string(),
                    display_name: "Fum Ret TD".to_string(),
                    group: "misc".to_string(),
                    abbr: "Off Fumb TD".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::O,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 19,
                    enabled: true,
                    name: "Field Goals 0-19 Yards".to_string(),
                    display_name: "FG 0-19".to_string(),
                    group: "fgs".to_string(),
                    abbr: "0-19 Yds".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::K,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 20,
                    enabled: true,
                    name: "Field Goals 20-29 Yards".to_string(),
                    display_name: "FG 20-29".to_string(),
                    group: "fgs".to_string(),
                    abbr: "20-29 Yds".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::K,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 21,
                    enabled: true,
                    name: "Field Goals 30-39 Yards".to_string(),
                    display_name: "FG 30-39".to_string(),
                    group: "fgs".to_string(),
                    abbr: "30-39 Yds".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::K,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 22,
                    enabled: true,
                    name: "Field Goals 40-49 Yards".to_string(),
                    display_name: "FG 40-49".to_string(),
                    group: "fgs".to_string(),
                    abbr: "40-49 Yds".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::K,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 23,
                    enabled: true,
                    name: "Field Goals 50+ Yards".to_string(),
                    display_name: "FG 50+".to_string(),
                    group: "fgs".to_string(),
                    abbr: "50+ Yds".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::K,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 29,
                    enabled: true,
                    name: "Point After Attempt Made".to_string(),
                    display_name: "PAT Made".to_string(),
                    group: "pat".to_string(),
                    abbr: "Made".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::K,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 30,
                    enabled: true,
                    name: "Point After Attempt Missed".to_string(),
                    display_name: "PAT Miss".to_string(),
                    group: "pat".to_string(),
                    abbr: "Miss".to_string(),
                    sort_order: 0,
                    position_type: yahoo::PositionType::K,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 31,
                    enabled: true,
                    name: "Points Allowed".to_string(),
                    display_name: "Pts Allow".to_string(),
                    group: "none".to_string(),
                    abbr: "Pts vs.".to_string(),
                    sort_order: 0,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 32,
                    enabled: true,
                    name: "Sack".to_string(),
                    display_name: "Sack".to_string(),
                    group: "team_tackles".to_string(),
                    abbr: "Sack".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 33,
                    enabled: true,
                    name: "Interception".to_string(),
                    display_name: "Int".to_string(),
                    group: "def_turnovers".to_string(),
                    abbr: "Int".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 34,
                    enabled: true,
                    name: "Fumble Recovery".to_string(),
                    display_name: "Fum Rec".to_string(),
                    group: "def_turnovers".to_string(),
                    abbr: "Fum Rec".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 35,
                    enabled: true,
                    name: "Touchdown".to_string(),
                    display_name: "TD".to_string(),
                    group: "tds".to_string(),
                    abbr: "TD".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 36,
                    enabled: true,
                    name: "Safety".to_string(),
                    display_name: "Safe".to_string(),
                    group: "team_tackles".to_string(),
                    abbr: "Safe".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 37,
                    enabled: true,
                    name: "Block Kick".to_string(),
                    display_name: "Blk Kick".to_string(),
                    group: "misc".to_string(),
                    abbr: "Blk Kick".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 49,
                    enabled: true,
                    name: "Kickoff and Punt Return Touchdowns".to_string(),
                    display_name: "Kick and Punt Ret TD".to_string(),
                    group: "return".to_string(),
                    abbr: "TD".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 50,
                    enabled: true,
                    name: "Points Allowed 0 points".to_string(),
                    display_name: "Pts Allow 0".to_string(),
                    group: "pts_allow".to_string(),
                    abbr: "0 Pts".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 51,
                    enabled: true,
                    name: "Points Allowed 1-6 points".to_string(),
                    display_name: "Pts Allow 1-6".to_string(),
                    group: "pts_allow".to_string(),
                    abbr: "1-6 Pts".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 52,
                    enabled: true,
                    name: "Points Allowed 7-13 points".to_string(),
                    display_name: "Pts Allow 7-13".to_string(),
                    group: "pts_allow".to_string(),
                    abbr: "7-13 Pts".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 53,
                    enabled: true,
                    name: "Points Allowed 14-20 points".to_string(),
                    display_name: "Pts Allow 14-20".to_string(),
                    group: "pts_allow".to_string(),
                    abbr: "14-20 Pts".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 54,
                    enabled: true,
                    name: "Points Allowed 21-27 points".to_string(),
                    display_name: "Pts Allow 21-27".to_string(),
                    group: "pts_allow".to_string(),
                    abbr: "21-27 Pts".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 55,
                    enabled: true,
                    name: "Points Allowed 28-34 points".to_string(),
                    display_name: "Pts Allow 28-34".to_string(),
                    group: "pts_allow".to_string(),
                    abbr: "28-34 Pts".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 56,
                    enabled: true,
                    name: "Points Allowed 35+ points".to_string(),
                    display_name: "Pts Allow 35+".to_string(),
                    group: "pts_allow".to_string(),
                    abbr: "35+ Pts".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
                yahoo::Stat {
                    stat_id: 82,
                    enabled: true,
                    name: "Extra Point Returned".to_string(),
                    display_name: "XPR".to_string(),
                    group: "misc".to_string(),
                    abbr: "XPR".to_string(),
                    sort_order: 1,
                    position_type: yahoo::PositionType::DT,
                    stat_position_types: vec![],
                },
            ]
            .to_vec(),
            stat_modifiers: [
                yahoo::StatModifier {
                    stat_id: 4,
                    value: 0.05,
                },
                yahoo::StatModifier {
                    stat_id: 5,
                    value: 4.0,
                },
                yahoo::StatModifier {
                    stat_id: 6,
                    value: -2.0,
                },
                yahoo::StatModifier {
                    stat_id: 9,
                    value: 0.1,
                },
                yahoo::StatModifier {
                    stat_id: 10,
                    value: 6.0,
                },
                yahoo::StatModifier {
                    stat_id: 11,
                    value: 0.5,
                },
                yahoo::StatModifier {
                    stat_id: 12,
                    value: 0.1,
                },
                yahoo::StatModifier {
                    stat_id: 13,
                    value: 6.0,
                },
                yahoo::StatModifier {
                    stat_id: 15,
                    value: 6.0,
                },
                yahoo::StatModifier {
                    stat_id: 16,
                    value: 2.0,
                },
                yahoo::StatModifier {
                    stat_id: 18,
                    value: -2.0,
                },
                yahoo::StatModifier {
                    stat_id: 57,
                    value: 6.0,
                },
                yahoo::StatModifier {
                    stat_id: 19,
                    value: 3.0,
                },
                yahoo::StatModifier {
                    stat_id: 20,
                    value: 3.0,
                },
                yahoo::StatModifier {
                    stat_id: 21,
                    value: 3.0,
                },
                yahoo::StatModifier {
                    stat_id: 22,
                    value: 4.0,
                },
                yahoo::StatModifier {
                    stat_id: 23,
                    value: 5.0,
                },
                yahoo::StatModifier {
                    stat_id: 29,
                    value: 1.0,
                },
                yahoo::StatModifier {
                    stat_id: 30,
                    value: -2.0,
                },
                yahoo::StatModifier {
                    stat_id: 32,
                    value: 2.0,
                },
                yahoo::StatModifier {
                    stat_id: 33,
                    value: 2.0,
                },
                yahoo::StatModifier {
                    stat_id: 34,
                    value: 2.0,
                },
                yahoo::StatModifier {
                    stat_id: 35,
                    value: 6.0,
                },
                yahoo::StatModifier {
                    stat_id: 36,
                    value: 3.0,
                },
                yahoo::StatModifier {
                    stat_id: 37,
                    value: 2.0,
                },
                yahoo::StatModifier {
                    stat_id: 49,
                    value: 6.0,
                },
                yahoo::StatModifier {
                    stat_id: 50,
                    value: 15.0,
                },
                yahoo::StatModifier {
                    stat_id: 51,
                    value: 10.0,
                },
                yahoo::StatModifier {
                    stat_id: 52,
                    value: 5.0,
                },
                yahoo::StatModifier {
                    stat_id: 53,
                    value: 3.0,
                },
                yahoo::StatModifier {
                    stat_id: 54,
                    value: 2.0,
                },
                yahoo::StatModifier {
                    stat_id: 55,
                    value: 0.0,
                },
                yahoo::StatModifier {
                    stat_id: 56,
                    value: -4.0,
                },
                yahoo::StatModifier {
                    stat_id: 82,
                    value: 2.0,
                },
            ]
            .to_vec(),
            divisions: [
                yahoo::Division {
                    division_id: 1,
                    name: "North".to_string(),
                },
                yahoo::Division {
                    division_id: 2,
                    name: "South".to_string(),
                },
                yahoo::Division {
                    division_id: 3,
                    name: "West".to_string(),
                },
            ]
            .to_vec(),
        }
    }
}
