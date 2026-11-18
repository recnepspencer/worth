"""Full-gate builds avoid compiler incremental state; edit loops keep Cargo defaults."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_cost'))
from cargo_capture import CargoRunner


class GateCargoRunner(CargoRunner):
    def __call__(self, command, cwd, log_path):
        command = list(command)
        compiles = command[:1] == ['cargo'] and command[1:2] in (
            ['build'], ['check'], ['test'], ['rustc'], ['clippy'], ['run'], ['nextest'])
        if compiles and not any(argument.startswith('--binaries-metadata') for argument in command):
            boundary = command.index('--') if '--' in command else len(command)
            if 'build.incremental=false' not in command[:boundary]:
                command[boundary:boundary] = ['--config', 'build.incremental=false']
        return super().__call__(command, cwd, log_path)
