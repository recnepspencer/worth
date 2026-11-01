"""Consumer CI routing uses fixture Cargo graphs, never qualification matrices."""
import contextlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch

import select_qualification as routing


class QualificationSelectionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        config = self.root/'tools/boundary-check/config/consumer-docs.toml'
        config.parent.mkdir(parents=True)
        config.write_text('documents=["docs/consumer.md"]\n', encoding='utf-8')
        self.graphs = {'Cargo.toml': [('crates/root-owner', True), ('crates/shared', False)],
                       routing.WORKSPACES['query']: [('workspaces/query/owner', True),
                                                     ('crates/shared', False)],
                       routing.WORKSPACES['bank']: [('workspaces/bank/owner', True)],
                       routing.FIXTURE: [('fixture/consumer', True), ('crates/shared', False)]}
        self.discover = Mock(side_effect=lambda repo, manifest, **_: self.graphs[manifest])

    def selected(self, *paths):
        result = routing.select(self.root, paths, self.discover)
        return result['consumer_docs'], result['checkpoint_recovery']

    def test_related_transitive_sources_select_their_real_consumer_closures(self):
        self.assertEqual(self.selected('crates/shared/src/changed.rs'), (True, True))
        self.assertEqual(self.selected('crates/root-owner/src/changed.rs'), (True, False))
        self.assertEqual(self.selected('fixture/consumer/tests/claim.rs'), (False, True))
        self.assertEqual(self.selected('crates/shared/tests/unrelated.rs'), (False, False))
        self.assertEqual(self.selected('workspaces/bank/owner/src/lib.rs'), (True, False))

    def test_configured_documents_select_docs_but_unrelated_plans_and_slot_data_skip_both(self):
        self.assertEqual(self.selected('docs/consumer.md'), (True, False))
        self.assertEqual(self.selected('README.md'), (True, False))
        self.assertEqual(self.selected('plans/task.md', 'docs/unlisted.md',
                                       'scripts/test_cost/slots/query/index.json'), (False, False))
        self.discover.assert_not_called()

    def test_deleted_sources_manifests_lockfiles_and_inherited_configuration_still_reach(self):
        for path in ('crates/shared/src/deleted.rs', 'crates/shared/Cargo.toml', 'Cargo.toml',
                     'Cargo.lock', '.cargo/config.toml', 'rust-toolchain.toml'):
            with self.subTest(path=path):
                self.assertEqual(self.selected(path), (True, True))
        self.assertEqual(self.selected('workspaces/worth-query-bank-world/Cargo.lock'), (True, False))

    def test_graph_failure_cannot_skip_qualification(self):
        self.discover.side_effect = subprocess.CalledProcessError(1, ['cargo', 'metadata'])
        result = routing.select(self.root, ['crates/shared/src/lib.rs'], self.discover)
        self.assertTrue(result['consumer_docs'] and result['checkpoint_recovery'])
        self.assertIn('graph unavailable', result['reason'])

    def test_unrecognized_events_and_manual_inputs_cannot_skip_qualification(self):
        for name, event in [('push', []), ('workflow_dispatch', {'inputs': []}),
                            ('workflow_dispatch', {'inputs': {'scope': 'unknown'}}), ('other', {})]:
            with self.subTest(name=name, event=event):
                result = routing.selection(self.root, name, event, self.discover)
                self.assertTrue(result['consumer_docs'] and result['checkpoint_recovery'])
        self.discover.assert_not_called()

    def test_explicit_manual_modes_never_discover_or_compare(self):
        with patch.object(routing, 'changed_paths', side_effect=AssertionError('unexpected diff')):
            for scope, wanted in [('all', (True, True)), ('consumer-docs', (True, False)),
                                  ('checkpoint-only', (False, True))]:
                result = routing.selection(self.root, 'workflow_dispatch', {'inputs': {'scope': scope}},
                                           self.discover)
                self.assertEqual((result['consumer_docs'], result['checkpoint_recovery']), wanted)
        self.discover.assert_not_called()

    def test_unknown_base_retains_both_and_pull_requests_use_the_complete_merge_base_diff(self):
        with patch.object(routing, 'changed_paths', side_effect=ValueError('unknown base')):
            result = routing.selection(self.root, 'push', {'before': 'missing', 'after': 'head'})
            self.assertTrue(result['consumer_docs'] and result['checkpoint_recovery'])
        event = {'pull_request': {'base': {'sha': 'base'}, 'head': {'sha': 'head'}}}
        with patch.object(routing, 'changed_paths', return_value=['plans/task.md']) as diff:
            result = routing.selection(self.root, 'pull_request', event, self.discover)
            diff.assert_called_once_with(self.root, 'base', 'head', merge_base=True)
            self.assertFalse(result['consumer_docs'] or result['checkpoint_recovery'])

    def test_qualification_commands_and_inventory_changes_select_both(self):
        for path in ('.github/workflows/ci.yml', 'scripts/ci/query_gate.py',
                     'scripts/test_cost/cargo_reach.py', 'scripts/test_cost/compiler_stages.json',
                     'tools/boundary-check/config/consumer-docs.toml'):
            with self.subTest(path=path):
                self.assertEqual(self.selected(path), (True, True))
        self.discover.assert_not_called()

    def test_cli_appends_job_outputs_and_keeps_missing_event_conservative(self):
        event_path = self.root/'event.json'; output = self.root/'output'
        event_path.write_text(json.dumps({'inputs': {'scope': 'checkpoint-only'}}))
        args = ['select', '--repository', str(self.root), '--event-name', 'workflow_dispatch',
                '--event-path', str(event_path), '--output', str(output)]
        with patch('sys.argv', args), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(routing.main(), 0)
        self.assertEqual(output.read_text(), 'consumer_docs=false\ncheckpoint_recovery=true\n')
        event_path.unlink()
        with patch('sys.argv', args), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(routing.main(), 0)
        self.assertTrue(output.read_text().endswith('consumer_docs=true\ncheckpoint_recovery=true\n'))


if __name__ == '__main__':
    unittest.main()
