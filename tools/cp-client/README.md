# cp-client

Python client for the closed-source **Command Post** API at
`https://cgf-uploads.net/production/`. Built by reverse-engineering an
mitmproxy capture of the real CP desktop app so [Tacitus](../../README.md)
can interoperate with the same replay corpus, map metadata, and user
directory.

Field names per endpoint are **confirmed against the captured wire** —
not guesses. The multipart body our client produces is byte-identical
to the real CP client's for the same call (modulo the boundary, the
per-call presence timestamps `last_activity` / `idle_time`, and the
`response_checksum` short-circuit token).

## Install (editable)

```powershell
py -m pip install -e tools/cp-client
```

Requires Python 3.10+. Pulls in `requests` and `requests-toolbelt`.

## Credentials

Pick them up from the environment — never commit them:

```powershell
$env:CP_USERNAME = "your-cp-username"
$env:CP_PASSWORD = "your-cp-password-or-session-token"
# Optional:
$env:CP_TOKEN    = "..."   # if your token differs from password (in captures it didn't)
$env:CP_BASE_URL = "..."   # override host
```

In captured traffic, the `password` and `token` fields were identical
(both equal to a derived session credential `ikKbvIYn9R<user_id>`).
`CP_TOKEN` is only needed if that pattern changes.

## CLI

```powershell
# minimal: confirms auth works (empty body)
python -m cp_client ping

# bootstrap snapshot — biggest first-call payload
python -m cp_client get_all_info --raw

# paginated list, default page 0
python -m cp_client fetch_replays

# page 2 of the list, filtered by uploader nick
python -m cp_client fetch_replays page_cur=2 search_text="::uploader:Cranium"

# user lookup (note the field is `users_search_text`, not `search_text`)
python -m cp_client fetch_users users_search_text="::user_id:122"

# arbitrary / not-yet-wrapped endpoint
python -m cp_client call some_new_endpoint foo=bar baz=qux

# download a public asset
python -m cp_client --asset public/images/icon_gdi.png > icon.png
```

Flags:

| Flag | Effect |
|---|---|
| `--raw` | Print the whole envelope (status, message, output, checksum, page metadata) instead of just `output`. |
| `--no-cache` | Don't send the cached `response_checksum`. Forces a full response. |
| `--asset` | Treat the positional as an asset path and dump raw bytes to stdout. |
| `--base-url URL` | Override the host (also via `CP_BASE_URL`). |
| `-v` | Log requests at DEBUG. |

## Library

```python
from cp_client import CommandPostAPI, search_dsl

with CommandPostAPI.from_env() as cp:
    cp.ping()

    # paginated fetch with search DSL
    page = cp.fetch_replays(
        page_cur=0,
        sort_by=0, sort_dir=0,
        search_text=search_dsl(("uploader", "Cranium")),
    )
    print(f"{page.total_count} replays match, showing page "
          f"{page.page_cur}/{page.page_max} ({page.num_per_page}/page)")
    for r in page.output:
        print(r["id"], r["title"], r["uploader_name"])

    # cache is per (endpoint + non-identity fields); second identical call
    # short-circuits via response_checksum
    again = cp.fetch_replays(
        page_cur=0,
        sort_by=0, sort_dir=0,
        search_text=search_dsl(("uploader", "Cranium")),
    )
    assert again.unchanged

    # user lookup — different DSL field name
    me = cp.fetch_users(users_search_text=search_dsl(("user_id", 5081)))

    # match-detail stats
    stats = cp.fetch_match_stats(seed_id=-1633710449, gs_id=2510, match_id=529445)
```

For raw access to an endpoint we haven't wrapped yet:

```python
env = cp.call("some_new_endpoint", fields={"foo": "bar"})
```

## The `search_text` DSL

Several endpoints accept a query DSL in their `search_text` (or
`users_search_text`) field:

```
::key:value[,key:value...]
```

Examples from the wire:

| Endpoint | Example |
|---|---|
| `fetch_replays` | `::uploader:bikerushownz` |
| `fetch_activity` | `::player:bike-RUsh+ownz+` (spaces → `+`) |
| `fetch_users` | `::user_id:122` |
| `fetch_nicks` | `::nick-name:Skiep,nick-type:1` &nbsp;or&nbsp; `::id:122` |

Server field names are not consistently snake/kebab cased — `nick-name`
uses a dash, `user_id` uses an underscore. The `search_dsl()` helper
takes `(key, value)` pairs verbatim:

```python
from cp_client import search_dsl
search_dsl(("uploader", "Cranium"))             # '::uploader:Cranium'
search_dsl(("nick-name", "Skiep"), ("nick-type", 1))
# '::nick-name:Skiep,nick-type:1'
```

For values containing spaces, the real CP client URL-encodes spaces as
`+` inside the multipart text part. This helper doesn't auto-encode —
do that explicitly if a server-side field requires it.

## Endpoint coverage

All 15 endpoints observed in the capture, with **confirmed** field names:

| Endpoint | Confirmed fields | Notes |
|---|---|---|
| `ping` | — | empty 0 B response |
| `get_all_info` | — | bootstrap, ~39 KB |
| `automatcher` | `page_cur` (sometimes) | 718 B fixed |
| `fetch_replays` | `page_cur`, `sort_by`, `sort_dir`, `search_text` | paginated list |
| `fetch_users` | `users_search_text` + `$is_info` + `$window_id`, OR `all_users=1` | targeted vs full-dump modes |
| `fetch_activity` | `search_text` (uses `::player:` DSL) | |
| `fetch_stats` | list mode: `page_cur`, `sort_by`, `sort_dir`, `search_text` | |
| `fetch_stats` (per-match) | `match_id`, `seed_id`, `gs_id` | wrapped as `fetch_match_stats()` |
| `fetch_nicks` | `search_text` + `$is_info` + `$window_id` | DSL filters |
| `fetch_ladders` | `ladder_type`, `page_cur`, `sort_dir`, `search_text` | no `sort_by` seen |
| `fetch_maps` | `page_cur`, `sort_by`, `sort_dir` | |
| `fetch_map_bundles` | `page_cur`, `sort_by`, `sort_dir` | |
| `fetch_metapacks` | — | |
| `fetch_bounties` | `page_cur`, `sort_by`, `sort_dir` | |
| `fetch_polls` | `page_cur`, `sort_by`, `sort_dir` | |
| `fetch_public_chat` | — (just `response_checksum`) | ~50 B no-op |

Common pagination pattern: `page_cur` is 0-indexed, `sort_by` is a
column index (`-1` = unsorted), `sort_dir` is `0`/`1` for asc/desc.
`num_per_page` is server-controlled (echoed in the response envelope,
not sent in the request).

Public asset GETs (no auth, no envelope):
- `/production/public/profiles/<user_id>/avatar_<unix_ts>.{png,jpg,jpeg}`
- `/production/public/images/icon_{gdi,nod,question}.png`

## Wire shape

```
POST https://cgf-uploads.net/production/<endpoint>.php
Content-Type: multipart/form-data; boundary=<uuid>

<each part>
  Content-Type: text/plain; charset=utf-8
  Content-Disposition: form-data; name=<unquoted-field-name>

  <value>
```

**Identity bundle** (sent on every call, 18 fields):

| Field | Default | Source |
|---|---|---|
| `username` / `password` / `token` | from env | required |
| `env_ver` | `"4.8.1 or later"` | literal version-range string |
| `os_ver` | auto-detected | `Microsoft Windows NT 10.0.26200.0 x64` |
| `app_ver` / `app_code` | `"1.5.15"` / `"255"` | observed CP version |
| `lang` | `"1"` | UI locale code |
| `machine_id`, `proc_id`, `bios_id`, `mobo_id`, `mac_id` | `""` | hardware fingerprint — we default empty, override via constructor if the server complains |
| `metadata_date` | client start time | when local metadata was refreshed |
| `last_game_time` | `"0001-01-01 00:00:00"` | when KW was last played |
| `game_status` | `"0"` | KW running state |
| `last_activity`, `idle_time` | auto | derived from client uptime |

**Response envelope**:

```json
{
  "status": 1,
  "message": "",
  "output": <payload>,
  "checksum": 4290514916,

  // paginated endpoints also include:
  "total_count": 27,
  "page_cur": 1,
  "page_max": 2,
  "num_per_page": 20,
  "sort_by": 0, "sort_dir": 0,
  "search_text": "...",
  "search_filters": {...},  // UI's available filter options
  "sort_filters": {...}     // UI's available sort columns
}
```

`status != 1` → `CommandPostError` is raised with the envelope attached.

### Checksum short-circuit

The server returns a `checksum` per payload. If you POST it back as
`response_checksum` and the payload hasn't changed, you get an empty
envelope (`output` absent, same `checksum` echoed). The client caches
checksums per `(endpoint, non-identity fields)` automatically — disable
with `cache_checksums=False` or `--no-cache`. `Envelope.unchanged`
detects the no-op response.

## Be polite

The CP server is community-run. Don't:
- bulk-scrape `fetch_users(all_users=True)` in a tight loop (3.6 MB dump)
- spoof the User-Agent of the real CP client (we identify as `tacitus-cp-client/0.1`)
- DOS via concurrent paginated fetches without backoff

If you're pulling the replay corpus for Tacitus, throttle to ~1 req/s
and cache locally. If a CP maintainer wants us to stop, we stop.

## Ethics / Legality

Client for a community API that's already publicly served. Doesn't
bypass paywalls, doesn't decrypt anything, doesn't include any
proprietary code. We use our own valid credentials. Schema knowledge
was derived from inspecting our own client's traffic with a
self-installed proxy.
