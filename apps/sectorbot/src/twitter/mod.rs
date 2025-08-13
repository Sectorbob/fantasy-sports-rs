pub mod syndication;
// pub mod fix_tweet;
mod data;
#[allow(unused_imports)]
pub use data::{
    MediaDetails, MediaType, Timeline, TimelineContent, TimelineEntry, TimelineResponse,
    TweetResponse, User,
};

pub const FX_TWITTER: &str = "https://fxtwitter.com";
