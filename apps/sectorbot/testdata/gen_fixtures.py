#!/usr/bin/env python3
"""Generates the SYNTHETIC fixtures used by sectorbot's tests.

Everything here is invented: the "Example Rovers" team, all people, accounts, channels,
videos, ids and rankings are fictional. The JSON/HTML shapes mirror the real APIs
(Twitter syndication, YouTube Data API v3, FantasyPros ECR page, Sleeper) so the parsers
are exercised realistically without redistributing anyone's content.

Run from sectorbot/:   python3 testdata/gen_fixtures.py
Output is deterministic.
"""
import gzip
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))


def write_json(rel, obj, indent=None):
    path = os.path.join(HERE, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(obj, f, indent=indent, ensure_ascii=False)
        f.write("\n")


# ------------------------------------------------------------------ twitter
def user(uid, name, handle):
    return {
        "id_str": uid,
        "name": name,
        "screen_name": handle,
        "is_blue_verified": True,
        "profile_image_shape": "Circle",
        "verified": False,
        "profile_image_url_https": f"https://pbs.example.invalid/profile_images/{uid}/avatar_normal.jpg",
    }


def media(tweet_url, short, img, start):
    disp = f"pic.example.invalid/{short}"
    tco = f"https://t.example.invalid/{short}"
    ent = {"display_url": disp, "expanded_url": tweet_url + "/photo/1", "indices": [start, start + 23], "url": tco}
    det = dict(ent)
    det.update({
        "ext_media_availability": {"status": "Available"},
        "media_url_https": img,
        "original_info": {"height": 900, "width": 600, "focus_rects": []},
        "sizes": {
            "large": {"h": 900, "resize": "fit", "w": 600},
            "medium": {"h": 900, "resize": "fit", "w": 600},
            "small": {"h": 680, "resize": "fit", "w": 383},
            "thumb": {"h": 150, "resize": "crop", "w": 150},
        },
        "type": "photo",
    })
    return ent, det


def tweet(tid, created, usr, text, quoted=None, hashtags=(), mentions=(), media_spec=None, extra=None):
    entities = {"hashtags": [], "urls": [], "user_mentions": [], "symbols": []}
    for tag in hashtags:
        i = text.index("#" + tag)
        entities["hashtags"].append({"indices": [i, i + len(tag) + 1], "text": tag})
    for uid, name, handle in mentions:
        i = text.index("@" + handle)
        entities["user_mentions"].append({"id_str": uid, "indices": [i, i + len(handle) + 1], "name": name, "screen_name": handle})
    t = {
        "__typename": "Tweet",
        "lang": "en",
        "favorite_count": 212,
        "created_at": created,
        "display_text_range": [0, len(text)],
        "entities": entities,
        "id_str": tid,
        "text": text,
        "user": usr,
        "edit_control": {"edit_tweet_ids": [tid], "editable_until_msecs": "1700000000000", "is_edit_eligible": True, "edits_remaining": "5"},
        "conversation_count": 14,
        "news_action_type": "conversation",
        "isEdited": False,
        "isStaleEdit": False,
    }
    if media_spec:
        ent, det = media_spec
        entities["media"] = [ent]
        t["mediaDetails"] = [det]
        t["photos"] = [{"expandedUrl": ent["expanded_url"], "url": det["media_url_https"], "width": 600, "height": 900}]
    if quoted:
        t["quoted_tweet"] = quoted
    if extra:
        t.update(extra)
    return t


SAM = user("1000000001", "Sam Sample", "SampleScribe")
ALEX = user("1000000002", "Alex Quote", "alexquote")
RILEY = ("1000000003", "Riley Rookie", "RileyRookie")


def tweet_with_quote_and_media():
    q_id = "1800000000000000099"
    q_url = f"https://x.example.invalid/alexquote/status/{q_id}"
    q_text = f"⁦@RileyRookie⁩ has reason to smile. He continues to handle his business and held his own at right tackle with the first team https://t.example.invalid/pic001"
    q = tweet(q_id, "2025-01-15T16:55:17.000Z", ALEX, q_text, mentions=[RILEY], media_spec=media(q_url, "pic001", "https://pbs.example.invalid/media/sample001.jpg", len(q_text) - 34))
    q["lang"] = "en"
    text = ("If rookie Riley Rookie hits for the Rovers -- and not just in a down the line kinda way -- extend the OL coach immediately.\n"
            "The Rovers have been desperate for a Day 3 OL draft hit since Pat Anchor (Lee Backup I guess).")
    return tweet("1800000000000000101", "2025-01-15T17:13:22.000Z", SAM, text, quoted=q)


def tweet_response_fixture():
    q_id = "1800000000000000040"
    q_url = f"https://x.example.invalid/alexquote/status/{q_id}"
    q = tweet(q_id, "2024-01-01T20:49:05.000Z", ALEX, "https://t.example.invalid/pic002", media_spec=media(q_url, "pic002", "https://pbs.example.invalid/media/sample002.jpg", 0))
    q["lang"] = "zxx"
    text = "I have to be the first person in Sample Takes history to be on the good end of one of these right??\nLFG #Rovers !!!"
    return tweet("1800000000000000050", "2024-01-01T21:01:16.000Z", SAM, text, quoted=q, hashtags=["Rovers"],
                 extra={"place": {"full_name": "Exampleton, EX", "id": "0000000000000001"}})


def timeline_entry(i, tid):
    text = f"Sample update #{i}: the Rovers practiced today. #Rovers"
    t = tweet(tid, "2025-01-15T12:00:00.000Z", SAM, text, hashtags=["Rovers"])
    t.pop("__typename")
    t.update({"id": 0, "location": "", "conversation_id_str": tid,
              "created_at": f"Wed Jan 15 12:{i:02d}:00 +0000 2025",
              "full_text": text, "permalink": f"/SampleScribe/status/{tid}", "favorited": False,
              "possibly_sensitive": False, "quote_count": 0, "reply_count": 1, "retweet_count": 2, "retweeted": False})
    return {"type": "tweet", "entry_id": f"tweet-{tid}", "sort_index": tid, "content": {"tweet": t}}


TIMELINE_COUNT = 5


def timeline_html():
    ids = [str(1800000000000000200 + i) for i in range(TIMELINE_COUNT)][::-1]
    data = {
        "props": {"pageProps": {
            "contextProvider": {}, "lang": "en", "maxHeight": None, "showHeader": True, "hideBorder": False,
            "hideFooter": False, "hideScrollBar": False, "transparent": False,
            "timeline": {"entries": [timeline_entry(len(ids) - n, tid) for n, tid in enumerate(ids)]},
            "latest_tweet_id": ids[0], "headerProps": {},
        }, "__N_SSP": True},
        "page": "/timeline-profile/screen-name/[screenName]",
        "query": {"screenName": "SampleScribe"},
        "buildId": "syntheticBuildId", "assetPrefix": "https://platform.example.invalid",
        "isFallback": False, "gssp": True, "customServer": True, "scriptLoader": [],
    }
    blob = json.dumps(data, ensure_ascii=False).replace(">", "\\u003e")  # parser regex forbids '>'
    html = ('<!DOCTYPE html><html><head><meta charSet="utf-8"/><title>synthetic timeline</title></head><body>'
            f'<script id="__NEXT_DATA__" type="application/json">{blob}</script></body></html>\n')
    with open(os.path.join(HERE, "timeline.html"), "w", encoding="utf-8") as f:
        f.write(html)


# ------------------------------------------------------------------ youtube
def thumbs(prefix, sizes):
    return {k: {"url": f"https://img.example.invalid/{prefix}/{k}.jpg", "width": w, "height": h} for k, (w, h) in sizes.items()}


CHANNELS = [
    # id, title, handle, description
    ("UCexampleRoverBanter0001", "Rover Banter", "@roverbanter", "Sam and Alex of the Rover Banter podcast break down All-22 game film. (synthetic)"),
    ("UCexampleRoverInsider001", "The Rover Insider", "@theroverinsider", "The latest news on everything Example Rovers. (synthetic)"),
    ("UCexampleRoversCollect01", "Rovers Collective", "@roverscollective", "Fan-run Example Rovers community channel. (synthetic)"),
    ("UCexampleRoverRadio00001", "Rover Radio", "@roverradio", "Talk radio for Example Rovers fans. (synthetic)"),
]
CH_SIZES = {"default": (88, 88), "medium": (240, 240), "high": (800, 800)}


def yt_channels():
    items = []
    for n, (cid, title, handle, desc) in enumerate(CHANNELS):
        items.append({
            "kind": "youtube#channel", "etag": f"etagChannel{n}", "id": cid,
            "snippet": {"title": title, "description": desc, "customUrl": handle, "publishedAt": f"202{n}-02-08T15:25:49.563614Z",
                        "thumbnails": thumbs(cid, CH_SIZES), "localized": {"title": title, "description": desc}, "country": "US"},
            "contentDetails": {"relatedPlaylists": {"likes": "", "uploads": "UU" + cid[2:]}},
            "statistics": {"viewCount": str(1000 * (n + 1)), "subscriberCount": str(100 * (n + 1)), "hiddenSubscriberCount": False, "videoCount": str(10 * (n + 1))},
            "topicDetails": {"topicIds": ["/m/0jm_", "/m/06ntj"], "topicCategories": ["https://en.wikipedia.org/wiki/American_football", "https://en.wikipedia.org/wiki/Sport"]},
            "status": {"privacyStatus": "public", "isLinked": True, "longUploadsStatus": "longUploadsUnspecified", "madeForKids": False},
        })
    return {"kind": "youtube#channelListResponse", "etag": "etagChannelList", "pageInfo": {"totalResults": len(items), "resultsPerPage": 50}, "items": items}


def yt_search():
    items = []
    extra = [("UCexampleRoverFilm000001", "Rover Film Room", "Film study for Example Rovers. (synthetic)")]
    rows = [(c[0], c[1], c[3]) for c in CHANNELS] + extra
    for n, (cid, title, desc) in enumerate(rows):
        pub = "2021-02-13T23:01:14Z" if n == 0 else f"202{n}-03-0{n}T10:00:00Z"
        items.append({
            "kind": "youtube#searchResult", "etag": f"etagSearch{n}",
            "id": {"kind": "youtube#channel", "channelId": cid},
            "snippet": {"publishedAt": pub, "channelId": cid, "title": title, "description": desc,
                        "thumbnails": {k: {"url": f"https://img.example.invalid/{cid}/{k}.jpg"} for k in ("default", "medium", "high")},
                        "channelTitle": title, "liveBroadcastContent": "none", "publishTime": pub},
        })
    return {"kind": "youtube#searchListResponse", "etag": "etagSearchList", "nextPageToken": "SYNTHETICTOKEN", "regionCode": "US",
            "pageInfo": {"totalResults": 12, "resultsPerPage": 5}, "items": items}


VIDEOS = [
    ("exampleVid01", "Rovers Camp: rookie delivers + big play in practice", "Sam and Alex break down the last two Rovers practices and a deep ball. That and a whole lot more!\n\n#Rovers #football #camp"),
    ("exampleVid02", "Rovers Film Room: the offensive line", "Breaking down the tape on the Rovers offensive line. (synthetic)\n\n#Rovers #film"),
    ("exampleVid03", "Rover Banter Mailbag", "Answering your questions. (synthetic)\n\n#Rovers #mailbag"),
    ("exampleVid04", "Rovers Preseason Preview", "What to watch this preseason. (synthetic)\n\n#Rovers #preseason"),
    ("exampleVid05", "Rovers Draft Recap", "Grading every pick. (synthetic)\n\n#Rovers #draft"),
]
PL_SIZES = {"default": (120, 90), "medium": (320, 180), "high": (480, 360), "standard": (640, 480), "maxres": (1280, 720)}


def yt_playlist():
    cid, ctitle = CHANNELS[0][0], CHANNELS[0][1]
    items = []
    for n, (vid, title, desc) in enumerate(VIDEOS):
        items.append({
            "kind": "youtube#playlistItem", "etag": f"etagPlaylistItem{n}", "id": f"syntheticPlaylistItemId{n}",
            "snippet": {"publishedAt": f"2025-01-{15 - n:02d}T21:18:21Z", "channelId": cid, "title": title, "description": desc,
                        "thumbnails": thumbs(vid, PL_SIZES), "channelTitle": ctitle, "playlistId": "UU" + cid[2:], "position": n,
                        "resourceId": {"kind": "youtube#video", "videoId": vid}, "videoOwnerChannelTitle": ctitle, "videoOwnerChannelId": cid},
            "contentDetails": {"videoId": vid, "videoPublishedAt": f"2025-01-{15 - n:02d}T21:18:21Z"},
        })
    return {"kind": "youtube#playlistItemListResponse", "etag": "etagPlaylist", "nextPageToken": "SYNTHETICTOKEN", "items": items,
            "pageInfo": {"totalResults": 50, "resultsPerPage": len(items)}}


# ------------------------------------------------------------------ fantasy
# fictional player pool shared by the FantasyPros page and the keepables context
POOL = [  # sleeper id, first, last, pos, team
    ("9001", "Casey", "Catcher", "WR"), ("9002", "Nico", "Nimble", "WR"), ("9003", "Will", "Wheeler", "QB"),
    ("9004", "Rex", "Runner", "RB"), ("9005", "Dana", "Dasher", "RB"), ("9006", "Tom", "Tightend", "TE"),
    ("9007", "Gabe", "Gunslinger", "QB"), ("9008", "Ira", "Iron", "RB"), ("9009", "Walt", "Wideout", "WR"),
    ("9010", "Tess", "Tackle", "TE"),
]
TEAM = "EXR"


def fp_html():
    players = []
    for rank, (sid, first, last, pos) in enumerate(POOL, start=1):
        name = f"{first} {last}"
        players.append({
            "player_id": 90000 + rank, "player_name": name, "player_team_id": TEAM, "player_position_id": pos, "player_positions": pos,
            "player_short_name": f"{first[0]}. {last}", "player_eligibility": pos,
            "player_page_url": f"https://rankings.example.invalid/players/{first.lower()}-{last.lower()}.php",
            "player_filename": f"{first.lower()}-{last.lower()}.php", "player_bye_week": "10", "player_owned_avg": 99.0 - rank,
            "player_ecr_delta": None, "rank_ecr": rank, "rank_min": str(max(1, rank - 2)), "rank_max": str(rank + 3),
            "rank_ave": f"{rank:.2f}", "rank_std": "1.00", "pos_rank": f"{pos}{rank}", "tier": 1 + rank // 4,
        })
    ecr = {"sport": "NFL", "type": "Draft Half PPR", "ranking_type_name": "draft", "year": "2025", "week": "0", "position_id": "ALL",
           "scoring": "HALF", "filters": "1,2,3", "count": len(players), "total_experts": 10, "last_updated": "1/15",
           "players": players, "last_updated_ts": 1736943600, "experts_available": {"1": "Synthetic Expert"}, "accessed": "2025-01-15 12:00:00"}
    html = ("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n    <title>Synthetic Half PPR Rankings</title>\n</head>\n<body>\n"
            "<script type=\"text/javascript\">\n    var ecrData = " + json.dumps(ecr, ensure_ascii=False) + ";\n</script>\n</body>\n</html>\n")
    path = os.path.join(HERE, "fantasypros", "synthetic_rankings_half_ppr.html.gz")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as raw, gzip.GzipFile(fileobj=raw, mode="wb", mtime=0) as g:
        g.write(html.encode())


# standard Sleeper half-PPR scoring keys (generic; nothing league-specific)
SCORING = {"pass_2pt": 2.0, "pass_int": -2.0, "pass_yd": 0.05, "pass_td": 4.0, "rec": 0.5, "rec_2pt": 2.0, "rec_td": 6.0, "rec_yd": 0.1, "rush_2pt": 2.0, "rush_td": 6.0, "rush_yd": 0.1, "fgmiss": 0.0, "fgm_0_19": 3.0, "fgm_20_29": 3.0, "fgm_30_39": 3.0, "fgm_40_49": 4.0, "fgm_50p": 5.0, "xpm": 1.0, "xpmiss": -2.0, "fum": 0.0, "fum_lost": -2.0, "pts_allow_0": 15.0, "pts_allow_1_6": 10.0, "pts_allow_7_13": 5.0, "pts_allow_14_20": 3.0, "pts_allow_21_27": 2.0, "pts_allow_28_34": 0.0, "pts_allow_35p": -4.0, "int": 2.0, "sack": 2.0, "safe": 3.0, "def_td": 6.0, "def_kr_td": 0.0, "fum_rec": 2.0, "fum_rec_td": 6.0, "ff": 1.0, "pass_int_td": 0.0, "def_st_td": 6.0, "def_st_fum_rec": 2.0, "def_st_ff": 0.0, "st_fum_rec": 1.0, "st_ff": 1.0, "st_td": 6.0, "blk_kick": 2.0, "kr_td": 6.0, "def_pr_td": 0.0, "pr_td": 6.0, "bonus_rec_yd_200": 0.0, "pass_td_40p": 0.0, "bonus_rush_yd_200": 0.0, "bonus_pass_yd_400": 0.0, "rush_td_40p": 0.0, "bonus_rec_te": 0.0, "pass_cmp_40p": 0.0, "rec_td_40p": 0.0, "rush_40p": 0.0, "rec_40p": 0.0}
LEAGUE_ID = "2000000000000000001"
OWNERS = [("3000000000000000001", "SampleOwner"), ("3000000000000000002", "ManagerBeta"),
          ("3000000000000000003", "ManagerGamma"), ("3000000000000000004", "ManagerDelta")]
NUM_TEAMS = len(OWNERS)


PLAYER_NULL_KEYS = ["injury_status", "injury_body_part", "injury_start_date", "practice_participation", "practice_description", "news_updated",
                    "team_changed_at", "age", "height", "weight", "birth_date", "birth_city", "birth_state", "birth_country", "high_school", "college",
                    "depth_chart_position", "depth_chart_order", "stats_id", "rotowire_id", "espn_id", "yahoo_id", "fantasy_data_id", "rotoworld_id",
                    "oddsjam_id", "sportradar_id", "gsis_id", "swish_id", "pandascore_id", "metadata", "competitions"]


def sleeper_player(sid, first, last, pos):
    p = {k: None for k in PLAYER_NULL_KEYS}
    p.update({"active": True, "player_id": sid, "first_name": first, "last_name": last, "hashtag": f"#{first.lower()}{last.lower()}-NFL-{TEAM}-1",
              "status": "Active", "sport": "nfl", "fantasy_positions": [pos], "number": int(sid) % 100, "position": pos, "team": TEAM,
              "search_first_name": first.lower(), "search_last_name": last.lower(), "search_full_name": f"{first}{last}".lower(),
              "search_rank": int(sid) - 9000, "years_exp": 2})
    return p


def pick(draft_id, pick_no, rnd, roster_id, owner_id, pid, keeper):
    first, last, pos = {p[0]: (p[1], p[2], p[3]) for p in POOL}[pid]
    return {"player_id": pid, "picked_by": owner_id, "roster_id": roster_id, "round": rnd, "draft_slot": roster_id, "pick_no": pick_no,
            "metadata": {"first_name": first, "injury_status": "", "last_name": last, "news_updated": "1700000000000", "number": str(int(pid) % 100),
                         "player_id": pid, "position": pos, "sport": "nfl", "status": "Active", "team": TEAM, "years_exp": "2"},
            "is_keeper": keeper, "draft_id": draft_id}


def draft(season, picks):
    did = f"20000000000000{season}"
    return {"draft": {
        "draft_id": did, "league_id": LEAGUE_ID, "sport": "nfl", "season": season, "season_type": "regular", "created": 1700000000000,
        "creators": [OWNERS[0][0]], "draft_order": {o[0]: i + 1 for i, o in enumerate(OWNERS)}, "metadata": {"name": "Synthetic League", "scoring_type": "half_ppr"},
        "settings": {"alpha_sort": 0, "cpu_autopick": 1, "rounds": 10, "player_type": 0, "pick_timer": 90, "teams": NUM_TEAMS},
        "status": "complete", "type": "snake", "slot_to_roster_id": {str(i + 1): i + 1 for i in range(NUM_TEAMS)},
    }, "picks": [pick(did, *p) for p in picks]}


def keepables_ctx():
    """Scenarios (keeper cost for season 2025, derived from the draft history):
       9001 Casey Catcher : kept in 2022, 2023 and 2024 (3 seasons)  -> cost "ADP"
       9002 Nico Nimble   : drafted rd 2 in 2024, not kept           -> cost "2"
       9003 Will Wheeler  : drafted rd 10 in 2024                    -> cost "10"
       9004 Rex Runner    : drafted rd 5 in 2023, kept (rd 4) 2024   -> cost "4"
       9005 Dana Dasher   : on a roster but never drafted            -> not a keepable
    """
    # rows: (pick_no, round, roster_id, owner_id, player_id, is_keeper)
    d2022 = [(1, 3, 1, OWNERS[0][0], "9001", True)]
    d2023 = [(1, 3, 1, OWNERS[0][0], "9001", True), (17, 5, 2, OWNERS[1][0], "9004", False)]
    d2024 = [(1, 3, 1, OWNERS[0][0], "9001", True), (6, 2, 1, OWNERS[0][0], "9002", False), (40, 10, 1, OWNERS[0][0], "9003", False),
             (14, 4, 2, OWNERS[1][0], "9004", True)]
    users = [{"username": None, "user_id": oid, "display_name": name, "avatar": None, "is_bot": False, "is_owner": i == 0, "league_id": LEAGUE_ID,
              "metadata": {"team_name": f"Team {name}"}} for i, (oid, name) in enumerate(OWNERS)]
    rosters = []
    for i, (oid, _) in enumerate(OWNERS):
        mine = {0: ["9001", "9002", "9003", "9005"], 1: ["9004", "9006"], 2: ["9007", "9008"], 3: ["9009", "9010"]}[i]
        rosters.append({"roster_id": i + 1, "owner_id": oid, "league_id": LEAGUE_ID, "starters": mine[:2],
                        "settings": {"wins": 0, "losses": 0, "ties": 0, "fpts": 0, "division": 1 + i % 2, "total_moves": 0, "waiver_position": i + 1, "waiver_budget_used": 0},
                        "reserve": None, "players": mine, "co_owners": None, "keepers": None})
    league = {
        "total_rosters": NUM_TEAMS, "status": "pre_draft", "sport": "nfl", "season_type": "regular", "season": "2025",
        "settings": {"best_ball": 0, "leg": 1, "last_leg": None, "last_scored_leg": None, "max_keepers": 3, "num_teams": NUM_TEAMS, "offseason_adds": 0,
                     "bench_locks": 0, "disable_trades": 0, "playoff_teams": 2, "playoff_week_start": 15, "start_week": 1, "type": 1, "waiver_type": 2,
                     "playoff_type": 0, "pick_trading": 1, "trade_deadline": 10, "reserve_slots": 1, "divisions": 2},
        "scoring_settings": SCORING,
        "roster_positions": ["QB", "RB", "WR", "TE", "FLEX", "BN"], "previous_league_id": None, "name": "Synthetic League",
        "league_id": LEAGUE_ID, "draft_id": "20000000000000" + "2025", "avatar": None, "last_pinned_message_id": None, "metadata": None,
    }
    ctx = {
        "fetched_at": 1736943600000, "current_league": league,
        "draft_history": {"2022": draft("2022", d2022), "2023": draft("2023", d2023), "2024": draft("2024", d2024)},
        "players": {p[0]: sleeper_player(*p) for p in POOL},
        "rosters": rosters, "users": users, "rankings": fp_ecr_dict(),
    }
    write_json("ecr/keepables_ctx.json", ctx, indent=1)


def fp_ecr_dict():
    # same data as the FantasyPros page, as an object
    players = []
    for rank, (sid, first, last, pos) in enumerate(POOL, start=1):
        players.append({
            "player_id": 90000 + rank, "player_name": f"{first} {last}", "player_team_id": TEAM, "player_position_id": pos, "player_positions": pos,
            "player_short_name": f"{first[0]}. {last}", "player_eligibility": pos,
            "player_page_url": f"https://rankings.example.invalid/players/{first.lower()}-{last.lower()}.php",
            "player_filename": f"{first.lower()}-{last.lower()}.php", "player_bye_week": "10", "player_owned_avg": 99.0 - rank,
            "player_ecr_delta": None, "rank_ecr": rank, "rank_min": str(max(1, rank - 2)), "rank_max": str(rank + 3),
            "rank_ave": f"{rank:.2f}", "rank_std": "1.00", "pos_rank": f"{pos}{rank}", "tier": 1 + rank // 4})
    return {"sport": "NFL", "type": "Draft Half PPR", "ranking_type_name": "draft", "year": "2025", "week": "0", "position_id": "ALL",
            "scoring": "HALF", "filters": "1,2,3", "count": len(players), "total_experts": 10, "last_updated": "1/15", "players": players,
            "last_updated_ts": 1736943600, "experts_available": {"1": "Synthetic Expert"}, "accessed": "2025-01-15 12:00:00"}


def main():
    tid = "1800000000000000101"
    write_json(f"twitter/tweet_{tid}.json", tweet_with_quote_and_media(), indent=1)
    write_json("tweet_response.json", tweet_response_fixture(), indent=1)
    timeline_html()
    write_json("yt/list_channels.json", yt_channels(), indent=1)
    write_json("yt/search_result.json", yt_search(), indent=1)
    write_json("yt/list_playlist_items.json", yt_playlist(), indent=1)
    fp_html()
    keepables_ctx()


if __name__ == "__main__":
    main()
