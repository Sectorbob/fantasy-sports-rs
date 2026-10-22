# fantasy-sports-cache

An HTTP `GET` client with a file system cache. It is the layer the Sleeper, Yahoo and
Fleaflicker clients in this workspace are built on, and is usable on its own.

- JSON or XML bodies, deserialized into your own `serde` types.
- Responses are stored on disk keyed by the SHA-256 of the request URL, with a time to live
  (24 hours by default). A hit never touches the network; `force_update` bypasses the cache.
- Only `200` responses are cached. `404` becomes `Error::NotFound`; any other status becomes
  `Error::ClientError { status, body }` so callers can read an API's error payload.
- Optional bearer token per request (not part of the cache key).
- A request counter, which makes "was this served from the cache?" easy to test.

```rust,no_run
use fantasy_sports_cache::{Client, RequestContext};
use serde_json::Value;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = Client::new("https://api.example.com").with_cache("/tmp/my-cache")?;
let value: Value = client
    .build_request(RequestContext { url: "/things/1".into(), filters: None }, false, None)
    .send()
    .await?;
# Ok(())
# }
```

## Development

```shell
cargo test -p fantasy-sports-cache
cargo llvm-cov -p fantasy-sports-cache --html   # coverage, needs cargo-llvm-cov
```
