import json
import shutil
from pathlib import Path
import tempfile
import unittest

from measured_slots import load
from slot_policy import generate, pending_claims
from policy_catalog import files as catalog_files, read as read_catalog


def sample(test='claim', cpu=1, wall=2, slots=8, exit=0, accounting='complete'):
    return {'type': 'test-cost-cpu', 'binary': 'court', 'test': test, 'cpu_seconds': cpu,
            'wall_seconds': wall, 'reserved_slots': slots, 'exit': exit, 'accounting': accounting}


class SlotPolicyTests(unittest.TestCase):
    def test_standalone_cpu_is_retained_but_only_an_ordinary_main_function_receives_a_rule(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            events = [sample('<main>', cpu=2, wall=1),
                      dict(sample('<main>', cpu=2, wall=1, slots=None), scheduler='standalone-process'),
                      sample('main', cpu=2, wall=1)]
            path = self.capture(root, 'actual.log', events)
            result = generate(root/'policy', [path])
            self.assertEqual(load(root/'policy'), {'court\nmain': 2})
            self.assertEqual(result['sample_count'], 3)
            self.assertEqual(result['complete_samples'], 3)
            self.assertEqual(result['standalone_complete_samples'], 2)
            self.assertEqual(result['scheduler_samples'], 1)
            self.assertEqual(result['unmeasured_samples'], 0)
            archived = read_catalog(root/'policy', result['catalogs']['samples'])
            self.assertEqual([row['observation'] for row in archived], events)
            self.assertEqual([row['disposition'] for row in archived],
                             ['standalone-measured', 'standalone-measured', 'measured-success'])
            self.assertTrue(all(row['recommended_slots'] is None for row in archived[:2]))

    def test_explicit_lineage_resolution_clears_only_named_pending_claims_and_preserves_all_samples(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            capture = self.capture(root, 'initial.log', [sample()])
            before = generate(root/'policy', [capture], pending=['court$resolved', 'court$remaining'])
            archives = {name: (root/'policy'/name).read_bytes() for name in catalog_files(root/'policy', before['catalogs']['samples'])}
            after = generate(root/'policy', [capture], resolved=['court$resolved'])
            self.assertEqual(pending_claims(root/'policy'), ['court$remaining'])
            self.assertEqual(after['sample_count'], before['sample_count'])
            self.assertEqual(after['catalogs']['sources'], before['catalogs']['sources'])
            self.assertEqual(after['catalogs']['samples'], before['catalogs']['samples'])
            self.assertEqual(archives, {name: (root/'policy'/name).read_bytes() for name in catalog_files(root/'policy', after['catalogs']['samples'])})
            explicit = generate(root/'policy', [capture], pending=['court$remaining'], resolved=['court$remaining'])
            self.assertEqual(pending_claims(root/'policy'), ['court$remaining'])

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
            self.assertEqual(pending_claims(root/'fresh-policy'), ['court$authority'])
            old_ids = {(row['capture_sha256'], row['sample_index'])
                       for row in read_catalog(root/'original-policy', before['catalogs']['samples'])}
            new_ids = {(row['capture_sha256'], row['sample_index'])
                       for row in read_catalog(root/'fresh-policy', after['catalogs']['samples'])}
            self.assertTrue(old_ids <= new_ids)

    def test_tampered_committed_archive_is_refused_without_original_logs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            initial = self.capture(root, 'initial.log', [sample()])
            before = generate(root/'policy', [initial])
            initial.unlink()
            archive = root/'policy'/before['catalogs']['samples']['file']
            archive.write_text('{}\n', encoding='utf-8')
            repair = self.capture(root, 'repair.log', [sample()])
            with self.assertRaisesRegex(ValueError, 'catalog hash mismatch'):
                generate(root/'policy', [repair])

    def capture(self, root, name, events):
        path = root / name
        path.write_text(f'Command: ["python", "phase_telemetry.py"]\nWorking directory: {root}\nHEAD: original-head\nGit status --short empty: True\n'
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
            self.assertEqual(result['source_count'], 2)
            archives = read_catalog(policy, result['catalogs']['samples'])
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
            self.assertEqual(pending_claims(root/'policy'), ['court$authority_pending'])
            self.assertEqual(result['status'], 'partial')

    def test_generated_policy_and_full_sample_archives_remain_bounded(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = self.capture(root, 'samples.log', [sample(str(i)) for i in range(205)])
            result = generate(root/'policy', [path])
            self.assertEqual(result['catalogs']['rules']['records'], 205)
            self.assertEqual(len(catalog_files(root/'policy', result['catalogs']['rules'])), 4)
            self.assertEqual(result['catalogs']['samples']['records'], 205)
            self.assertEqual(len(catalog_files(root/'policy', result['catalogs']['samples'])), 4)
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
