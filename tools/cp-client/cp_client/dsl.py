"""Helpers for Command Post's ``search_text`` query DSL.

Observed in captures across ``fetch_replays``, ``fetch_activity``,
``fetch_users``, ``fetch_nicks``:

    ::uploader:bikerushownz
    ::player:bike-RUsh+ownz+
    ::user_id:122
    ::nick-name:Skiep,nick-type:1
    ::id:122

Format: ``::key:value[,key:value...]``. Server-side field names are
**not** consistently snake_case or kebab-case (``nick-name`` uses a
dash, ``user_id`` uses an underscore), so this helper doesn't mangle
them — pass them through verbatim.

Spaces in values are encoded as ``+`` (URL-encoding-style) inside the
multipart text part. That's a CP convention; this helper doesn't apply
it automatically — if your value contains spaces, decide whether to
``+``-encode them based on what the server expects for that field.
"""
from __future__ import annotations

from typing import Any


def search_dsl(*filters: tuple[str, Any]) -> str:
    """Build a CP ``search_text`` string from ``(key, value)`` pairs.

    >>> search_dsl(("uploader", "bikerushownz"))
    '::uploader:bikerushownz'
    >>> search_dsl(("nick-name", "Skiep"), ("nick-type", 1))
    '::nick-name:Skiep,nick-type:1'
    >>> search_dsl()
    ''
    """
    if not filters:
        return ""
    return "::" + ",".join(f"{k}:{v}" for k, v in filters)
