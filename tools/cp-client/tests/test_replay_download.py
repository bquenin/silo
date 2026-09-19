import json
import sys
import unittest
from pathlib import Path
from unittest.mock import Mock

import requests

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from cp_client import CommandPostAPI


def response(payload):
    result = requests.Response()
    result.status_code = 200
    result._content = json.dumps(payload).encode()
    return result


class ReplayDownloadTests(unittest.TestCase):
    def setUp(self):
        self.session = Mock(spec=requests.Session)
        self.session.headers = {}
        self.session.get.return_value = response({})
        self.client = CommandPostAPI("test-user", "test-password", session=self.session)

    def test_raw_and_escaped_filenames_keep_their_identity(self):
        for filename, encoded in [
            ("Replay #1.KWReplay", "Replay%20%231.KWReplay"),
            ("Replay%20%231.KWReplay", "Replay%20%231.KWReplay"),
            ("Replay%2Fname.KWReplay", "Replay%2Fname.KWReplay"),
            ("100% win.KWReplay", "100%25%20win.KWReplay"),
            ("\u00e9 [1].KWReplay", "%C3%A9%20%5B1%5D.KWReplay"),
        ]:
            with self.subTest(filename=filename):
                prefix = "https://example.test/production/public/replays/"
                self.client.download_replay(1, detail={"url": prefix + filename})
                self.assertEqual(self.session.get.call_args.args[0], prefix + encoded)

    def test_detail_after_previous_lookup_or_download_always_has_a_record(self):
        detail = {"id": 1, "url": "https://example.test/replay.KWReplay"}
        bodies = []

        def post(_url, **kwargs):
            body = kwargs["data"].to_string()
            bodies.append(body)
            unchanged = b'name="response_checksum"' in body
            return response({"status": 1, "checksum": 12,
                             "output": None if unchanged else [detail]})

        self.session.post.side_effect = post
        # Seed a checksum through the raw API, then use the record wrapper.
        self.client.call("fetch_replays", fields={"type": 4, "replay_id": 1})
        self.assertEqual(self.client.fetch_replay_detail(1), detail)
        self.assertEqual(self.client.fetch_replay_detail(1), detail)
        self.assertEqual(self.client.download_replay(1)[1], detail)
        self.assertTrue(all(b'name="response_checksum"' not in b for b in bodies))


if __name__ == "__main__":
    unittest.main()
