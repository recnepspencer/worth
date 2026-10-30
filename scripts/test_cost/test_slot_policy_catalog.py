import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

from policy_catalog import read
from slot_policy import generate, pending_claims, reservation_values


class LargePolicyCatalogTests(unittest.TestCase):
    def test_thousand_source_history_keeps_fixed_root_and_all_samples_across_a_partial_update(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            history = []
            for number in range(1_005):
                identity = hashlib.sha256(str(number).encode()).hexdigest()
                source = {'sha256': identity, 'telemetry_log': str(root/f'absent-{number}.log'),
                          'git_status_short_empty': True, 'head': 'fixture-head',
                          'headers': {'Command': '["python", "phase_telemetry.py"]', 'Working directory': str(root),
                              'HEAD': 'fixture-head', 'Git status --short empty': 'True',
                              'Start': '2026-01-01T00:00:00+00:00', 'End': '2026-01-01T00:00:01+00:00', 'Exit code': '0'}}
                event = {'type': 'test-cost-cpu', 'binary': 'court', 'test': f'claim_{number}',
                         'cpu_seconds': 3, 'wall_seconds': 2, 'reserved_slots': 8,
                         'accounting': 'complete', 'exit': 0}
                history.append((source, [event]))
            policy = root/'policy'
            with patch('slot_policy.read_history', return_value=history):
                before = generate(policy, [], pending=[f'authority_{n}' for n in range(1_005)])
            self.assertEqual(before['schema'], 2)
            self.assertEqual(before['source_count'], 1_005)
            self.assertEqual(before['catalogs']['rules']['records'], 1_005)
            self.assertEqual(before['catalogs']['samples']['records'], 1_005)
            self.assertEqual(before['catalogs']['pending']['records'], 1_005)
            old_samples = read(policy, before['catalogs']['samples'])
            copied = root/'fresh-checkout'
            shutil.copytree(policy, copied)
            # No historical raw logs exist. This is an actual new retained input.
            log = root/'partial.log'
            event = dict(history[0][1][0], cpu_seconds=1)
            log.write_text(f'Command: ["python", "phase_telemetry.py"]\nWorking directory: {root}\nHEAD: new-head\n'
                           'Git status --short empty: False\nStart: 2026-01-01T00:00:00+00:00\n'
                           'End: 2026-01-01T00:00:01+00:00\nExit code: 0\n'+json.dumps(event), encoding='utf-8')
            after = generate(copied, [log], known={'court\nclaim_0'})
            self.assertEqual(reservation_values(read(copied, after['catalogs']['rules'])), {'court\nclaim_0': 2})
            self.assertEqual(after['retired_identities'], 1_004)
            self.assertEqual(after['catalogs']['retired']['records'], 1_004)
            self.assertEqual(len(pending_claims(copied)), 1_005)
            samples = read(copied, after['catalogs']['samples'])
            self.assertEqual(after['sample_count'], 1_006)
            old = {(s['capture_sha256'], s['sample_index']): s['observation'] for s in old_samples}
            current = {(s['capture_sha256'], s['sample_index']): s['observation'] for s in samples}
            self.assertEqual(old, {key: current[key] for key in old})
            for directory in (policy, copied):
                self.assertTrue(all(len(path.read_text().splitlines()) <= 400 for path in directory.iterdir()))
                self.assertLess(len((directory/'index.json').read_text().splitlines()), 100)


if __name__ == '__main__':
    unittest.main()
