use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const YOUTUBE_API_V3_BASE_URL: &str = "https://www.googleapis.com/youtube/v3";

// const MAX_PAGE_RESULTS: u32 = 50; // Maximum results per page for YouTube API
const MAX_PAGE_RESULTS_STR: &str = "50"; // String representation for query parameters

const SEARCH_CREDITS: usize = 100; // Credits consumed for search operations
const CHANNEL_LIST_CREDITS: usize = 1; // Credits consumed for channel-list operations

pub struct YoutubeClient {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    credits_consumed: AtomicUsize,
}
impl YoutubeClient {
    pub fn new<E: Into<String>, F: Into<String>>(api_key: E, base_url_override: Option<F>) -> Self {
        YoutubeClient {
            api_key: api_key.into(),
            base_url: base_url_override
                .map(Into::into)
                .unwrap_or(YOUTUBE_API_V3_BASE_URL.to_string()),
            client: reqwest::Client::new(),
            credits_consumed: AtomicUsize::new(0),
        }
    }

    #[allow(dead_code)]
    pub async fn search_channels(
        &self,
        query: &str,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let url = format!("{}/search", self.base_url);
        log::debug!("Searching YouTube channels with query: {}", query);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("part", "snippet"),
                ("type", "channel"),
                ("q", query),
                ("key", &self.api_key),
                // ("maxResults", "10"), // Adjust max results as needed
            ])
            .send()
            .await?;

        if response.status().is_success() {
            self.credits_consumed
                .fetch_add(SEARCH_CREDITS, Ordering::SeqCst);
            Ok(response.json().await?)
        } else {
            Err(format!("Failed to search YouTube channels: {}", response.status()).into())
        }
    }

    #[allow(dead_code)]
    pub async fn search_channel_videos(
        &self,
        channel_id: &str,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let url = format!("{}/search", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("part", "snippet"),
                ("type", "video"),
                ("channelId", channel_id),
                ("order", "date"),
                ("maxResults", "10"),
                ("key", &self.api_key),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            self.credits_consumed
                .fetch_add(SEARCH_CREDITS, Ordering::SeqCst);
            Ok(response.json().await?)
        } else {
            Err(format!(
                "Failed to fetch YouTube channel videos: {}",
                response.status()
            )
            .into())
        }
    }

    pub async fn list_channels(
        &self,
        channel_ids: Vec<String>,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let url = format!("{}/channels", self.base_url);
        let mut query_params = vec![
            (
                "part",
                "snippet,contentDetails,statistics,status,topicDetails",
            ),
            ("maxResults", MAX_PAGE_RESULTS_STR),
            ("key", &self.api_key),
        ];
        let channel_ids = channel_ids.join(",");
        if !channel_ids.is_empty() {
            query_params.push(("id", channel_ids.as_str()));
        }

        let response = match self.client.get(&url).query(&query_params).send().await {
            Ok(response) => response,
            Err(err) => {
                log::error!("Error sending request to YouTube API: {}", err);
                return Err(err.into());
            }
        };

        if response.status().is_success() {
            self.credits_consumed
                .fetch_add(CHANNEL_LIST_CREDITS, Ordering::SeqCst);
            Ok(response.json().await?)
        } else {
            Err(format!("Failed to list YouTube playlists: {}", response.status()).into())
        }
    }

    pub async fn list_playlist_items(
        &self,
        playlist_id: &str,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let url = format!("{}/playlistItems", self.base_url);
        let response = match self
            .client
            .get(&url)
            .query(&[
                ("part", "snippet,contentDetails"),
                ("playlistId", playlist_id),
                ("maxResults", MAX_PAGE_RESULTS_STR),
                ("key", &self.api_key),
            ])
            .send()
            .await
        {
            Ok(response) => response,
            Err(err) => {
                log::error!("Error sending request to YouTube API: {}", err);
                return Err(err.into());
            }
        };

        if response.status().is_success() {
            self.credits_consumed
                .fetch_add(CHANNEL_LIST_CREDITS, Ordering::SeqCst);
            Ok(response.json().await?)
        } else {
            Err(format!(
                "Failed to list YouTube playlist items: {}",
                response.status()
            )
            .into())
        }
    }

    pub fn credits_consumed(&self) -> usize {
        self.credits_consumed.load(Ordering::SeqCst)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]

pub enum Response {
    #[serde(rename = "youtube#searchListResponse")]
    SearchListResponse {
        etag: String,
        #[serde(rename = "nextPageToken")]
        next_page_token: Option<String>,
        #[serde(rename = "regionCode")]
        region_code: RegionCode,
        #[serde(rename = "pageInfo")]
        page_info: PageInfo,
        items: Vec<Item>,
    },
    #[serde(rename = "youtube#channelListResponse")]
    ChannelListResponse {
        etag: String,
        #[serde(rename = "nextPageToken")]
        next_page_token: Option<String>,
        // #[serde(rename = "regionCode")]
        // region_code: RegionCode,
        #[serde(rename = "pageInfo")]
        page_info: PageInfo,
        items: Vec<Item>,
    },
    #[serde(rename = "youtube#playlistItemListResponse")]
    PlaylistItemListResponse {
        etag: String,
        #[serde(rename = "nextPageToken")]
        next_page_token: Option<String>,
        #[serde(rename = "pageInfo")]
        page_info: PageInfo,
        items: Vec<Item>,
    },
    #[serde(untagged)]
    Other {
        kind: String,
        etag: String,
        #[serde(rename = "nextPageToken")]
        next_page_token: Option<String>,
        #[serde(rename = "regionCode")]
        region_code: RegionCode,
        #[serde(rename = "pageInfo")]
        page_info: PageInfo,
        items: Vec<Item>,
    },
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total_results: u32,
    pub results_per_page: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum RegionCode {
    US,
    CA,
    GB,
    // Add more region codes as needed
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub kind: ItemKind,
    pub etag: String,
    pub id: ItemId,
    pub snippet: Option<Snippet>,
    pub content_details: Option<ContentDetails>,
    // pub statistics: Option<Statistics>,
    // pub status: Option<ItemStatus>,
    // pub topic_details: Option<TopicDetails>,
}
impl Item {
    pub fn video_id(&self) -> Option<String> {
        match self.kind {
            ItemKind::SearchResult => todo!(),
            ItemKind::Channel => todo!(),
            ItemKind::PlaylistItem => {
                if let Some(content_details) = &self.content_details {
                    if let Some(video_id) = &content_details.video_id {
                        Some(video_id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ItemKind::Video => Some(self.id.id()),
            ItemKind::Other => None,
        }
    }
    pub fn is_search_result(&self) -> bool {
        matches!(self.kind, ItemKind::SearchResult)
    }
    pub fn is_channel(&self) -> bool {
        matches!(self.kind, ItemKind::Channel)
    }
    pub fn is_playlist_item(&self) -> bool {
        matches!(self.kind, ItemKind::PlaylistItem)
    }
    pub fn title(&self) -> String {
        self.snippet
            .as_ref()
            .map_or_else(|| String::new(), |snippet| snippet.title.clone())
    }
    pub fn description(&self) -> String {
        self.snippet
            .as_ref()
            .map_or_else(|| String::new(), |snippet| snippet.description.clone())
    }
    pub fn link(&self) -> Option<String> {
        match self.kind {
            ItemKind::SearchResult => {
                match &self.id {
                    ItemId::Obj {
                        kind: _,
                        channel_id,
                        video_id,
                    } => {
                        if let Some(video_id) = video_id {
                            return Some(format!("https://www.youtube.com/watch?v={video_id}"));
                        }
                        if let Some(channel_id) = channel_id {
                            return Some(format!("https://www.youtube.com/channel/{channel_id}"));
                        }
                    }
                    ItemId::Str(id) => {
                        return Some(format!("https://www.youtube.com/TBD2?v={id}"));
                    }
                }
                Some(format!("https://www.youtube.com/TBD1?v={}", self.id.id()))
            }
            ItemKind::Channel => {
                if let Some(snippet) = &self.snippet {
                    if let Some(custom_url) = &snippet.custom_url {
                        return Some(format!("https://www.youtube.com/{custom_url}"));
                    }
                    if let Some(channel_id) = &snippet.channel_id {
                        return Some(format!("https://www.youtube.com/channel/{channel_id}"));
                    }
                }
                Some(format!("https://www.youtube.com/channel/{}", self.id.id()))
            }
            ItemKind::PlaylistItem => {
                if let Some(content_details) = &self.content_details {
                    if let Some(video_id) = &content_details.video_id {
                        return Some(format!("https://www.youtube.com/watch?v={video_id}"));
                    }
                }
                None
            }
            ItemKind::Video => Some(format!("https://www.youtube.com/watch?v={}", self.id.id())),
            ItemKind::Other => None,
        }
    }
    pub fn published_at(&self) -> Option<DateTime<Utc>> {
        match self.kind {
            ItemKind::Other => None,
            _ => {
                if let Some(snippet) = &self.snippet {
                    Some(snippet.published_at)
                } else {
                    None
                }
            }
        }
    }
    pub fn uploads_playlist_id(&self) -> Option<String> {
        if let Some(content_details) = &self.content_details {
            if let Some(related_playlists) = &content_details.related_playlists {
                related_playlists.uploads.clone()
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ItemKind {
    #[serde(rename = "youtube#searchResult")]
    SearchResult,
    #[serde(rename = "youtube#channel")]
    Channel,
    #[serde(rename = "youtube#playlistItem")]
    PlaylistItem,
    #[serde(rename = "youtube#video")]
    Video,
    #[serde(untagged)]
    Other,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentDetails {
    pub related_playlists: Option<RelatedPlaylists>,
    pub video_id: Option<String>,
    pub video_published_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RelatedPlaylists {
    pub likes: Option<String>,
    pub uploads: Option<String>,
    pub watch_history: Option<String>,
    pub watch_later: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    pub view_count: Option<String>,
    pub subscriber_count: Option<String>,
    pub hidden_subscriber_count: Option<bool>,
    pub video_count: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemStatus {
    pub privacy_status: String,
    pub is_linked: bool,
    pub long_uploads_status: String,
    pub made_for_kids: bool,
    // pub publish_at: Option<DateTime<Utc>>,
    // pub license: Option<String>,
    // pub embeddable: Option<bool>,
    // pub public_stats_viewable: Option<bool>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TopicDetails {
    pub topic_ids: Vec<String>,
    pub topic_categories: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ItemId {
    Obj {
        kind: ItemKind,
        channel_id: Option<String>,
        video_id: Option<String>,
    },
    Str(String),
}
impl ItemId {
    pub fn id(&self) -> String {
        match self {
            ItemId::Obj {
                video_id,
                channel_id,
                ..
            } => {
                if let Some(video_id) = video_id {
                    video_id.clone()
                } else if let Some(channel_id) = channel_id {
                    channel_id.clone()
                } else {
                    "".to_string()
                }
            }
            ItemId::Str(id) => id.clone(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub published_at: DateTime<Utc>,
    pub channel_id: Option<String>,
    pub title: String,
    pub description: String,
    pub thumbnails: Thumbnails,
    pub channel_title: Option<String>,
    pub live_broadcast_content: Option<String>,
    pub publish_time: Option<DateTime<Utc>>,
    pub custom_url: Option<String>,
    pub country: Option<String>,
    pub resource_id: Option<ItemId>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnails {
    pub default: Thumbnail,
    pub medium: Thumbnail,
    pub high: Thumbnail,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YoutubeErrorResponse {
    pub error: YoutubeRequestError,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YoutubeRequestError {
    pub message: String,
    pub code: u16,
    errors: Vec<YoutubeError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YoutubeError {
    pub message: String,
    pub domain: String,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use std::fs;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

    use crate::yt::{Item, Response, YoutubeClient};

    #[tokio::test]
    async fn test_search_channels_happy_path() {
        // 1. Arrange: Start a MockServer
        let mock_server = MockServer::start().await;

        // 2. Arrange: Define the mock behavior
        setup_mock("/search", "testdata/yt/search_result.json")
            .mount(&mock_server)
            .await;
        // 3. Act: Create a YoutubeClient and perform the search
        let client = YoutubeClient::new("", Some(mock_server.uri()));
        let response = client.search_channels("Big Blue Banter").await.unwrap();

        // 4. Assert: Check the response
        if let Response::SearchListResponse {
            etag: _,
            next_page_token: _,
            region_code: _,
            page_info: _,
            items,
        } = response
        {
            assert_eq!(items.len(), 5);
            let bbb_channel = items
                .iter()
                .find(|item| item.is_search_result() && item.title() == "Big Blue Banter");
            assert!(
                bbb_channel.is_some(),
                "Big Blue Banter channel not found in search results"
            );
            let bbb_channel = bbb_channel.unwrap();
            if let Some(snippet) = &bbb_channel.snippet {
                assert_eq!(snippet.title, "Big Blue Banter");
                assert_eq!(
                    snippet.channel_id,
                    Some("UCJ5JxdXQ6xh48GL8kGQJT0w".to_string())
                );
                assert_eq!(snippet.channel_title, Some("Big Blue Banter".to_string()));
                assert_eq!(
                    snippet.description,
                    "Welcome to Big Blue Banter! Dan Schneier and Nick Falato of the Big Blue Banter podcast are here to break down All-22 game ..."
                );
                assert_eq!(
                    snippet.published_at,
                    DateTime::parse_from_rfc3339("2021-02-13T23:01:14Z").unwrap()
                );
            } else {
                panic!("Expected SearchResult but got {:?}", bbb_channel);
            }
        } else {
            panic!("Expected SearchListResponse but got {:?}", response);
        }
    }

    #[tokio::test]
    async fn test_list_channels_happy_path() {
        // 1. Arrange: Start a MockServer
        let mock_server = MockServer::start().await;

        // 2. Arrange: Define the mock behavior
        setup_mock("/channels", "testdata/yt/list_channels.json")
            .mount(&mock_server)
            .await;
        // 3. Act: Create a YoutubeClient and perform the search
        let client = YoutubeClient::new("", Some(mock_server.uri()));
        assert_eq!(0, client.credits_consumed());
        let response = client
            .list_channels(vec![
                String::from("UC2uN7halHCl7g2GW_r7p6DA"),
                String::from("UCF_O9xJbBoQ0D7x8Q8F0mAA"),
                String::from("UC-AfHSMCZfj9_SB__lDiVGQ"),
                String::from("UCJ5JxdXQ6xh48GL8kGQJT0w"),
            ])
            .await
            .unwrap();

        // 4. Assert: Check the response
        assert_eq!(1, client.credits_consumed());
        if let Response::ChannelListResponse {
            etag: _,
            next_page_token: _,
            page_info: _,
            items,
        } = response
        {
            // println!("Items: {:#?}", items);
            assert_eq!(items.len(), 4);
            assert_eq!(
                items.iter().filter(|item| Item::is_channel(item)).count(),
                4
            );
            assert_eq!(
                items
                    .iter()
                    .filter(|item| Item::is_search_result(item))
                    .count(),
                0
            );
            let bbb_channel = items
                .iter()
                .find(|item| item.is_channel() && item.title() == "Big Blue Banter")
                .expect("Big Blue Banter channel not found in search results");
            assert_eq!(bbb_channel.id.id(), "UCJ5JxdXQ6xh48GL8kGQJT0w");
            assert_eq!(
                bbb_channel.link(),
                Some("https://www.youtube.com/@bigbluebanter1108".to_string())
            );
            assert_eq!(
                bbb_channel.uploads_playlist_id(),
                Some("UUJ5JxdXQ6xh48GL8kGQJT0w".to_string())
            );

            let tgi_channel = items
                .iter()
                .find(|item| item.is_channel() && item.title() == "The Giant Insider")
                .expect("The Giant Insider channel not found in search results");
            assert_eq!(tgi_channel.id.id(), "UC-AfHSMCZfj9_SB__lDiVGQ");
            assert_eq!(
                tgi_channel.link(),
                Some("https://www.youtube.com/@thegiantinsider".to_string())
            );
            assert_eq!(
                tgi_channel.uploads_playlist_id(),
                Some("UU-AfHSMCZfj9_SB__lDiVGQ".to_string())
            );

            let gcoll_channel = items
                .iter()
                .find(|item| item.is_channel() && item.title() == "Giants Collective")
                .expect("Giants Collective channel not found in search results");
            assert_eq!(gcoll_channel.id.id(), "UC2uN7halHCl7g2GW_r7p6DA");
            assert_eq!(
                gcoll_channel.link(),
                Some("https://www.youtube.com/@giantscollective".to_string())
            );
            assert_eq!(
                gcoll_channel.uploads_playlist_id(),
                Some("UU2uN7halHCl7g2GW_r7p6DA".to_string())
            );
        } else {
            panic!("Expected ChannelListResponse but got {:?}", response);
        }
    }

    #[tokio::test]
    async fn test_list_playlist_items_happy_path() {
        // 1. Arrange: Start a MockServer
        let mock_server = MockServer::start().await;

        // 2. Arrange: Define the mock behavior
        setup_mock("/playlistItems", "testdata/yt/list_playlist_items.json")
            .mount(&mock_server)
            .await;
        // 3. Act: Create a YoutubeClient and perform the search
        let client = YoutubeClient::new("", Some(mock_server.uri()));
        assert_eq!(0, client.credits_consumed());
        let response = client
            .list_playlist_items("UUJ5JxdXQ6xh48GL8kGQJT0w")
            .await
            .unwrap();

        // 4. Assert: Check the response
        assert_eq!(1, client.credits_consumed());
        if let Response::PlaylistItemListResponse {
            etag: _,
            next_page_token: _,
            page_info: _,
            items,
        } = response
        {
            // println!("PageInfo: {:#?}", page_info);
            println!("Items: {:#?}", items);
            assert_eq!(items.len(), 25);
            assert_eq!(
                items
                    .iter()
                    .filter(|item| Item::is_playlist_item(item))
                    .count(),
                25
            );
            assert_eq!(
                items
                    .iter()
                    .filter(|item| Item::is_search_result(item))
                    .count(),
                0
            );
        } else {
            panic!("Expected PlaylistItemsListResponse but got {:?}", response);
        }
    }

    fn setup_mock(path: &str, data_file: &str) -> Mock {
        Mock::given(matchers::method("GET"))
            .and(matchers::path(path))
            .respond_with(
                ResponseTemplate::new(200)
                    .append_header("content-type", "application/json; charset=UTF-8")
                    .append_header("vary", "X-Origin")
                    .append_header("vary", "Referer")
                    .append_header("vary", "Origin,Accept-Encoding")
                    .append_header("date", "Sun, 27 Jul 2025 05:14:15 GMT")
                    .append_header("server", "scaffolding on HTTPServer2")
                    .append_header("x-xss-protection", 0)
                    .append_header("x-frame-options", "SAMEORIGIN")
                    .append_header("x-content-type-options", "nosniff")
                    .append_header("accept-ranges", "none")
                    .append_header(
                        "alt-svc",
                        "h3=\":443\"; ma=2592000,h3-29=\":443\"; ma=2592000",
                    )
                    .set_body_string(
                        fs::read_to_string(data_file).expect("Failed to read mock data file"),
                    ),
            )
    }

    #[allow(dead_code, unused_variables)]
    fn setup_mock_with_error(path: &str, code: u16, msg: &str, domain: &str, reason: &str) -> Mock {
        let msg = r#"The request cannot be completed because you have exceeded your \u003ca href=\"/youtube/v3/getting-started#quota\"\u003equota\u003c/a\u003e."#;
        let domain = "youtube.quota";
        let reason = "quotaExceeded";
        Mock::given(matchers::method("GET"))
            .and(matchers::path(path))
            .respond_with(
                ResponseTemplate::new(code)
                    // < vary: X-Origin
                    // < vary: Referer
                    // < vary: Origin,Accept-Encoding
                    .append_header("content-type", "application/json; charset=UTF-8")
                    // < date: Wed, 23 Jul 2025 18:33:10 GMT
                    // < server: scaffolding on HTTPServer2
                    // < x-xss-protection: 0
                    // < x-frame-options: SAMEORIGIN
                    // < x-content-type-options: nosniff
                    // < accept-ranges: none
                    // < alt-svc: h3=":443"; ma=2592000,h3-29=":443"; ma=2592000
                    .set_body_string(format!(
                        r#"{{
                            "error": {{
                                "code": {code},
                                "message": "{msg}",
                                "errors": [
                                {{
                                    "message": "{msg}",
                                    "domain": "{domain}",
                                    "reason": "{reason}"
                                }}
                                ]
                            }}
                        }}"#
                    )),
            )
    }
}
