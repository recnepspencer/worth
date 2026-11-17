from pathlib import Path
import unittest
from unittest.mock import patch

from query_gate_cargo import GateCargoRunner
from cargo_capture import CargoRunner
from unit_plan import command_for


class GateCargoPolicyTests(unittest.TestCase):
    def run_command(self, command):
        with patch.object(CargoRunner, '__call__', return_value=0) as execute:
            GateCargoRunner(Path.cwd())(command, Path.cwd(), Path('unused.log'))
        return execute.call_args.args[0]

    def test_build_and_declared_graph_use_the_same_nonincremental_mode(self):
        for command in (['cargo', 'test', '--', 'case'],
                        ['cargo', 'clippy', '--', '-D', 'warnings'],
                        ['cargo', 'nextest', 'list', '--all-targets']):
            with self.subTest(command=command):
                issued = self.run_command(command)
                graph = command_for(issued)
                self.assertIn('build.incremental=false', issued)
                self.assertIn('build.incremental=false', graph)
                self.assertEqual(self.run_command(graph), graph)
                if '--' in command:
                    self.assertEqual(issued[issued.index('--'):], command[command.index('--'):])
                self.assertNotIn('--config', command)

    def test_format_metadata_and_prebuilt_execution_keep_their_arguments(self):
        for command in (['cargo', 'fmt', '--', '--check'],
                        ['cargo', 'metadata', '--format-version', '1'],
                        ['cargo', 'nextest', 'run', '--binaries-metadata', 'binaries.json']):
            with self.subTest(command=command):
                self.assertEqual(self.run_command(command), command)
