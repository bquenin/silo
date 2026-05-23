"""CLI: ``python -m cp_client <endpoint> [k=v ...] [options]``

Examples
--------
    # cheapest call — just confirms auth works
    python -m cp_client ping

    # bootstrap snapshot, raw envelope so you can see checksum/status
    python -m cp_client get_all_info --raw

    # paginated replay catalogue page 1
    python -m cp_client fetch_replays page=1

    # resolve nicks to user_ids
    python -m cp_client fetch_nicks nicks=TsuG,Cranium

    # single-replay detail (includes the download `url` field)
    python -m cp_client fetch_replay_detail 137

    # download a replay's .KWReplay bytes
    python -m cp_client download_replay 137 --output bro_137.kwreplay

    # arbitrary endpoint not in the wrapper list
    python -m cp_client call some_new_endpoint foo=bar

Credentials come from ``CP_USERNAME`` and ``CP_PASSWORD`` in the environment
(plus optional ``CP_TOKEN`` if it diverges from password, and ``CP_BASE_URL``
to override the host — useful if you're pointing at a staging instance).
"""
from __future__ import annotations

import argparse
import json
import logging
import sys

from .endpoints import CommandPostAPI, ENDPOINTS


def _parse_kv(items: list[str]) -> dict[str, str]:
    out: dict[str, str] = {}
    for item in items:
        if "=" not in item:
            raise SystemExit(f"bad field {item!r} — expected key=value")
        key, _, value = item.partition("=")
        if not key:
            raise SystemExit(f"bad field {item!r} — empty key")
        out[key] = value
    return out


def _make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="cp-client",
        description="Command Post API client (cgf-uploads.net)",
    )
    parser.add_argument(
        "endpoint",
        help=f"endpoint name ({', '.join(ENDPOINTS)}) or 'call' for an arbitrary one",
    )
    parser.add_argument(
        "fields",
        nargs="*",
        help="additional fields as key=value (first positional becomes the "
        "endpoint name when 'call' is used)",
    )
    parser.add_argument("--no-cache", action="store_true", help="don't send the cached response_checksum")
    parser.add_argument("--raw", action="store_true", help="print the whole envelope, not just .output")
    parser.add_argument("--asset", action="store_true", help="treat 'endpoint' as an asset path and dump bytes to stdout")
    parser.add_argument(
        "--output", "-o", default=None,
        help="for download_replay: write bytes to this path instead of stdout",
    )
    parser.add_argument(
        "--base-url",
        default=None,
        help="override base URL (defaults to CP_BASE_URL env or https://cgf-uploads.net)",
    )
    parser.add_argument("-v", "--verbose", action="store_true", help="log requests at DEBUG")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _make_parser().parse_args(argv)

    if args.verbose:
        logging.basicConfig(
            level=logging.DEBUG,
            format="%(asctime)s %(levelname)s %(name)s: %(message)s",
        )

    overrides: dict = {"cache_checksums": not args.no_cache}
    if args.base_url:
        overrides["base_url"] = args.base_url

    try:
        client = CommandPostAPI.from_env(**overrides)
    except RuntimeError as e:
        print(str(e), file=sys.stderr)
        return 2

    with client:
        if args.asset:
            data = client.get_asset(args.endpoint)
            # Binary safe write on Windows: use the underlying buffer.
            sys.stdout.buffer.write(data)
            return 0

        if args.endpoint == "call":
            if not args.fields:
                print("call requires <endpoint> as the next positional", file=sys.stderr)
                return 2
            custom_endpoint = args.fields[0]
            fields = _parse_kv(args.fields[1:])
            env = client.call(custom_endpoint, fields=fields)
        elif args.endpoint == "download_replay":
            if not args.fields or "=" in args.fields[0]:
                print("download_replay requires a positional <replay_id>", file=sys.stderr)
                return 2
            replay_id = args.fields[0]
            content, detail = client.download_replay(replay_id)
            if args.output:
                with open(args.output, "wb") as f:
                    f.write(content)
                print(
                    f"wrote {len(content)} bytes to {args.output} "
                    f"(title={detail.get('title')!r}, url={detail.get('url')})",
                    file=sys.stderr,
                )
            else:
                sys.stdout.buffer.write(content)
            return 0
        elif args.endpoint == "fetch_replay_detail":
            if not args.fields or "=" in args.fields[0]:
                print("fetch_replay_detail requires a positional <replay_id>", file=sys.stderr)
                return 2
            detail = client.fetch_replay_detail(args.fields[0])
            json.dump(detail, sys.stdout, indent=2, default=str, ensure_ascii=False)
            sys.stdout.write("\n")
            return 0
        elif args.endpoint in ENDPOINTS:
            fields = _parse_kv(args.fields)
            method = getattr(client, args.endpoint)
            env = method(**fields)
        else:
            print(
                f"unknown endpoint {args.endpoint!r}. "
                f"Known: {', '.join(ENDPOINTS)} (or use 'call <name>' for arbitrary).",
                file=sys.stderr,
            )
            return 2

    payload = env.raw if args.raw else env.output
    json.dump(payload, sys.stdout, indent=2, default=str, ensure_ascii=False)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
