"""Python client for the Command Post API at cgf-uploads.net.

Reverse-engineered from packet captures of the closed-source Command
Post desktop app. Field names per endpoint are confirmed against the
wire; argument shapes match what the real client sends.
"""
from .client import (
    CommandPostClient,
    CommandPostError,
    Envelope,
    DEFAULT_BASE_URL,
    DEFAULT_PATH_PREFIX,
)
from .dsl import search_dsl
from .endpoints import CommandPostAPI, ENDPOINTS

__all__ = [
    "CommandPostAPI",
    "CommandPostClient",
    "CommandPostError",
    "Envelope",
    "ENDPOINTS",
    "DEFAULT_BASE_URL",
    "DEFAULT_PATH_PREFIX",
    "search_dsl",
]
__version__ = "0.1.0"
