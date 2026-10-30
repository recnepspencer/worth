import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from slot_history import archive_digest, read_history


class PortableHistoryTests(unittest.TestCase):
    def fixture(self, root):
        log = root/'original.log'
        event = {'type': 'test-cost-cpu', 'binary': 'court', 'test': 'claim', 'exit': 0,
                 'cpu_seconds': 1, 'wall_seconds': 2, 'reserved_slots': 8, 'accounting': 'complete'}
        log.write_text('Command: ["python", "phase_telemetry.py"]\nHEAD: captured-head\nGit status --short empty: True\n'
                       'Start: 2026-01-01T00:00:00+00:00\nEnd: 2026-01-01T00:00:01+00:00\nExit code: 0\n'+json.dumps(event), encoding='utf-8')
        source, events = read_history(root, {}, [log])[0]
        archive = root/'samples.jsonl'
        archive.write_text(json.dumps({'capture_sha256': source['sha256'], 'sample_index': 0,
                                      'observation': event})+'\n', encoding='utf-8')
        manifest = {'observations': [source], 'observation_files': [archive.name],
                    'observation_sha256': {archive.name: archive_digest(archive.read_bytes())}}
        return log, archive, manifest, events

    def test_committed_archive_remains_authoritative_when_original_log_is_absent(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            log, _, manifest, events = self.fixture(root)
            log.unlink()
            self.assertEqual(read_history(root, manifest, [])[0][1], events)

    def test_archive_tampering_refuses_even_when_original_log_is_absent(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            log, archive, manifest, _ = self.fixture(root)
            log.unlink()
            archive.write_text('{}\n', encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'archive hash mismatch'):
                read_history(root, manifest, [])

    def test_explicit_new_log_must_exist_and_immutable_sample_indices_cannot_repeat(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            log, archive, manifest, _ = self.fixture(root)
            with self.assertRaises(FileNotFoundError):
                read_history(root, manifest, [root/'missing-new.log'])
            archive.write_text(archive.read_text()*2, encoding='utf-8')
            manifest['observation_sha256'][archive.name] = archive_digest(archive.read_bytes())
            with self.assertRaisesRegex(ValueError, 'identity repeats'):
                read_history(root, manifest, [])

    def test_explicit_log_requires_a_git_status_observation_and_retains_its_boolean(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            log, _, manifest, _ = self.fixture(root)
            self.assertTrue(manifest['observations'][0]['git_status_short_empty'])
            log.write_text(log.read_text().replace('Git status --short empty: True\n', ''), encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'complete retained command evidence'):
                read_history(root, {}, [log])


if __name__ == '__main__':
    unittest.main()
