import json
from pathlib import Path
import tempfile
import unittest

from executed_coverage import ExecutedCoverageFailure, observations, prove, verify


def row(binary='real::lib', test='a', ignored=False, selected=True):
    return dict(binary=binary, test=test, ignored=ignored, selected=selected)


def discovered():
    return {'rust-suites': {
        'real::lib': {'package-name': 'owner', 'binary-name': 'owner_lib', 'kind': 'lib',
                      'testcases': {'a': {'ignored': False}, 'slow': {'ignored': True}}},
        'real::main': {'package-name': 'runner', 'binary-name': 'checks', 'kind': 'test',
                       'harness': False, 'testcases': {'<main>': {'ignored': False}}},
    }}


def event(name='real::lib$a', outcome='ok'):
    return {'type': 'test', 'event': outcome, 'name': name}


def log(root, name, command, events):
    path = root / name
    path.write_text('Command: ' + json.dumps(command) + '\n' +
                    '\n'.join(json.dumps(value) for value in events), encoding='utf-8')
    return path


class ExecutedCoverageTests(unittest.TestCase):
    def test_native_alias_standalone_actual_and_doc_exact_ids_execute_once(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            paths = [log(root, 'native.log', ['cargo', 'nextest', 'run'],
                         [event('owner::owner_lib$a'), event('owner::owner_lib$slow', 'ignored')]),
                     log(root, 'main.log', ['python', str(root / 'account.py'), 'independent.exe'],
                         [event('real::main$<main>')]),
                     log(root, 'doc.log', ['cargo', 'test', '--doc'],
                         [event('owner::doc$src/api.rs - Api (line 17)')])]
            docs = [row('owner::doc', 'src/api.rs - Api (line 17)')]
            result = verify(discovered(), {'native-and-main': discovered()}, paths,
                            document_inventory=docs, require_full_workspace=True)
            self.assertEqual(result['status'], 'pass')
            self.assertEqual((result['executed'], result['passed'], result['scheduled_only']), (3, 3, 1))

    def test_partial_query_is_exact_for_requested_tests_but_not_full_workspace(self):
        known = [row(test='a'), row(test='b')]
        result = prove(known, {'request': known[:1]}, [event()])
        self.assertEqual((result['status'], result['scope']), ('partial', 'selected-subset'))
        self.assertEqual(result['unselected'], [('real::lib', 'b')])
        self.assertEqual(prove(known, {'request': known[:1]}, [event()],
                               require_full_workspace=True)['status'], 'fail')

    def test_filter_misses_are_not_requested_executions(self):
        selected = {'rust-suites': {'real::lib': {'testcases': {
            'a': {'ignored': False, 'filter-match': {'status': 'matches'}},
            'b': {'ignored': False, 'filter-match': {'status': 'mismatch'}}}}}}
        result = prove([row(test='a'), row(test='b')], {'request': selected}, [event()])
        self.assertEqual(result['status'], 'partial')
        self.assertEqual(result['selected'], 1)

    def test_unknown_duplicate_missing_and_unselected_executions_are_refused(self):
        known = [row(), row(test='b')]
        result = prove(known, {'request': known},
                       [event(), event(), event('unknown$test')])
        self.assertEqual(result['status'], 'fail')
        self.assertEqual(result['missing'], [('real::lib', 'b')])
        self.assertEqual(result['multiple'], [('real::lib', 'a')])
        self.assertEqual(result['unknown'], [('unknown', 'test')])
        unexpected = prove(known, {'request': known[:1]}, [event(), event('real::lib$b')])
        self.assertEqual(unexpected['unexpected'], [('real::lib', 'b')])

    def test_duplicate_lane_selection_is_not_excused_by_single_execution(self):
        result = prove([row()], {'one': [row()], 'two': [row()]}, [event()])
        self.assertEqual(result['selection_multiple'], [('real::lib', 'a')])
        self.assertEqual(result['status'], 'fail')

    def test_ignored_observations_are_skipped_but_ignored_execution_is_refused(self):
        known = [row(), row(test='slow', ignored=True)]
        for outcome in ('ok', 'failed'):
            with self.subTest(outcome=outcome):
                result = prove(known, {'lane': known}, [event(), event('real::lib$slow', outcome)])
                self.assertEqual(result['ignored_executed'], [('real::lib', 'slow')])
                self.assertEqual(result['status'], 'fail')
        result = prove(known, {'lane': known}, [event(), event('unknown$ignored', 'ignored')])
        self.assertEqual(result['status'], 'pass')

    def test_failed_execution_is_counted_but_cannot_pass(self):
        result = prove([row()], {'lane': [row()]}, [event(outcome='failed')])
        self.assertEqual(result['executed'], 1)
        self.assertEqual(result['missing'], [])
        self.assertEqual(result['failed'], [('real::lib', 'a')])
        self.assertEqual(result['status'], 'fail')

    def test_selection_cannot_reclassify_an_ignored_test_as_landing(self):
        result = prove([row(ignored=True)], {'lane': [row()]}, [event()])
        self.assertEqual(result['ignored_selected'], [('real::lib', 'a')])
        self.assertEqual(result['status'], 'fail')

    def test_unlisted_inventory_is_not_an_empty_exact_proof(self):
        value = discovered()
        value['rust-suites']['real::lib']['status'] = 'skipped'
        with self.assertRaisesRegex(ValueError, 'not listed'):
            prove(value, {}, [])
        with self.assertRaisesRegex(ValueError, 'not listed'):
            observations([], value)

    def test_canonical_fields_must_agree_with_the_exact_event_name(self):
        actual = dict(event(), binary='real::main', test='<main>')
        with self.assertRaisesRegex(ValueError, 'contradicts'):
            prove([row()], {'lane': [row()]}, [actual])

    def test_refusal_carries_the_reviewable_result(self):
        with self.assertRaises(ExecutedCoverageFailure) as caught:
            verify(discovered(), {'all': discovered()}, [])
        self.assertEqual(len(caught.exception.result['missing']), 2)

    def test_native_alias_must_be_discovered_and_unambiguous(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = log(root, 'run.log', ['cargo', 'nextest', 'run'], [event('real::lib$a')])
            with self.assertRaisesRegex(ValueError, 'unknown native binary alias'):
                observations([path], discovered())
            ambiguous = discovered()
            ambiguous['rust-suites']['other'] = dict(ambiguous['rust-suites']['real::lib'])
            with self.assertRaisesRegex(ValueError, 'ambiguous native'):
                observations([], ambiguous)

    def test_no_binary_identity_cannot_be_proved(self):
        with self.assertRaisesRegex(ValueError, 'no binary identity'):
            prove([row()], {'all': [row()]}, [event('a')])

    def test_retained_log_is_read_once_and_nonexecution_logs_do_not_add_tests(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            build = log(root, 'build.log', ['cargo', 'nextest', 'list'], [event('bogus')])
            path = log(root, 'run.log', ['cargo', 'nextest', 'run'], [event('owner::owner_lib$a')])
            events, walls = observations([build, path], discovered())
            self.assertEqual(len(events), 1)
            self.assertEqual(list(walls), [str(path)])
            with self.assertRaisesRegex(ValueError, 'log repeats'):
                observations([path, path], discovered())


if __name__ == '__main__':
    unittest.main()
