from pathlib import Path
import tempfile
import tomllib
import unittest

from reservations import configuration, original_reservations, recommended
from workspace import nextest_configuration_source


class MeasuredReservationsTests(unittest.TestCase):
    def test_nested_workspace_inherits_repository_ci_deadline_in_generated_config(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / '.config').mkdir()
            (root / '.config/nextest.toml').write_text(
                '[profile.ci]\nslow-timeout={period="60s",terminate-after=10,grace-period="0s"}\n')
            manifest = root / 'workspaces/query/Cargo.toml'
            output = root / 'generated.toml'
            configuration(nextest_configuration_source(root, manifest), output,
                          'ci', {}, root / 'account.py')
            actual = tomllib.loads(output.read_text())
            self.assertEqual(actual['profile']['ci']['slow-timeout'],
                             {'period': '60s', 'terminate-after': 10, 'grace-period': '0s'})
            self.assertEqual(actual['profile']['ci']['scripts'][0]['run-wrapper'], 'test-cost')

    def test_nested_workspace_retains_its_own_deadline_instead_of_repository_policy(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / '.config').mkdir()
            (root / '.config/nextest.toml').write_text(
                '[profile.ci]\nslow-timeout={period="60s",terminate-after=10}\n')
            local = root / 'workspaces/store/.config'
            local.mkdir(parents=True)
            (local / 'nextest.toml').write_text(
                '[profile.ci]\nslow-timeout={period="30s",terminate-after=4}\n')
            output = root / 'generated.toml'
            configuration(nextest_configuration_source(root, local.parent / 'Cargo.toml'),
                          output, 'ci', {}, root / 'account.py')
            actual = tomllib.loads(output.read_text())
            self.assertEqual(actual['profile']['ci']['slow-timeout'],
                             {'period': '30s', 'terminate-after': 4})

    def test_recommendation_uses_peak_observed_cpu_parallelism_without_a_timing_gate(self):
        rows = [dict(binary='court', test='claim', cpu_seconds=cpu, wall_seconds=2,
                     reserved_slots=8, accounting='complete') for cpu in (1, 5)]
        self.assertEqual(recommended(rows), {'court\nclaim': 3})

    def test_generated_config_preserves_timeouts_and_places_measured_rule_first(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / 'original.toml'
            source.write_text('[profile.default]\nslow-timeout={period="60s",terminate-after=10}\n'
                              '[[profile.default.overrides]]\nfilter="all()"\nthreads-required=8\n')
            output = root / 'generated.toml'
            configuration(source, output, 'default', {'court\nclaim': 2}, root / 'account.py')
            value = tomllib.loads(output.read_text())
            self.assertEqual(value['profile']['default']['slow-timeout']['terminate-after'], 10)
            self.assertEqual([row['threads-required'] for row in value['profile']['default']['overrides']], [2, 8])

    def test_nextest_filter_match_precedence_is_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'nextest.toml'
            path.write_text('[[profile.default.overrides]]\nfilter="first"\nthreads-required=2\n'
                            '[[profile.default.overrides]]\nfilter="second"\nthreads-required=8\n')
            rows = [dict(binary='court', test='claim', ignored=False)]
            self.assertEqual(original_reservations(path, 'default', rows, lambda *_: rows), {'court\nclaim': 2})


if __name__ == '__main__':
    unittest.main()
