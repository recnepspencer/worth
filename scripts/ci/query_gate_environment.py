"""One immutable environment for a complete Query gate build and its tests."""
import os
import json
from pathlib import Path
import shutil
import sys
import uuid

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from application_build import application_build_environment
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_cost'))
from workspace import WORKSPACES


def require_build_identity(value):
    if not isinstance(value, str) or len(value) != 36:
        raise ValueError('WORTH_IMPLEMENTATION_BUILD_ID must be a nonzero UUID')
    try:
        parsed = uuid.UUID(value)
    except ValueError:
        raise ValueError('WORTH_IMPLEMENTATION_BUILD_ID must be a nonzero UUID') from None
    if not parsed.int or str(parsed) != value.lower():
        raise ValueError('WORTH_IMPLEMENTATION_BUILD_ID must be a nonzero UUID')
    return value


def verification_identity(root, output, workspaces):
    """Read issued native-build inputs; this never qualifies an old execution."""
    identities = {}
    for workspace in sorted(workspaces):
        manifest = (Path(root) / WORKSPACES[workspace]).resolve()
        receipt = Path(output) / workspace / 'build/binaries-build-units.json'
        try:
            plan = json.loads(receipt.read_text(encoding='utf-8'))
            if type(plan.get('schema')) is not int or plan['schema'] != 1 or plan['type'] != 'compiler-unit-plan':
                return None
            contract = plan['contract']
            command = contract['command']
            if command[:3] != ['cargo', 'nextest', 'list'] or Path(contract['cwd']).resolve() != manifest.parent:
                return None
            if Path(command[command.index('--manifest-path') + 1]).resolve() != manifest:
                return None
            if command[command.index('--list-type') + 1] != 'binaries-only':
                return None
            identity = require_build_identity(contract['compiler_inputs']['environment']['WORTH_IMPLEMENTATION_BUILD_ID'])
            graph = plan.get('graph')
            if (not isinstance(plan.get('units'), list) or not isinstance(graph, dict)
                    or type(graph.get('version')) is not int or graph['version'] != 1
                    or not isinstance(graph.get('units'), list) or not isinstance(graph.get('roots'), list)):
                return None
        except (OSError, ValueError, KeyError, TypeError, IndexError, AttributeError):
            return None
        identities.setdefault(identity.lower(), identity)
    return next(iter(identities.values())) if len(identities) == 1 else None


def require_normal_identity(root, output, expected, commands, suites):
    """Bind this run to existing compiler-input owners; no execution reuse."""
    expected = require_build_identity(expected).lower()
    required = set()
    if any(suite != 'root-doc' for suite in suites):
        issued = verification_identity(root, Path(output) / 'test-cost', ['root'])
        if issued is None or issued.lower() != expected:
            raise ValueError('normal native build identity differs or is missing')
        required.add('test-cost/root/build/binaries-build-units.json')
    if 'root-doc' in suites:
        required.add('test-cost/root/documentation/doctests-units.json')
    from unit_plan import command_for
    from logs import json_objects
    for rows in commands.values():
        required.update(label + '-units.json' for label, argv in rows if command_for(argv) is not None)
    paths = set(Path(output).rglob('*-units.json'))
    if not {Path(output)/name for name in required} <= paths:
        raise ValueError('normal compiler-input artifact is missing')
    for path in paths:
        try:
            value = json.loads(path.read_text(encoding='utf-8'))
            contract = value['contract']
            identity = require_build_identity(
                contract['compiler_inputs']['environment']['WORTH_IMPLEMENTATION_BUILD_ID'])
            if (type(value.get('schema')) is not int or value['schema'] != 1 or
                    value.get('type') != 'compiler-unit-plan' or identity.lower() != expected):
                raise ValueError('normal compiler build identity differs')
            log = path.with_name(path.name.removesuffix('-units.json') + '.log')
            invocations = [row for row in json_objects(log.read_text(encoding='utf-8'))
                           if row.get('type') == 'cargo-invocation' and 'compiler_contract' in row]
            if len(invocations) != 1 or invocations[0]['compiler_contract'] != contract:
                raise ValueError('normal compiler invocation differs from its declared inputs')
        except (KeyError, TypeError, AttributeError) as error:
            raise ValueError('normal compiler identity is missing') from error


def environment(root):
    # Gate.plan may reuse an issued verification input before any Cargo work.
    env = application_build_environment(os.environ)
    # Test binaries are run directly, so cargo's [env] table does not reach them.
    env.setdefault("RUST_MIN_STACK", "16777216")
    env.setdefault("WORTH_REPO", str(root))
    env.setdefault("CARGO", shutil.which("cargo") or "cargo")
    if shutil.which("sccache") and "RUSTC_WRAPPER" not in env:
        if env.get("CARGO_INCREMENTAL") == "1":
            del env["CARGO_INCREMENTAL"]
        env["RUSTC_WRAPPER"] = "sccache"
    return env
