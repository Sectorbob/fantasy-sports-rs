# Test data

Everything in this directory is **synthetic**. The "Example Rovers" team, all people,
Twitter/YouTube accounts, videos, ids and rankings are invented. The JSON/HTML shapes
mirror the real APIs (Twitter syndication, YouTube Data API v3, the FantasyPros ECR page,
Sleeper) so the parsers are exercised realistically without redistributing anyone's content.

Regenerate with (from `sectorbot/`):

```shell
python3 testdata/gen_fixtures.py
```

The output is deterministic. Edit the generator, not the generated files.
