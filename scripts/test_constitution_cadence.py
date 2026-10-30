"""Phase-end constitution checks stay explicit; real tools prove the boundary."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import time
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONFIGURATIONS = (ROOT / '.claude/settings.json', ROOT / '.codex/hooks.json')
PACKAGE = 'worth-core-cadence'
DEPENDENCY = 'fixture-core-dependency'
POLICY = '''seed_skeletons = []
subworkspaces = [{ path = "." }]
[machine_authority]
canonical_config = "tools/boundary-check/config/road1.toml"
[rule_contracts]
replay_surfaces = []
band_rules = []
[rule_contracts.query_audience]
engine_package = "fixture-engine"
audiences = []
[routing_proof]
exemplars = []
[[source_identifier_denials]]
root = "crates/worth-core-cadence"
forbidden_identifiers = ["DeniedBoundaryCall"]
guidance = "the fixture cannot use this door"
[[dependency_denials]]
workspace_manifest = "Cargo.toml"
sources = ["worth-core-cadence"]
forbidden_targets = ["fixture-core-dependency"]
guidance = "the fixture cannot depend on this package"
'''


def event_commands(settings, event, tool):
    """Small test driver for the settings' actual event/matcher contract."""
    for registration in settings.get('hooks', {}).get(event, []):
        if re.search(registration.get('matcher', '.*'), tool):
            for hook in registration.get('hooks', []):
                yield hook['command']


def prepared_tool(name):
    suffix = '.exe' if os.name == 'nt' else ''
    owner = ROOT / 'tools' / name
    inputs = [owner / 'Cargo.toml', *(path for path in (owner / 'src').rglob('*') if path.is_file())]
    newest_input = max(path.stat().st_mtime_ns for path in inputs)
    paths = (ROOT / 'target/constitution-tools/debug' / (name + suffix),
             owner / 'target/debug' / (name + suffix))
    for path in paths:
        if path.is_file() and path.stat().st_mtime_ns >= newest_input:
            return path
    raise RuntimeError('prepare current constitution tools before this focused test; existing candidates are missing or stale')


class ExplicitFixture:
    def __init__(self, root):
        self.root = root
        self.prepared = root / 'prepared'
        self.package = root / 'crates' / PACKAGE
        self.source = self.package / 'src/behavior.rs'
        self.manifest = self.package / 'Cargo.toml'
        self.write('Cargo.toml', '[workspace]\nmembers = ["crates/*"]\nresolver = "3"\n')
        self.write('.gitignore', '/prepared/\n/target/\n')
        self.write('rust-toolchain.toml', (ROOT / 'rust-toolchain.toml').read_text())
        self.write('.cargo/config.toml', (ROOT / '.cargo/config.toml').read_text())
        for package in (PACKAGE, DEPENDENCY):
            self.write(f'crates/{package}/Cargo.toml',
                       f'[package]\nname = "{package}"\nversion = "0.1.0"\nedition = "2024"\n')
            self.write(f'crates/{package}/src/lib.rs',
                       'mod behavior;\npub mod facade;\n')
            self.write(f'crates/{package}/src/facade.rs',
                       'pub use crate::behavior::permitted;\n')
            self.write(f'crates/{package}/src/behavior.rs', 'pub fn permitted() {}\n')
        config = 'tools/boundary-check/config/'
        for name, text in {
            'road1.toml': POLICY, 'runtime-roots.toml': 'roots = []\n',
            'court-frontier-splits.toml': '', 'consumer-docs.toml': 'documents = []\n',
            'control-callers.txt': '', 'certification-construction-callers.txt': '',
            'test-workspaces.txt': '', 'src-court-baseline.json': '[]\n',
        }.items():
            self.write(config + name, text)
        subprocess.run(['git', 'init', '--quiet', str(root)], check=True)
        subprocess.run(['git', '-C', str(root), 'add', '.'], check=True)
        for name in ('boundary-check', 'agent-context'):
            source = prepared_tool(name)
            destination = self.prepared / 'debug' / source.name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        # Derive real contexts rather than authoring a plausible stand-in.
        generated = subprocess.run(
            [str(prepared_tool('agent-context')), 'generate', '--root', str(root),
             '--config', 'tools/boundary-check/config/road1.toml'],
            capture_output=True, text=True, encoding='utf-8')
        if generated.returncode:
            raise AssertionError(generated.stdout + generated.stderr)

    def write(self, relative, text):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding='utf-8')

    def check(self, prepared=None):
        shell = shutil.which('pwsh') or shutil.which('powershell')
        if not shell:
            raise RuntimeError('PowerShell is required for the explicit launcher test')
        environment = os.environ.copy()
        selected = {'WORTH_CONSTITUTION_PREBUILT': '1',
                    'WORTH_CONSTITUTION_ROOT': str(self.root),
                    'WORTH_CONSTITUTION_TOOL_TARGET': str(prepared or self.prepared)}
        environment.update(selected)
        command = [shell, '-NoProfile', '-File', str(ROOT / 'scripts/check-constitution.ps1'),
                   '--format', 'json']
        start = time.monotonic()
        result = subprocess.run(command, env=environment, capture_output=True,
                                text=True, encoding='utf-8')
        print(json.dumps({'explicit_command': command, 'runtime_inputs': selected,
                          'exit': result.returncode, 'wall_seconds_telemetry': time.monotonic() - start,
                          'stdout': result.stdout, 'stderr': result.stderr}), flush=True)
        return result.returncode, json.loads(result.stdout)


class ConstitutionCadenceTests(unittest.TestCase):
    def test_prepared_tool_selection_prefers_current_session_tools_and_refuses_stale_candidates(self):
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            owner = root / 'tools/boundary-check'
            (owner / 'src').mkdir(parents=True)
            for source in (owner / 'Cargo.toml', owner / 'src/main.rs'):
                source.write_text('fixture input', encoding='utf-8')
                os.utime(source, (200, 200))
            suffix = '.exe' if os.name == 'nt' else ''
            session = root / 'target/constitution-tools/debug' / ('boundary-check' + suffix)
            legacy = owner / 'target/debug' / ('boundary-check' + suffix)
            for path in (session, legacy):
                path.parent.mkdir(parents=True)
                path.write_text('fixture executable', encoding='utf-8')
                os.utime(path, (300, 300))
            with patch(__name__ + '.ROOT', root):
                self.assertEqual(prepared_tool('boundary-check'), session)
                os.utime(session, (100, 100))
                self.assertEqual(prepared_tool('boundary-check'), legacy)
                os.utime(legacy, (100, 100))
                with self.assertRaisesRegex(RuntimeError, 'missing or stale'):
                    prepared_tool('boundary-check')

    def test_edit_events_select_no_automatic_constitution_command(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'source.rs'
            for configuration in CONFIGURATIONS:
                settings = json.loads(configuration.read_text(encoding='utf-8'))
                calls = []
                for tool in ('Write', 'Edit', 'MultiEdit', 'apply_patch', 'Bash'):
                    path.write_text('fn changed() {}\n', encoding='utf-8')
                    calls.extend(event_commands(settings, 'PostToolUse', tool))
                renamed = path.with_name('renamed.rs')
                path.rename(renamed)
                calls.extend(event_commands(settings, 'PostToolUse', 'Bash'))
                renamed.unlink()
                calls.extend(event_commands(settings, 'PostToolUse', 'Bash'))
                constitution = [command for command in calls if any(name in command.lower() for name in
                    ('check-constitution', 'boundary-check', 'agent-context'))]
                self.assertEqual(constitution, [], configuration)
                print(json.dumps({'configuration': str(configuration.relative_to(ROOT)),
                                  'edit_events': 7, 'constitution_commands_selected': len(constitution)}))

    def test_unrelated_hooks_and_existing_preparation_remain_available(self):
        for configuration in CONFIGURATIONS:
            settings = json.loads(configuration.read_text(encoding='utf-8'))
            self.assertIn('powershell -NoProfile -File scripts/prepare-constitution-hook.ps1',
                          list(event_commands(settings, 'SessionStart', 'session')))
            # An unrelated hook at the same event still works; no event system
            # or matcher is globally disabled by the configuration cutover.
            settings['hooks'].setdefault('PostToolUse', []).append({
                'matcher': 'Write', 'hooks': [{'type': 'command', 'command': 'unrelated-hook'}]})
            self.assertEqual(list(event_commands(settings, 'PostToolUse', 'Write')), ['unrelated-hook'])
        self.assertFalse((ROOT / 'scripts/check-constitution-post-tool-use.ps1').exists())
        self.assertTrue((ROOT / 'scripts/check-constitution.ps1').is_file())

    def test_explicit_phase_gate_detects_real_source_dependency_and_generated_context_violations(self):
        with tempfile.TemporaryDirectory(prefix='worth-constitution-cadence-') as temporary:
            fixture = ExplicitFixture(Path(temporary))
            code, report = fixture.check()
            self.assertEqual((code, report['ok'], report['diagnostics']), (0, True, []))
            fixture.source.write_text('pub fn permitted() {}\nfn DeniedBoundaryCall() {}\n', encoding='utf-8')
            code, report = fixture.check()
            self.assertEqual(code, 1)
            self.assertFalse(report['ok'])
            self.assertEqual({d['code'] for d in report['diagnostics']}, {'BC2001_BAND_DEPENDENCY_VIOLATION'})
            self.assertTrue(any('DeniedBoundaryCall' in d['message'] for d in report['diagnostics']))
            fixture.source.write_text('pub fn permitted() {}\n', encoding='utf-8')
            original = fixture.manifest.read_text(encoding='utf-8')
            fixture.manifest.write_text(original +
                f'\n[dependencies]\n{DEPENDENCY} = {{ path = "../{DEPENDENCY}" }}\n', encoding='utf-8')
            code, report = fixture.check()
            self.assertEqual(code, 1)
            self.assertEqual({d['code'] for d in report['diagnostics']}, {'BC2001_BAND_DEPENDENCY_VIOLATION'})
            self.assertTrue(any(DEPENDENCY in d['message'] for d in report['diagnostics']))
            fixture.manifest.write_text(original, encoding='utf-8')
            context = fixture.package / 'AGENT_CONTEXT.md'
            context.write_text('not the generated context\n', encoding='utf-8')
            code, report = fixture.check()
            self.assertEqual(code, 1)
            self.assertEqual({d['code'] for d in report['diagnostics']}, {'CONSTITUTION_AGENT_CONTEXT'})
            code, report = fixture.check(prepared=fixture.root / 'not-prepared')
            self.assertEqual(code, 1)
            self.assertEqual({d['code'] for d in report['diagnostics']}, {'CONSTITUTION_PREBUILT_TOOLS_MISSING'})
            for binary in (fixture.prepared / 'debug').iterdir():
                os.utime(binary, (1, 1))
            code, report = fixture.check()
            self.assertEqual(code, 1)
            self.assertEqual({d['code'] for d in report['diagnostics']}, {'CONSTITUTION_PREBUILT_TOOLS_STALE'})


if __name__ == '__main__':
    unittest.main()
