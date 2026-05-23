"""Transport layer for the Command Post API.

All Command Post endpoints share the same wire shape:

    POST https://cgf-uploads.net/production/<endpoint>.php
    Content-Type: multipart/form-data; boundary=...
    body parts: each with `Content-Type: text/plain; charset=utf-8`
                and `Content-Disposition: form-data; name=<field>` (unquoted)
    response: application/json; charset=utf-8
             {"status": 1, "message": "", "output": <payload>,
              "checksum": <int>, ...endpoint-specific paging/filter fields...}

Every request carries a fixed "identity bundle" of ~18 fields: auth
credentials, hardware fingerprint, app version, and presence telemetry.
This client defaults the hardware fields to empty strings so we identify
cleanly as a non-CP client; the caller can override any of them.

The server supports a checksum-based no-op: include the previously seen
``checksum`` as ``response_checksum`` and, if the payload hasn't changed,
the server returns an envelope with ``output`` empty and the same checksum.
This client caches checksums per (endpoint, non-identity-fields) automatically.
"""
from __future__ import annotations

import json
import logging
import os
import platform
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any

import requests
from requests_toolbelt.multipart.encoder import MultipartEncoder

log = logging.getLogger(__name__)

DEFAULT_BASE_URL = "https://cgf-uploads.net"
DEFAULT_PATH_PREFIX = "production"
DEFAULT_ENV_VER = "4.8.1 or later"      # literal string the real client sends
DEFAULT_APP_VER = "1.5.15"              # CP version observed in captures
DEFAULT_APP_CODE = "255"                # build code observed
DEFAULT_LANG = "1"
DEFAULT_USER_AGENT = "tacitus-cp-client/0.1 (+https://github.com/bquenin/tacitus)"
DEFAULT_TIMEOUT = 30.0
EPOCH_NEVER = "0001-01-01 00:00:00"     # what real CP sends for "never happened"
PART_CONTENT_TYPE = "text/plain; charset=utf-8"


def _default_os_ver() -> str:
    """Mirror the ``os_ver`` string the real CP client sends.

    Observed: ``Microsoft Windows NT 10.0.26200.0 x64``.
    """
    system = platform.system()
    if system == "Windows":
        release = platform.version()
        arch = platform.machine()
        arch = "x64" if arch in ("AMD64", "x86_64") else arch.lower()
        return f"Microsoft Windows NT {release}.0 {arch}"
    return f"{system} {platform.release()} {platform.machine()}"


def _now_str() -> str:
    """``YYYY-MM-DD HH:MM:SS`` in local time — matches CP's wire format."""
    return datetime.now().strftime("%Y-%m-%d %H:%M:%S")


@dataclass
class Envelope:
    """Parsed Command Post response envelope.

    Paginated endpoints carry pagination metadata in the envelope (not
    inside ``output``): :attr:`total_count`, :attr:`page_cur`,
    :attr:`page_max`, :attr:`num_per_page`. They're surfaced as
    properties for convenience; the full payload remains in :attr:`raw`.
    """

    status: int
    message: str
    output: Any
    checksum: int | None = None
    raw: dict = field(default_factory=dict)

    @classmethod
    def from_dict(cls, data: dict) -> "Envelope":
        return cls(
            status=int(data.get("status", 0)),
            message=str(data.get("message", "")),
            output=data.get("output"),
            checksum=data.get("checksum"),
            raw=data,
        )

    @property
    def ok(self) -> bool:
        return self.status == 1

    @property
    def unchanged(self) -> bool:
        """True if the server short-circuited via ``response_checksum`` match."""
        return self.ok and self.output in (None, "", [], {}) and self.checksum is not None

    @property
    def total_count(self) -> int | None:
        v = self.raw.get("total_count")
        return int(v) if v is not None else None

    @property
    def page_cur(self) -> int | None:
        v = self.raw.get("page_cur")
        return int(v) if v is not None else None

    @property
    def page_max(self) -> int | None:
        v = self.raw.get("page_max")
        return int(v) if v is not None else None

    @property
    def num_per_page(self) -> int | None:
        v = self.raw.get("num_per_page")
        return int(v) if v is not None else None


class CommandPostError(Exception):
    def __init__(self, message: str, envelope: Envelope | None = None):
        super().__init__(message)
        self.envelope = envelope


class CommandPostClient:
    """Low-level transport: authenticated multipart POST, envelope unwrapping."""

    def __init__(
        self,
        username: str,
        password: str,
        *,
        token: str | None = None,
        # --- transport ---
        base_url: str = DEFAULT_BASE_URL,
        path_prefix: str = DEFAULT_PATH_PREFIX,
        session: requests.Session | None = None,
        timeout: float = DEFAULT_TIMEOUT,
        user_agent: str = DEFAULT_USER_AGENT,
        cache_checksums: bool = True,
        # --- environment / app version ---
        env_ver: str = DEFAULT_ENV_VER,
        os_ver: str | None = None,
        app_ver: str = DEFAULT_APP_VER,
        app_code: str = DEFAULT_APP_CODE,
        lang: str = DEFAULT_LANG,
        # --- hardware fingerprint (default empty -> non-CP identity) ---
        machine_id: str = "",
        proc_id: str = "",
        bios_id: str = "",
        mobo_id: str = "",
        mac_id: str = "",
        # --- presence telemetry ---
        metadata_date: str | None = None,
        last_game_time: str = EPOCH_NEVER,
        game_status: str | int = "0",
        track_presence: bool = True,
    ):
        if not username or not password:
            raise ValueError("username and password are required")
        self.username = username
        self.password = password
        # In captures the ``token`` was identical to ``password`` (a derived
        # session credential). Default to password if not overridden.
        self.token = token if token is not None else password
        self.base_url = base_url.rstrip("/")
        self.path_prefix = path_prefix.strip("/")
        self.timeout = timeout
        self.cache_checksums = cache_checksums
        self._checksums: dict[str, int] = {}
        self.session = session or requests.Session()
        self.session.headers["User-Agent"] = user_agent

        self.env_ver = env_ver
        self.os_ver = os_ver or _default_os_ver()
        self.app_ver = app_ver
        self.app_code = str(app_code)
        self.lang = str(lang)

        self.machine_id = machine_id
        self.proc_id = proc_id
        self.bios_id = bios_id
        self.mobo_id = mobo_id
        self.mac_id = mac_id

        self.metadata_date = metadata_date or _now_str()
        self.last_game_time = last_game_time
        self.game_status = str(game_status)
        self.track_presence = track_presence
        self._started_at = time.monotonic()

    @classmethod
    def from_env(cls, **overrides) -> "CommandPostClient":
        """Construct from ``CP_USERNAME`` / ``CP_PASSWORD`` (+ optional
        ``CP_TOKEN``, ``CP_BASE_URL``).
        """
        username = os.environ.get("CP_USERNAME")
        password = os.environ.get("CP_PASSWORD")
        if not username or not password:
            raise RuntimeError(
                "CP_USERNAME and CP_PASSWORD must be set in the environment"
            )
        token = os.environ.get("CP_TOKEN")
        base_url = os.environ.get("CP_BASE_URL")
        if token is not None and "token" not in overrides:
            overrides["token"] = token
        if base_url is not None and "base_url" not in overrides:
            overrides["base_url"] = base_url
        return cls(username=username, password=password, **overrides)

    # ----- request building -------------------------------------------------

    def _url(self, endpoint: str) -> str:
        endpoint = endpoint.lstrip("/")
        if not endpoint.endswith(".php"):
            endpoint += ".php"
        return f"{self.base_url}/{self.path_prefix}/{endpoint}"

    #: Fields the server treats as identity/presence — excluded from the
    #: checksum cache key so warm calls hit the no-op short-circuit.
    _IDENTITY_KEYS: frozenset[str] = frozenset({
        "username", "password", "token", "env_ver", "os_ver",
        "machine_id", "proc_id", "bios_id", "mobo_id", "mac_id",
        "lang", "app_ver", "app_code",
        "metadata_date", "last_game_time", "game_status",
        "last_activity", "idle_time",
    })

    def _identity_fields(self) -> dict[str, str]:
        """The full ~18-field identity+presence bundle sent on every call."""
        now = _now_str()
        idle = int(time.monotonic() - self._started_at) if self.track_presence else 0
        return {
            "username": self.username,
            "password": self.password,
            "token": self.token,
            "env_ver": self.env_ver,
            "os_ver": self.os_ver,
            "machine_id": self.machine_id,
            "proc_id": self.proc_id,
            "bios_id": self.bios_id,
            "mobo_id": self.mobo_id,
            "mac_id": self.mac_id,
            "lang": self.lang,
            "app_ver": self.app_ver,
            "app_code": self.app_code,
            "metadata_date": self.metadata_date,
            "last_game_time": self.last_game_time,
            "game_status": self.game_status,
            "last_activity": now,
            "idle_time": str(idle),
        }

    def _checksum_key(self, endpoint: str, fields: dict) -> str:
        sub = {
            k: v
            for k, v in fields.items()
            if k not in self._IDENTITY_KEYS and k != "response_checksum"
        }
        return endpoint + "|" + json.dumps(sub, sort_keys=True, default=str)

    # ----- main entry point -------------------------------------------------

    def call(
        self,
        endpoint: str,
        *,
        fields: dict[str, Any] | None = None,
        files: dict[str, tuple] | None = None,
        use_cached_checksum: bool | None = None,
    ) -> Envelope:
        """POST to ``endpoint`` and return the parsed envelope.

        ``fields`` are merged with the identity bundle and sent as multipart
        form data. ``files`` is a ``{name: (filename, fileobj, content_type)}``
        mapping for binary uploads.

        When ``use_cached_checksum`` is true (the default if the client was
        configured with ``cache_checksums=True``), the cached checksum for
        this (endpoint, non-identity fields) is sent as ``response_checksum``
        so the server can short-circuit unchanged payloads.
        """
        fields = dict(fields or {})
        # endpoint-specific fields override identity defaults if names collide
        all_fields = {**self._identity_fields(), **{k: str(v) for k, v in fields.items()}}

        use_cs = self.cache_checksums if use_cached_checksum is None else use_cached_checksum
        if use_cs and "response_checksum" not in all_fields:
            key = self._checksum_key(endpoint, fields)
            if key in self._checksums:
                all_fields["response_checksum"] = str(self._checksums[key])

        url = self._url(endpoint)

        # Real CP sends each part with `Content-Type: text/plain; charset=utf-8`.
        # MultipartEncoder accepts (filename, value, content_type) tuples to
        # control the per-part type; `filename=None` keeps the part as a plain
        # form field (no `filename=` in Content-Disposition).
        parts: list[tuple[str, Any]] = [
            (k, (None, v, PART_CONTENT_TYPE)) for k, v in all_fields.items()
        ]
        if files:
            for name, spec in files.items():
                parts.append((name, spec))
        encoder = MultipartEncoder(fields=parts)

        log.debug(
            "POST %s fields=%s files=%s",
            url,
            sorted(set(all_fields) - self._IDENTITY_KEYS),
            list((files or {}).keys()),
        )

        resp = self.session.post(
            url,
            data=encoder,
            headers={"Content-Type": encoder.content_type},
            timeout=self.timeout,
        )
        resp.raise_for_status()

        if not resp.content:
            return Envelope(status=1, message="", output=None, raw={})

        try:
            data = resp.json()
        except ValueError as e:
            raise CommandPostError(
                f"non-JSON response from {endpoint}: {resp.text[:200]!r}"
            ) from e

        env = Envelope.from_dict(data)
        if env.checksum is not None and use_cs:
            self._checksums[self._checksum_key(endpoint, fields)] = int(env.checksum)

        if not env.ok:
            raise CommandPostError(
                f"{endpoint} returned status={env.status} message={env.message!r}",
                envelope=env,
            )
        return env

    # ----- public asset GETs ------------------------------------------------

    def get_asset(self, path: str) -> bytes:
        """GET a public asset (avatars, faction icons).

        ``path`` may be absolute (``/production/public/...``) or relative
        (``public/profiles/5081/avatar_1714.png`` — the ``/production/``
        prefix is added automatically).
        """
        if path.startswith(("http://", "https://")):
            url = path
        else:
            path = path.lstrip("/")
            if not path.startswith(f"{self.path_prefix}/"):
                path = f"{self.path_prefix}/{path}"
            url = f"{self.base_url}/{path}"
        resp = self.session.get(url, timeout=self.timeout)
        resp.raise_for_status()
        return resp.content

    # ----- lifecycle --------------------------------------------------------

    def clear_checksums(self) -> None:
        self._checksums.clear()

    def close(self) -> None:
        self.session.close()

    def __enter__(self) -> "CommandPostClient":
        return self

    def __exit__(self, *exc) -> None:
        self.close()
