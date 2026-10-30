import json
import shutil
from pathlib import Path
import tempfile
import unittest

from measured_slots import load
from slot_policy import generate


def sample(test='claim', cpu=1, wall=2, slots=8, exit=0, accounting='complete'):
    return {'type': 'test-cost-cpu', 'binary': 'court', 'test': test, 'cpu_seconds': cpu,
            'wall_seconds': wall, 'reserved_slots': slots, 'exit': exit, 'accounting': accounting}


class SlotPolicyTests(unittest.TestCase):
    def test_explicit_lineage_resolution_clears_only_named_pending_claims_and_preserves_all_samples(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            capture = self.capture(root, 'initial.log', [sample()])
            before = generate(root/'policy', [capture], pending=['court$resolved', 'court$remaining'])
            archives = {name: (root/'policy'/name).read_bytes() for name in before['observation_files']}
            after = generate(root/'policy', [capture], resolved=['court$resolved'])
            self.assertEqual(after['pending_claims'], ['court$remaining'])
            self.assertEqual(after['sample_count'], before['sample_count'])
            self.assertEqual(after['observations'], before['observations'])
            self.assertEqual(after['observation_sha256'], before['observation_sha256'])
            self.assertEqual(archives, {name: (root/'policy'/name).read_bytes() for name in after['observation_files']})
            explicit = generate(root/'policy', [capture], pending=['court$remaining'], resolved=['court$remaining'])
            self.assertEqual(explicit['pending_claims'], ['court$remaining'])

    def test_fresh_checkout_keeps_archived_maxima_without_ignored_target_logs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            initial = self.capture(root, 'initial.log', [sample('parallel', cpu=6), sample('sequential')])
            before = generate(root/'original-policy', [initial], pending=['court$authority'])
            shutil.copytree(root/'original-policy', root/'fresh-policy')
            for archive in (root/'fresh-policy').glob('*.jsonl'):
                archive.write_bytes(archive.read_bytes().replace(b'\r\n', b'\n'))
            initial.unlink()
            repair = self.capture(root, 'repair.log', [sample('parallel')])
            after = generate(root/'fresh-policy', [repair])
            self.assertEqual(load(root/'fresh-policy'), {'court\nparallel': 3, 'court\nsequential': 1})
            self.assertEqual(after['sample_count'], 3)
            self.assertEqual(after['pending_claims'], ['court$authority'])
            old_ids = {(json.loads(line)['capture_sha256'], json.loads(line)['sample_index'])
                       for name in before['observation_files'] for line in (root/'original-policy'/name).read_text().splitlines()}
            new_ids = {(json.loads(line)['capture_sha256'], json.loads(line)['sample_index'])
                       for name in after['observation_files'] for line in (root/'fresh-policy'/name).read_text().splitlines()}
            self.assertTrue(old_ids <= new_ids)

    def test_tampered_committed_archive_is_refused_without_original_logs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            initial = self.capture(root, 'initial.log', [sample()])
            before = generate(root/'policy', [initial])
            initial.unlink()
            archive = root/'policy'/before['observation_files'][0]
            archive.write_text('{}\n', encoding='utf-8')
            repair = self.capture(root, 'repair.log', [sample()])
            with self.assertRaisesRegex(ValueError, 'archive hash mismatch'):
                generate(root/'policy', [repair])

    def capture(self, root, name, events):
        path = root / name
        path.write_text('Command: ["python", "phase_telemetry.py"]\nHEAD: original-head\nGit status --short empty: True\n'
                        'Start: 2026-01-01T00:00:00+00:00\nEnd: 2026-01-01T00:00:01+00:00\nExit code: 0\n' +
                        '\n'.join(json.dumps(event) for event in events), encoding='utf-8')
        return path

    def test_partial_repair_preserves_full_history_and_uses_maximum_per_exact_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            initial = self.capture(root, 'initial.log', [sample('sequential'), sample('parallel', cpu=6, wall=2)])
            repair = self.capture(root, 'repair.log', [sample('parallel', cpu=1), sample('new')])
            policy = root / 'policy'
            generate(policy, [initial])
            result = generate(policy, [repair])
            self.assertEqual(load(policy), {'court\nsequential': 1, 'court\nparallel': 3, 'court\nnew': 1})
            self.assertEqual(result['sample_count'], 4)
            self.assertEqual(len(result['observations']), 2)
            archives = [json.loads(line) for name in result['observation_files'] for line in (policy/name).read_text().splitlines()]
            self.assertEqual(len(archives), 4)
            self.assertTrue(all(row['capture_sha256'] and row['observation'] for row in archives))
            again = generate(policy, [repair])
            self.assertEqual(again, result)

    def test_failed_or_unusable_samples_cannot_create_zero_or_override_measured_success(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = self.capture(root, 'samples.log', [sample(cpu=2, wall=1), sample(cpu=999, exit=101),
                sample('unknown', cpu=None), sample('incomplete', accounting='descendants-still-active')])
            result = generate(root/'policy', [path], pending=['court$authority_pending'])
            self.assertEqual(load(root/'policy'), {'court\nclaim': 2})
            self.assertEqual(result['complete_samples'], 1)
            self.assertEqual(result['unmeasured_samples'], 2)
            self.assertEqual(result['failed_child_samples'], 1)
            self.assertEqual(result['pending_claims'], ['court$authority_pending'])
            self.assertEqual(result['status'], 'partial')

    def test_generated_policy_and_full_sample_archives_remain_bounded(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = self.capture(root, 'samples.log', [sample(str(i)) for i in range(205)])
            result = generate(root/'policy', [path])
            self.assertEqual(len(result['files']), 3)
            self.assertEqual(len(result['observation_files']), 3)
            self.assertTrue(all(len(p.read_text().splitlines()) <= 400 for p in (root/'policy').iterdir()))

    def test_missing_or_duplicate_retained_inputs_are_refused_without_erasing_the_policy(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = self.capture(root, 'samples.log', [sample()])
            generate(root/'policy', [path])
            before = (root/'policy/index.json').read_text()
            with self.assertRaises(ValueError):
                generate(root/'policy', [path, path])
            with self.assertRaises(FileNotFoundError):
                generate(root/'policy', [root/'missing.log'])
            self.assertEqual((root/'policy/index.json').read_text(), before)

    def test_incremental_generation_preserves_declared_baseline_and_refuses_changed_retained_samples(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = self.capture(root, 'initial.log', [sample(slots=1)])
            generate(root/'policy', [path], baseline={'court\nclaim': 8})
            generate(root/'policy', [path])
            self.assertEqual(load(root/'policy'), {'court\nclaim': 1})
            path.write_text(path.read_text().replace('"cpu_seconds": 1', '"cpu_seconds": 999'), encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'source changed'):
                generate(root/'policy', [path])


if __name__ == '__main__':
    unittest.main()
