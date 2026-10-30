"""Verification artifacts carry descriptive identity, never qualification."""
import copy
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from query_gate_environment import require_build_identity, verification_identity
from workspace import WORKSPACES


FIRST = '146c51ac-3521-4165-bdb5-87a7681c3629'
SECOND = 'e4fb56cd-8429-44ec-9f63-41d850e73c7e'


def receipt(root, output, workspace, identity=FIRST):
    manifest = (root / WORKSPACES[workspace]).resolve()
    value = {'schema': 1, 'type': 'compiler-unit-plan', 'units': [],
             'graph': {'version': 1, 'units': [], 'roots': []},
             'contract': {'cwd': str(manifest.parent),
                'command': ['cargo', 'nextest', 'list', '--manifest-path', str(manifest),
                            '--cargo-profile', 'dev', '--list-type', 'binaries-only'],
                'compiler_inputs': {'environment': {'WORTH_IMPLEMENTATION_BUILD_ID': identity}}}}
    path = output/workspace/'build/binaries-build-units.json'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value), encoding='utf-8')
    return path, value


class VerificationArtifactIdentityTests(unittest.TestCase):
    def test_uuid_case_variants_are_coherent_and_one_artifact_keeps_its_issued_spelling(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve(); output = root/'out'
            receipt(root, output, 'query', FIRST.upper())
            self.assertEqual(verification_identity(root, output, ['query']), FIRST.upper())
            receipt(root, output, 'bank', FIRST)
            self.assertEqual(verification_identity(root, output, ['query', 'bank']), FIRST)

    def test_each_requested_artifact_must_carry_one_coherent_issued_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve(); output = root/'out'
            self.assertIsNone(verification_identity(root, output, ['query']))
            receipt(root, output, 'query')
            self.assertEqual(verification_identity(root, output, ['query']), FIRST)
            self.assertIsNone(verification_identity(root, output, ['query', 'bank']))
            receipt(root, output, 'bank')
            self.assertEqual(verification_identity(root, output, ['query', 'bank']), FIRST)
            receipt(root, output, 'bank', SECOND)
            self.assertIsNone(verification_identity(root, output, ['query', 'bank']))
            self.assertEqual(verification_identity(root, output, ['query']), FIRST)

    def test_unrelated_or_malformed_json_cannot_supply_an_artifact_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve(); output = root/'out'
            path, valid = receipt(root, output, 'query')
            variants = [{}, [], {'WORTH_IMPLEMENTATION_BUILD_ID': FIRST}]
            for key, value in (('schema', True), ('type', 'unrelated-artifact'), ('units', None), ('graph', []),
                               ('graph', {'version': 1}), ('graph', {'version': True, 'units': [], 'roots': []})):
                changed = copy.deepcopy(valid); changed[key] = value; variants.append(changed)
            for key, value in (('cwd', str(root)), ('command', ['cargo', 'build'])):
                changed = copy.deepcopy(valid); changed['contract'][key] = value; variants.append(changed)
            changed = copy.deepcopy(valid)
            changed['contract']['command'][4] = str(root/'Cargo.toml'); variants.append(changed)
            changed = copy.deepcopy(valid)
            changed['contract']['command'][-1] = 'unrelated-listing'; variants.append(changed)
            for identity in ('not-a-uuid', '', '00000000-0000-0000-0000-000000000000'):
                changed = copy.deepcopy(valid)
                changed['contract']['compiler_inputs']['environment']['WORTH_IMPLEMENTATION_BUILD_ID'] = identity
                variants.append(changed)
            for variant in variants:
                with self.subTest(variant=variant):
                    path.write_text(json.dumps(variant), encoding='utf-8')
                    self.assertIsNone(verification_identity(root, output, ['query']))
            path.write_text('{', encoding='utf-8')
            self.assertIsNone(verification_identity(root, output, ['query']))

    def test_identity_input_reuse_does_not_claim_source_or_graph_freshness(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve(); output = root/'out'
            path, value = receipt(root, output, 'root')
            value['contract']['manifests'] = {'old source': 'stale hash'}
            value['graph']['source_freshness'] = 'not established by identity reuse'
            path.write_text(json.dumps(value), encoding='utf-8')
            self.assertEqual(verification_identity(root, output, ['root']), FIRST)
            self.assertIsNone(verification_identity(root, output/'different-artifact', ['root']))
            self.assertIsNone(verification_identity(root, output, []))

    def test_explicit_identity_syntax_matches_the_existing_nonzero_uuid_contract(self):
        self.assertEqual(require_build_identity(FIRST.upper()), FIRST.upper())
        for value in (None, 1, '', FIRST.replace('-', ''), '{'+FIRST+'}',
                      '00000000-0000-0000-0000-000000000000'):
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, 'nonzero UUID'):
                require_build_identity(value)


class GateIdentityLifecycleTests(unittest.TestCase):
    def test_repeated_focused_plans_share_artifact_input_and_freeze_it_for_all_children(self):
        import query_gate
        with tempfile.TemporaryDirectory() as temporary, patch.dict(os.environ, {}, clear=True):
            root = Path(temporary).resolve(); output = root/'out'
            receipt(root, output/'test-cost', 'query')
            with patch.object(query_gate, 'ROOT', root):
                first = query_gate.Gate(output); first.plan(['query-execution'])
                second = query_gate.Gate(output); second.plan(['query-rest'])
                self.assertEqual(first.env['WORTH_IMPLEMENTATION_BUILD_ID'], FIRST)
                self.assertEqual(second.env['WORTH_IMPLEMENTATION_BUILD_ID'], FIRST)
                self.assertIs(first.cost.environment, first.env)
                self.assertIs(first.prerequisites.environment, first.env)
                receipt(root, output/'test-cost', 'query', SECOND)
                first.plan(['query-rest'])
                self.assertEqual(first.env['WORTH_IMPLEMENTATION_BUILD_ID'], FIRST)

    def test_missing_malformed_mixed_and_new_output_artifacts_get_one_new_verification_identity(self):
        import query_gate
        with tempfile.TemporaryDirectory() as temporary, patch.dict(os.environ, {}, clear=True):
            root = Path(temporary).resolve(); output = root/'out'
            with patch.object(query_gate, 'ROOT', root):
                missing = query_gate.Gate(output); missing.plan(['query-execution'])
                path, _ = receipt(root, output/'test-cost', 'query')
                path.write_text('{}', encoding='utf-8')
                malformed = query_gate.Gate(output); malformed.plan(['query-execution'])
                receipt(root, output/'test-cost', 'query')
                receipt(root, output/'test-cost', 'bank', SECOND)
                mixed = query_gate.Gate(output); mixed.plan(['query-execution', 'bank'])
                distinct = query_gate.Gate(root/'new-output'); distinct.plan(['query-execution'])
                values = [gate.env['WORTH_IMPLEMENTATION_BUILD_ID'] for gate in (missing, malformed, mixed, distinct)]
                self.assertEqual(len(set(values)), 4)
                self.assertTrue(all(value not in (FIRST, SECOND) for value in values))
                self.assertTrue(all(require_build_identity(value) == value for value in values))

    def test_explicit_caller_identity_precedes_receipts_and_invalid_input_refuses_before_execution(self):
        import query_gate
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve(); output = root/'out'
            receipt(root, output/'test-cost', 'query')
            with patch.object(query_gate, 'ROOT', root):
                with patch.dict(os.environ, {'WORTH_IMPLEMENTATION_BUILD_ID': SECOND}):
                    gate = query_gate.Gate(output)
                gate.plan(['query-execution'])
                self.assertEqual(gate.env['WORTH_IMPLEMENTATION_BUILD_ID'], SECOND)
                for value in ('bad', '', '00000000-0000-0000-0000-000000000000'):
                    with patch.dict(os.environ, {'WORTH_IMPLEMENTATION_BUILD_ID': value}):
                        with self.assertRaisesRegex(ValueError, 'nonzero UUID'):
                            query_gate.Gate(output)

    def test_direct_governed_run_plans_identity_before_entering_the_existing_execution_owner(self):
        import query_gate
        with tempfile.TemporaryDirectory() as temporary, patch.dict(os.environ, {}, clear=True):
            root = Path(temporary).resolve(); output = root/'out'; output.mkdir()
            receipt(root, output/'test-cost', 'query')
            with patch.object(query_gate, 'ROOT', root):
                gate = query_gate.Gate(output)
                def run(suite):
                    self.assertTrue(gate.cost.configured)
                    self.assertEqual(gate.cost.environment['WORTH_IMPLEMENTATION_BUILD_ID'], FIRST)
                    return {'ok': True}
                with patch.object(gate.cost, 'run', side_effect=run) as owner:
                    self.assertTrue(gate.run('query-execution')['ok'])
                    owner.assert_called_once_with('query-execution')


if __name__ == '__main__':
    unittest.main()
