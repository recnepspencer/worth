"""Layer measured compiler options without replacing caller or repository flags."""
from contextlib import contextmanager
import json
import os


@contextmanager
def arguments(runner, flags, *, environment=None):
    flags = list(flags)
    if any(not isinstance(flag, str) or not flag or '\x00' in flag for flag in flags):
        raise ValueError('compiler flags must be nonempty argument strings')
    original = getattr(runner, 'environment', None)
    inherited = os.environ if original is None else original
    encoded = 'CARGO_ENCODED_RUSTFLAGS' in inherited or 'RUSTFLAGS' in inherited
    replace = encoded or bool(environment)
    if replace and not hasattr(runner, 'environment'):
        raise ValueError('compiler runner must own its environment')
    if replace:
        changed = dict(inherited)
        if encoded:
            if 'CARGO_ENCODED_RUSTFLAGS' in inherited:
                previous = inherited['CARGO_ENCODED_RUSTFLAGS']
                tokens = previous.split('\x1f') if previous else []
            else:
                tokens = inherited['RUSTFLAGS'].split()
            changed['CARGO_ENCODED_RUSTFLAGS'] = '\x1f'.join(tokens + flags)
        changed.update(environment or {})
        runner.environment = changed
    try:
        # Cargo merges config arrays with inherited repository build.rustflags.
        yield [] if encoded else ['--config', 'build.rustflags=' + json.dumps(flags)]
    finally:
        if replace:
            runner.environment = original
