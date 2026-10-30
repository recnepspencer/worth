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


def validate_command(command):
    if not command:
        return
    executable = str(command[0]).replace('\\', '/').rsplit('/', 1)[-1].lower()
    if executable in ('rustup', 'rustup.exe'):
        raise LaneEnvironmentRefusal('qualification uses the checkout compiler; omit a rustup command override')
    if executable not in ('cargo', 'cargo.exe'):
        return
    boundary = command.index('--') if '--' in command else len(command)
    if any(value == '--config' or value.startswith('--config=') for value in command[1:boundary]):
        raise LaneEnvironmentRefusal(
            'qualification loads committed Cargo configuration from the checkout; omit --config overrides')
    if command[1:2] and command[1].startswith('+'):
        raise LaneEnvironmentRefusal('qualification uses the checkout compiler; omit the Cargo toolchain override')
    if command[1:2] == ['rustc'] and boundary < len(command):
        raise LaneEnvironmentRefusal('qualification uses committed compiler inputs; omit external rustc flags')
