"""Wrappers for every Command Post endpoint observed in captures.

Field names below are confirmed against the actual wire (multipart parts
extracted from ``scratch/cp-capture.mitm``), not guesses. Where multiple
modes were observed for a single endpoint (e.g. ``fetch_stats`` has a
list mode and a per-match mode), they get separate methods.

Common shared patterns:

* **Paginated list** — ``page_cur`` (0-indexed), ``sort_by`` (column
  index; ``-1`` = unsorted/default), ``sort_dir`` (``0`` asc, ``1``
  desc). ``num_per_page`` is server-controlled and echoed back in the
  response envelope.
* **search_text** — query DSL like ``::uploader:bikerushownz``. Use
  :func:`cp_client.dsl.search_dsl` to build it.
* **Two literal ``$``-prefixed fields** appear on ``fetch_nicks`` /
  ``fetch_users``: ``$is_info`` and ``$window_id``. The ``$`` is part
  of the field name. They look like UI-context tags; we mirror what
  the real client sends (both ``1``).
"""
from __future__ import annotations

from typing import Any

from .client import CommandPostClient, CommandPostError, Envelope

#: All endpoints observed in captures.
ENDPOINTS: tuple[str, ...] = (
    "automatcher",
    "fetch_activity",
    "fetch_bounties",
    "fetch_ladders",
    "fetch_map_bundles",
    "fetch_maps",
    "fetch_metapacks",
    "fetch_nicks",
    "fetch_polls",
    "fetch_public_chat",
    "fetch_replays",
    "fetch_stats",
    "fetch_users",
    "get_all_info",
    "ping",
)


class CommandPostAPI(CommandPostClient):
    """Typed wrappers around every known endpoint."""

    # ----- session lifecycle ------------------------------------------------

    def ping(self) -> Envelope:
        """Keepalive — server returns an empty body (no envelope)."""
        return self.call("ping")

    def get_all_info(self) -> Envelope:
        """Bootstrap snapshot (~39 KB). No params beyond the identity bundle.

        The logged-in user's profile (``user_id``, ``user_email``,
        ``user_token``) plus all global state that doesn't have its own
        fetcher.
        """
        return self.call("get_all_info")

    # ----- matchmaking ------------------------------------------------------

    def automatcher(self, *, page_cur: int | str = 0, **extra: Any) -> Envelope:
        """Match-making queue poll (718 B, fixed size).

        ``page_cur`` was sent inconsistently in captures (sometimes ``0``,
        sometimes absent). Defaults to ``0`` here.
        """
        fields: dict[str, Any] = {"page_cur": page_cur, **extra}
        return self.call("automatcher", fields=fields)

    # ----- replay corpus ----------------------------------------------------

    def fetch_replays(
        self,
        *,
        page_cur: int | str = 0,
        sort_by: int | str = -1,
        sort_dir: int | str = 0,
        search_text: str = "",
        **extra: Any,
    ) -> Envelope:
        """Paginated replay catalogue.

        ``search_text`` uses the DSL — e.g. ``::uploader:bikerushownz``
        (see :func:`cp_client.dsl.search_dsl`). Pass ``""`` for unfiltered.

        Response envelope carries ``total_count``, ``page_cur``,
        ``page_max``, ``num_per_page``, plus ``search_filters`` and
        ``sort_filters`` (the UI's available filter / sort options).

        Each output record has 33 fields including: ``id``, ``data``
        (nested replay metadata dict), ``uploader_id``, ``date_uploaded``,
        ``size``, ``title``, ``match_format``, ``summary``, ``duration``,
        ``map_path``, ``map_code``, ``patch_code``, ``gs_id``, ``seed_id``,
        ``tournament_id``, ``is_disputed``, ``is_private``, ``force_public``,
        ``is_bookmarked``, ``match_id``, ``uploader_name``,
        ``uploader_avatar``, ``downloads``, ``likes``, ``comments``,
        ``is_liked``, ``downloaders`` (nested list of users who downloaded).

        The list-mode response does **not** include the binary download
        ``url``. To get that, call :meth:`fetch_replay_detail` (which uses
        the ``type=4`` single-replay mode) or :meth:`download_replay`.
        """
        fields: dict[str, Any] = {
            "page_cur": page_cur,
            "sort_by": sort_by,
            "sort_dir": sort_dir,
            "search_text": search_text,
            **extra,
        }
        return self.call("fetch_replays", fields=fields)

    def fetch_replay_detail(self, replay_id: int | str, **extra: Any) -> dict:
        """Single-replay lookup — the only way to get the download ``url``.

        Wraps ``fetch_replays.php`` in its ``type=4`` mode (different from
        the paginated list). The response is a 1-element list; we unwrap
        and return the record directly. Notably, this record adds a
        ``url`` field that points at
        ``https://cgf-uploads.net/production/public/replays/<filename>.KWReplay``
        — a plain octet-stream GET, no auth on the asset itself.

        Use :meth:`download_replay` for the convenience download.
        """
        fields: dict[str, Any] = {"type": 4, "replay_id": replay_id, **extra}
        # This wrapper promises a record, not an Envelope. A checksum-only
        # response cannot satisfy that promise without a cached record.
        env = self.call("fetch_replays", fields=fields, use_cached_checksum=False)
        out = env.output
        if isinstance(out, list) and out:
            return out[0]
        if isinstance(out, dict):
            return out
        raise CommandPostError(
            f"fetch_replay_detail({replay_id}) returned empty output", envelope=env
        )

    def download_replay(
        self,
        replay_id: int | str,
        *,
        detail: dict | None = None,
    ) -> tuple[bytes, dict]:
        """Download the raw ``.KWReplay`` bytes for ``replay_id``.

        Two-step under the hood:

        1. ``fetch_replays type=4 replay_id=<id>`` → returns the record
           with a populated ``url`` field. Skipped if ``detail`` is passed
           (already-known record from a previous :meth:`fetch_replay_detail`
           call).
        2. ``GET <url>`` → raw bytes. The asset endpoint at
           ``/production/public/replays/<filename>`` returns
           ``application/octet-stream`` without any further auth on the
           GET itself.

        Returns ``(bytes, detail_record)``. The detail record's ``size``
        field should match ``len(bytes)``.
        """
        if detail is None:
            detail = self.fetch_replay_detail(replay_id)
        url = detail.get("url")
        if not url:
            raise CommandPostError(
                f"replay {replay_id} record has no download url "
                "(may be private or removed). Fields: "
                + ", ".join(sorted(detail.keys()))
            )
        # Server returns the URL with the raw replay filename (which can
        # contain ``#``, spaces, brackets, etc.). The ``#`` in particular
        # is a URL fragment separator and ``requests`` will silently drop
        # everything after it unless we percent-encode the path first.
        # We only re-encode the path/filename — scheme + host stay as-is.
        import re
        from urllib.parse import quote, urlsplit, urlunsplit
        parts = urlsplit(url, allow_fragments=False)
        # Keep existing escapes (including encoded slashes) intact and encode
        # bare percent signs before quoting raw filename characters.
        path = re.sub(r"%(?![0-9a-fA-F]{2})", "%25", parts.path)
        encoded_path = quote(path, safe="/%")
        safe_url = urlunsplit(
            (parts.scheme, parts.netloc, encoded_path, parts.query, "")
        )
        resp = self.session.get(safe_url, timeout=self.timeout)
        resp.raise_for_status()
        return resp.content, detail

    # ----- user directory ---------------------------------------------------

    def fetch_users(
        self,
        *,
        users_search_text: str | None = None,
        all_users: bool = False,
        is_info: int | str = 1,
        window_id: int | str = 1,
        **extra: Any,
    ) -> Envelope:
        """User lookup.

        Two modes (mutually exclusive, ``users_search_text`` wins):

        * **Targeted** — pass ``users_search_text="::user_id:122"`` (or
          another DSL filter). Returns matching user(s). Note the field
          name is ``users_search_text``, not ``search_text``.
        * **Full dump** — pass ``all_users=True``. ~3.6 MB; be polite.

        ``$is_info`` and ``$window_id`` are sent in targeted mode to
        mirror the real client.
        """
        fields: dict[str, Any] = {}
        if all_users:
            fields["all_users"] = "1"
        elif users_search_text is not None:
            fields["users_search_text"] = users_search_text
            fields["$is_info"] = is_info
            fields["$window_id"] = window_id
        else:
            raise ValueError(
                "fetch_users requires either users_search_text=... or all_users=True"
            )
        fields.update(extra)
        return self.call("fetch_users", fields=fields)

    def fetch_activity(self, *, search_text: str, **extra: Any) -> Envelope:
        """Per-player match history (~47 KB for an active player).

        ``search_text`` uses the DSL — e.g. ``::player:NickName``.
        Spaces in nicks are encoded as ``+`` on the wire (the real
        client sends ``::player:bike-RUsh+ownz+`` for the nick
        ``bike-RUsh ownz``).
        """
        fields: dict[str, Any] = {"search_text": search_text, **extra}
        return self.call("fetch_activity", fields=fields)

    def fetch_stats(
        self,
        *,
        page_cur: int | str = 0,
        sort_by: int | str = -1,
        sort_dir: int | str = 0,
        search_text: str = "",
        **extra: Any,
    ) -> Envelope:
        """List-mode stats (~6 KB). For a single match, use
        :meth:`fetch_match_stats`."""
        fields: dict[str, Any] = {
            "page_cur": page_cur,
            "sort_by": sort_by,
            "sort_dir": sort_dir,
            "search_text": search_text,
            **extra,
        }
        return self.call("fetch_stats", fields=fields)

    def fetch_match_stats(
        self,
        *,
        seed_id: int | str,
        gs_id: int | str,
        match_id: int | str | None = None,
        **extra: Any,
    ) -> Envelope:
        """Per-match stats — alternate ``fetch_stats`` mode (~8 KB).

        ``seed_id`` + ``gs_id`` identify a specific match. ``match_id``
        was present in one capture and absent in another, so it's
        optional. Pass it if you have it.
        """
        fields: dict[str, Any] = {"seed_id": seed_id, "gs_id": gs_id}
        if match_id is not None:
            fields["match_id"] = match_id
        fields.update(extra)
        return self.call("fetch_stats", fields=fields)

    def fetch_nicks(
        self,
        *,
        search_text: str,
        is_info: int | str = 1,
        window_id: int | str = 1,
        **extra: Any,
    ) -> Envelope:
        """Resolve in-game nicknames to ``user_id`` (~610 B).

        ``search_text`` DSL examples:

        * ``::nick-name:Skiep,nick-type:1`` — lookup by nick + type
        * ``::id:122`` — reverse lookup (user_id → nick)
        """
        fields: dict[str, Any] = {
            "search_text": search_text,
            "$is_info": is_info,
            "$window_id": window_id,
            **extra,
        }
        return self.call("fetch_nicks", fields=fields)

    # ----- ladders ----------------------------------------------------------

    def fetch_ladders(
        self,
        *,
        ladder_type: int | str = -1,
        page_cur: int | str = 0,
        sort_dir: int | str = 0,
        search_text: str = "",
        **extra: Any,
    ) -> Envelope:
        """Ranked ladder leaderboards (~7 KB).

        ``ladder_type``: ``-1`` for all, ``0`` for first ladder, etc.
        No ``sort_by`` observed in captures — likely fixed by
        ``ladder_type``.
        """
        fields: dict[str, Any] = {
            "ladder_type": ladder_type,
            "page_cur": page_cur,
            "sort_dir": sort_dir,
            "search_text": search_text,
            **extra,
        }
        return self.call("fetch_ladders", fields=fields)

    # ----- maps & packs -----------------------------------------------------

    def fetch_maps(
        self,
        *,
        page_cur: int | str = 0,
        sort_by: int | str = 0,
        sort_dir: int | str = 0,
        **extra: Any,
    ) -> Envelope:
        """Map catalogue (~7 KB).

        Each record: ``id``, ``display_name``, ``num_players``, image
        refs, ``downloads``.
        """
        fields: dict[str, Any] = {
            "page_cur": page_cur,
            "sort_by": sort_by,
            "sort_dir": sort_dir,
            **extra,
        }
        return self.call("fetch_maps", fields=fields)

    def fetch_map_bundles(
        self,
        *,
        page_cur: int | str = 0,
        sort_by: int | str = 0,
        sort_dir: int | str = 0,
        **extra: Any,
    ) -> Envelope:
        """Curated map bundles (~26 KB), e.g. "Nurabsal's Best Of 4v4"."""
        fields: dict[str, Any] = {
            "page_cur": page_cur,
            "sort_by": sort_by,
            "sort_dir": sort_dir,
            **extra,
        }
        return self.call("fetch_map_bundles", fields=fields)

    def fetch_metapacks(self) -> Envelope:
        """Map-pack registry (~2.7 KB). No params beyond the identity bundle."""
        return self.call("fetch_metapacks")

    # ----- community --------------------------------------------------------

    def fetch_bounties(
        self,
        *,
        page_cur: int | str = 0,
        sort_by: int | str = 0,
        sort_dir: int | str = 0,
        **extra: Any,
    ) -> Envelope:
        """Community challenges / bounties (~5.5 KB)."""
        fields: dict[str, Any] = {
            "page_cur": page_cur,
            "sort_by": sort_by,
            "sort_dir": sort_dir,
            **extra,
        }
        return self.call("fetch_bounties", fields=fields)

    def fetch_polls(
        self,
        *,
        page_cur: int | str = 0,
        sort_by: int | str = 0,
        sort_dir: int | str = 0,
        **extra: Any,
    ) -> Envelope:
        """Community polls (~8 KB)."""
        fields: dict[str, Any] = {
            "page_cur": page_cur,
            "sort_by": sort_by,
            "sort_dir": sort_dir,
            **extra,
        }
        return self.call("fetch_polls", fields=fields)

    def fetch_public_chat(self, **extra: Any) -> Envelope:
        """Public chat polling — typically a 49-byte no-op."""
        return self.call("fetch_public_chat", fields=extra)
