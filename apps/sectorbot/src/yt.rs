use chrono::{DateTime, Utc};
use serde::Deserialize;

const YOUTUBE_API_V3_BASE_URL: &str = "https://www.googleapis.com/youtube/v3";

pub struct YoutubeClient {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}
impl YoutubeClient {
    pub fn new<E: Into<String>, F: Into<String>>(api_key: E, base_url_override: Option<F>) -> Self {
        YoutubeClient {
            api_key: api_key.into(),
            base_url: base_url_override
                .map(Into::into)
                .unwrap_or(YOUTUBE_API_V3_BASE_URL.to_string()),
            client: reqwest::Client::new(),
        }
    }

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
            Ok(response.json().await?)
        } else {
            Err(format!("Failed to search YouTube channels: {}", response.status()).into())
        }
    }

    pub async fn fetch_channel_videos(
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
            Ok(response.json().await?)
        } else {
            Err(format!(
                "Failed to fetch YouTube channel videos: {}",
                response.status()
            )
            .into())
        }
    }
}

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
#[serde(tag = "kind", rename_all = "camelCase")]

pub enum Item {
    #[serde(rename = "youtube#searchResult")]
    SearchResult {
        etag: String,
        id: ItemId,
        snippet: Snippet,
    },
    #[serde(untagged)]
    Other {
        kind: String,
        etag: String,
        id: ItemId,
        snippet: Snippet,
    },
}
impl Item {
    pub fn id(&self) -> String {
        match self {
            Item::SearchResult { id, .. } => id.id(),
            Item::Other { id, .. } => id.id(),
        }
    }
    pub fn title(&self) -> String {
        match self {
            Item::SearchResult { snippet, .. } => snippet.title.clone(),
            Item::Other { snippet, .. } => snippet.title.clone(),
        }
    }
    pub fn description(&self) -> String {
        match self {
            Item::SearchResult { snippet, .. } => snippet.description.clone(),
            Item::Other { snippet, .. } => snippet.description.clone(),
        }
    }
    pub fn link(&self) -> String {
        match self {
            Item::SearchResult { id, .. } => {
                if let Some(video_id) = &id.video_id {
                    format!("https://www.youtube.com/watch?v={}", video_id)
                } else if let Some(channel_id) = &id.channel_id {
                    format!("https://www.youtube.com/channel/{}", channel_id)
                } else {
                    String::new()
                }
            }
            Item::Other { id, .. } => {
                if let Some(video_id) = &id.video_id {
                    format!("https://www.youtube.com/watch?v={}", video_id)
                } else if let Some(channel_id) = &id.channel_id {
                    format!("https://www.youtube.com/channel/{}", channel_id)
                } else {
                    String::new()
                }
            }
        }
    }
    pub fn published_at(&self) -> DateTime<Utc> {
        match self {
            Item::SearchResult { snippet, .. } => snippet.published_at,
            Item::Other { snippet, .. } => snippet.published_at,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemId {
    pub kind: String,
    pub channel_id: Option<String>,
    pub video_id: Option<String>,
}
impl ItemId {
    pub fn id(&self) -> String {
        if let Some(video_id) = &self.video_id {
            video_id.clone()
        } else if let Some(channel_id) = &self.channel_id {
            channel_id.clone()
        } else {
            "".to_string()
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub published_at: DateTime<Utc>,
    pub channel_id: String,
    pub title: String,
    pub description: String,
    // pub thumbnails: Thumbnails,
    pub channel_title: String,
    pub live_broadcast_content: String,
    pub publish_time: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use tokio::fs;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::yt::{Item, Response, YoutubeClient};

    #[tokio::test]
    async fn test_search_channels_happy_path() {
        // 1. Arrange: Start a MockServer
        let mock_server = MockServer::start().await;

        // 2. Arrange: Define the mock behavior
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(
                ResponseTemplate::new(200)
                    .append_header("Content-Type", "application/json")
                    .set_body_string(
                        fs::read_to_string("testdata/yt/search_result.json")
                            .await
                            .unwrap(),
                    ),
            )
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
            let bbb_channel = items.iter().find(|item| {
                if let Item::SearchResult { snippet, .. } = item {
                    snippet.channel_title == "Big Blue Banter"
                } else {
                    false
                }
            });
            assert!(
                bbb_channel.is_some(),
                "Big Blue Banter channel not found in search results"
            );
            let bbb_channel = bbb_channel.unwrap();
            if let Item::SearchResult { snippet, .. } = bbb_channel {
                assert_eq!(snippet.title, "Big Blue Banter");
                assert_eq!(snippet.channel_id, "UCJ5JxdXQ6xh48GL8kGQJT0w");
                assert_eq!(snippet.channel_title, "Big Blue Banter");
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
}
