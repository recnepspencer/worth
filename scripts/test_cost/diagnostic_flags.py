"""Retain the prior compiler engine's diagnostic flags without a trybuild cfg."""
from contextlib import contextmanager
import json
import os


FLAGS = ['--verbose', '-A', 'dead_code']


@contextmanager
def arguments(runner):
    original = getattr(runner, 'environment', None)
    environment = os.environ if original is None else original
    if 'CARGO_ENCODED_RUSTFLAGS' in environment or 'RUSTFLAGS' in environment:
        if not hasattr(runner, 'environment'):
            raise ValueError('diagnostic runner must own its compiler environment')
        flags = (environment['CARGO_ENCODED_RUSTFLAGS'].split('\x1f')
                 if 'CARGO_ENCODED_RUSTFLAGS' in environment else environment['RUSTFLAGS'].split())
        runner.environment = dict(environment, CARGO_ENCODED_RUSTFLAGS='\x1f'.join(flags + FLAGS))
        try:
            yield []
        finally:
            runner.environment = original
    else:
        # Cargo merges this array with ancestor build.rustflags. Preserve the
        # repository's overflow/windowing flags rather than replacing them.
        yield ['--config', 'build.rustflags=' + json.dumps(FLAGS)]
