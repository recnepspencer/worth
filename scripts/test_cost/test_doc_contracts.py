import json
from pathlib import Path
from subprocess import CompletedProcess
import tempfile
import unittest

from doc_contracts import (DocContractFailure, DocFailureKind, execute, owners_for,
                           owners_from_metadata, parse_output)


def block(rows=(), failed=0, ignored=0):
    lines = [f'running {len(rows)} tests']
    lines += [f'test {name} ... {event}' for name, event in rows]
    lines += [f'test result: {"FAILED" if failed else "ok"}. {len(rows)-failed-ignored} passed; '
              f'{failed} failed; {ignored} ignored; 0 measured; 0 filtered out; finished in 0.01s']
    return '\n'.join(lines) + '\n'


class DocContractTests(unittest.TestCase):
    def test_combined_headings_own_multiple_merged_groups_and_standalone_cases(self):
        text = '   Doc-tests owner_pkg\n'
        text += block([('src/lib.rs - first (line 1)', 'ok')])
        text += block([('src/lib.rs - second (line 9) - compile fail', 'ok')])
        text += block([('src/lib.rs - scheduled (line 20)', 'ignored')], ignored=1)
        text += 'all doctests ran in 0.02s; merged doctests compilation took 0.01s\n'
        text += '   Doc-tests empty_pkg\n' + block()
        result = parse_output(text, '', {'owner_pkg': 'owner-pkg', 'empty_pkg': 'empty-pkg'})
        self.assertEqual(len(result['inventory']), 3)
        self.assertTrue(all(r['binary'] == 'owner-pkg::doc' for r in result['inventory']))
        self.assertEqual(result['inventory'][2]['test'], 'src/lib.rs - second (line 9)')
        self.assertEqual([e['event'] for e in result['events']], ['ok', 'ok', 'ignored'])

    def test_multiline_generic_item_identity_and_progress_preserve_the_real_framework_count(self):
        name = ('src/authoring.rs - ApplicationQueryResultAuthoring<Schema,Query,Parameters,Scope,\n'
                'Root>::result_shape (line 71)').replace('\\n', '\n')
        text = ('Doc-tests owner_pkg\nrunning 1 test\ntest '+name+
                ' has been running for over 60 seconds\ntest '+name+' - compile fail ... ok\n'
                'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 148.57s\n')
        result = parse_output(text, '', {'owner_pkg': 'owner-pkg'})
        self.assertEqual(result['inventory'], [{'binary': 'owner-pkg::doc', 'test': name, 'ignored': False}])
        self.assertEqual(len(result['events']), 1)
        self.assertEqual(result['blocks'][0]['summary'], {'passed': 1, 'failed': 0, 'ignored': 0})
        self.assertIn('\n', result['events'][0]['name'])

    def test_adjacent_multiline_names_stay_distinct_and_complete_duplicates_refuse(self):
        first = 'src/lib.rs - Generic<A,\nB>::first (line 7)'
        second = 'src/lib.rs - Generic<A,\nB>::second (line 8)'
        text = 'Doc-tests a\n'+block([(first, 'ok'), (second, 'ok')])
        result = parse_output(text, '', {'a': 'a'})
        self.assertEqual([row['test'] for row in result['inventory']], [first, second])
        duplicate = 'Doc-tests a\n'+block([(first, 'ok'), (first, 'ok')])
        with self.assertRaises(DocContractFailure) as raised:
            parse_output(duplicate, '', {'a': 'a'})
        self.assertEqual(raised.exception.kind, DocFailureKind.DUPLICATE)

    def test_multiline_test_rows_cannot_absorb_other_rows_or_invent_summary_counts(self):
        prefix = 'Doc-tests a\nrunning 1 test\ntest src/lib.rs - Generic<A,\n'
        summary = 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'
        for ending in [summary, 'test src/lib.rs - actual (line 3) ... ok\n'+summary, '']:
            with self.subTest(ending=ending), self.assertRaises(DocContractFailure) as raised:
                parse_output(prefix+ending, '', {'a': 'a'})
            self.assertEqual(raised.exception.kind, DocFailureKind.FRAMING)

    def test_separate_streams_are_accepted_only_when_block_assignment_is_unique(self):
        text = block([('src/lib.rs - first (line 1)', 'ok')]) + block()
        result = parse_output(text, '   Doc-tests a\n   Doc-tests b\n', {'a': 'a', 'b': 'b'})
        self.assertEqual(result['inventory'][0]['binary'], 'a::doc')

    def test_ambiguous_separate_streams_do_not_guess_owners_from_source_paths(self):
        text = block() + block() + 'all doctests ran in 0.02s; merged doctests compilation took 0.01s\n'
        text += block() + block() + block()
        text += 'all doctests ran in 0.02s; merged doctests compilation took 0.01s\n'
        with self.assertRaises(DocContractFailure) as failure:
            parse_output(text, 'Doc-tests a\nDoc-tests b\nDoc-tests c\n', {'a': 'a', 'b': 'b', 'c': 'c'})
        self.assertEqual(failure.exception.kind, DocFailureKind.AMBIGUOUS_OWNER)

    def test_missing_truncated_and_inconsistent_observations_are_refused(self):
        cases = [('Doc-tests a\nrunning 1 test\n', DocFailureKind.FRAMING),
                 ('Doc-tests a\n' + block([('src/lib.rs - x (line 1)', 'ok')]).replace('1 passed', '0 passed'), DocFailureKind.COUNTS),
                 ('Doc-tests a\n', DocFailureKind.FRAMING),
                 (block(), DocFailureKind.MISSING_OWNER)]
        for text, kind in cases:
            with self.subTest(text=text):
                with self.assertRaises(DocContractFailure) as failure:
                    parse_output(text, '', {'a': 'a'})
                self.assertEqual(failure.exception.kind, kind)

    def test_duplicate_names_unknown_owners_and_repeated_headings_are_refused(self):
        duplicate = 'Doc-tests a\n' + block([('src/lib.rs - x (line 1)', 'ok')]) * 2
        for text, kind in [(duplicate, DocFailureKind.DUPLICATE),
                           ('Doc-tests foreign\n' + block(), DocFailureKind.UNKNOWN_OWNER),
                           ('Doc-tests a\n' + block() + 'Doc-tests a\n' + block(), DocFailureKind.AMBIGUOUS_OWNER)]:
            with self.subTest(kind=kind):
                with self.assertRaises(DocContractFailure) as failure:
                    parse_output(text, '', {'a': 'a'})
                self.assertEqual(failure.exception.kind, kind)

    def workspace(self, root):
        (root / 'Cargo.toml').write_text('[workspace]\nmembers=["owner"]\n', encoding='utf-8')
        (root / 'owner/src').mkdir(parents=True)
        (root / 'owner/Cargo.toml').write_text('[package]\nname="owner-pkg"\nversion="0.1.0"\n', encoding='utf-8')
        (root / 'owner/src/lib.rs').write_text('', encoding='utf-8')
        return root / 'Cargo.toml'

    def test_explicit_doc_selection_uses_actual_library_name_even_when_default_doctests_are_disabled(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = self.workspace(root)
            self.assertEqual(owners_for(manifest), {'owner_pkg': 'owner-pkg'})
            package = root / 'owner/Cargo.toml'
            package.write_text(package.read_text() + '[lib]\nname="public_api"\n', encoding='utf-8')
            self.assertEqual(owners_for(manifest), {'public_api': 'owner-pkg'})
            package.write_text(package.read_text() + 'doctest=false\n', encoding='utf-8')
            self.assertEqual(owners_for(manifest), {'public_api': 'owner-pkg'})
            package.write_text(package.read_text() + 'crate-type=["cdylib", "staticlib"]\n', encoding='utf-8')
            self.assertEqual(owners_for(manifest), {})

    def test_captured_metadata_owners_include_disabled_defaults_but_only_doctestable_members(self):
        def package(identity, crate_type='lib'):
            return {'id': identity, 'name': identity, 'targets': [{'name': identity.replace('-', '_'),
                    'kind': [crate_type], 'crate_types': [crate_type], 'doctest': False}]}
        metadata = {'workspace_members': ['owner-pkg', 'dynamic'],
                    'packages': [package('owner-pkg'), package('dynamic', 'cdylib'), package('dependency')]}
        self.assertEqual(owners_from_metadata(metadata), {'owner_pkg': 'owner-pkg'})
        metadata['workspace_members'].append('missing')
        with self.assertRaises(DocContractFailure) as failure:
            owners_from_metadata(metadata)
        self.assertEqual(failure.exception.kind, DocFailureKind.MISSING_OWNER)

    def test_explicit_empty_library_is_honored_when_automatic_discovery_is_disabled(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = self.workspace(root)
            package = root / 'owner/Cargo.toml'
            package.write_text(package.read_text() + 'autolib=false\n[lib]\n', encoding='utf-8')
            self.assertEqual(owners_for(manifest), {'owner_pkg': 'owner-pkg'})
            (root / 'owner/src/lib.rs').unlink()
            with self.assertRaises(DocContractFailure) as failure:
                owners_for(manifest)
            self.assertEqual(failure.exception.kind, DocFailureKind.MISSING_OWNER)

    def test_library_name_collisions_are_refused_before_cargo_launch(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = self.workspace(root)
            (root / 'Cargo.toml').write_text('[workspace]\nmembers=["owner", "second"]\n', encoding='utf-8')
            (root / 'second/src').mkdir(parents=True)
            (root / 'second/src/lib.rs').write_text('', encoding='utf-8')
            (root / 'second/Cargo.toml').write_text(
                '[package]\nname="second"\nversion="0.1.0"\n[lib]\nname="owner_pkg"\n', encoding='utf-8')
            with self.assertRaises(DocContractFailure) as failure:
                owners_for(manifest)
            self.assertEqual(failure.exception.kind, DocFailureKind.AMBIGUOUS_OWNER)

    def test_failure_diagnostics_cannot_introduce_another_test_observation(self):
        text = 'Doc-tests a\nrunning 1 test\ntest src/lib.rs - x (line 1) ... FAILED\n'
        text += '\nfailures:\n---- src/lib.rs - x (line 1) stdout ----\n'
        text += 'test src/lib.rs - invented (line 5) ... ok\n'
        text += '\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n'
        result = parse_output(text, '', {'a': 'a'})
        self.assertEqual([row['event'] for row in result['events']], ['failed'])
        self.assertEqual(len(result['inventory']), 1)

    def test_empty_workspace_library_set_has_no_invented_tests(self):
        result = parse_output('', '', {})
        self.assertEqual(result['inventory'], [])
        self.assertEqual(result['events'], [])

    def test_execution_keeps_failures_and_appends_observed_events_to_both_logs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = self.workspace(root)
            text = 'Doc-tests owner_pkg\n' + block([
                ('src/lib.rs - good (line 1)', 'ok'),
                ('src/lib.rs - bad (line 5)', 'FAILED'),
                ('src/lib.rs - scheduled (line 9)', 'ignored')], failed=1, ignored=1)

            class Runner:
                environment = {}
                logs = []
                def __call__(self, command, cwd, log):
                    self.command = command
                    self.original = log
                    log.write_text(text, encoding='utf-8')
                    retained = root / 'canonical.log'
                    retained.write_text(text, encoding='utf-8')
                    self.logs.append(retained)
                    return CompletedProcess(command, 101, text, '')

            runner = Runner()
            result = execute(manifest, root / 'output', runner, profile='ci-test')
            self.assertEqual(result['status'], 'fail')
            self.assertEqual(result['exit_code'], 101)
            self.assertEqual(len(result['inventory']), 3)
            self.assertEqual(len(result['selections']), 2)
            self.assertEqual(result['counts'], {'ok': 1, 'failed': 1, 'ignored': 1})
            self.assertIn('--doc', runner.command)
            self.assertIn('--no-fail-fast', runner.command)
            self.assertIn('--message-format=json', runner.command)
            self.assertEqual(runner.command[runner.command.index('--profile')+1], 'ci-test')
            self.assertFalse(any(a in runner.command for a in ['--ignored', '--include-ignored']))
            for path in (runner.original, runner.logs[-1]):
                events = [json.loads(line) for line in path.read_text().splitlines() if line.startswith('{')]
                self.assertEqual([r['event'] for r in events], ['ok', 'failed', 'ignored'])
            self.assertEqual(json.loads((root / 'output/inventory.json').read_text()), result['inventory'])

    def test_aggregate_nonzero_exit_does_not_invent_a_failed_test(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = self.workspace(root)
            text = 'Doc-tests owner_pkg\n' + block([('src/lib.rs - good (line 1)', 'ok')])
            def runner(command, cwd, log):
                log.write_text(text, encoding='utf-8')
                return CompletedProcess(command, 101, text, '')
            result = execute(manifest, root / 'output', runner)
            self.assertEqual(result['status'], 'fail')
            self.assertEqual(result['events'][0]['event'], 'ok')

    def test_inherited_ignored_selection_refuses_before_cargo_launch(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = self.workspace(root)
            class Runner:
                environment = {'RUSTDOCFLAGS': '--test-args=--include-ignored'}
                def __call__(self, *_):
                    self.fail('Cargo must not launch')
            with self.assertRaises(DocContractFailure) as failure:
                execute(manifest, root / 'output', Runner())
            self.assertEqual(failure.exception.kind, DocFailureKind.UNSAFE_SELECTION)


if __name__ == '__main__':
    unittest.main()
