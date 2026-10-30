"""Qualification uses the checkout's compiler and committed Cargo configuration."""
import os


class LaneEnvironmentRefusal(ValueError):
    pass


def validate(environment):
    compiler_inputs = {
        'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTDOCFLAGS',
        'CARGO_ENCODED_RUSTDOCFLAGS', 'RUSTUP_TOOLCHAIN', 'RUSTC', 'RUSTDOC',
        'RUSTC_BOOTSTRAP', 'CARGO_INCREMENTAL', 'CARGO_BUILD_RUSTC',
        'CARGO_BUILD_RUSTDOC', 'CARGO_BUILD_RUSTFLAGS', 'CARGO_BUILD_RUSTDOCFLAGS',
        'CARGO_BUILD_INCREMENTAL', 'CARGO_BUILD_TARGET',
    }
    overrides = sorted(name for name in environment if
        name.upper() in compiler_inputs or name.upper().startswith('CARGO_PROFILE_') or
        (name.upper().startswith('CARGO_TARGET_') and name.upper().endswith('_RUSTFLAGS')))
    if overrides:
        raise LaneEnvironmentRefusal(
            'qualification requires committed Cargo profiles/configuration and the checkout compiler; '
            'remove external overrides: ' + ', '.join(overrides))


def inherited():
    environment = os.environ.copy()
    validate(environment)
    return environment
