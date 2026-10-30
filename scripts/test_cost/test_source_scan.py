import unittest

from source_scan import scan_text


class SourceCostTests(unittest.TestCase):
    def test_direct_cargo_environment_metadata_and_trybuild_pass_are_found(self):
        text = '''
        // Command::new("cargo");
        fn boundary() {
            let c = Command :: new(std::env::var_os("CARGO").unwrap());
            let cases = trybuild :: TestCases :: new();
            cases.pass("valid.rs");
            cargo_metadata::MetadataCommand::new();
        }
        '''
        sites = scan_text(text)
        self.assertEqual({s['kind'] for s in sites},
                         {'cargo-command', 'cargo-environment', 'trybuild',
                          'trybuild-pass', 'cargo-metadata'})
        self.assertTrue(all(s['function'] == 'boundary' for s in sites))

    def test_comments_cannot_create_cost_sites(self):
        self.assertEqual(scan_text('// trybuild::TestCases\n/* thread::sleep( */'), [])

    def test_domain_pass_methods_are_not_mislabeled_as_compiler_sessions(self):
        self.assertEqual(scan_text('fn claim() { answer.pass(value); }'), [])
        sites = scan_text('fn claim(cases: &trybuild::TestCases) { cases.pass("yes.rs"); }')
        self.assertIn('trybuild-pass', [site['kind'] for site in sites])

    def test_named_executable_and_imported_command_alias_cannot_bypass_the_rule(self):
        for text in ['fn proof() { let executable="cargo"; Command::new(executable); }',
                     'const EXECUTABLE: &str = "cargo"; fn proof() { Command::new(EXECUTABLE); }',
                     'static EXECUTABLE: &str = "cargo.exe"; fn proof() { Command::new(EXECUTABLE); }',
                     'use std::process::Command as Process; fn proof() { Process::new("cargo.exe"); }']:
            self.assertIn('cargo-command', [site['kind'] for site in scan_text(text)])

    def test_absolute_and_case_variant_cargo_executables_cannot_bypass_the_rule(self):
        for executable in ['r"C:\\Rust\\bin\\cargo.exe"', '"/opt/rust/bin/cargo"',
                           '"cargo.EXE"', 'r"\\\\server\\tools\\Cargo.Exe"', '"CARGO"']:
            for expression in [f'Command::new({executable});',
                               f'let executable={executable}; Command::new(executable);',
                               f'use std::process::Command as Process; Process::new({executable});']:
                with self.subTest(expression=expression):
                    self.assertIn('cargo-command', [site['kind'] for site in scan_text(expression)])
        self.assertEqual(scan_text('Command::new("/opt/bin/cargo-harness");'), [])

    def test_async_sleeps_are_found_and_literals_cannot_impersonate_calls(self):
        self.assertEqual([s['kind'] for s in scan_text('tokio::time::sleep(delay).await;')], ['sleep'])
        self.assertEqual(scan_text('let text = "Command::new(\\\"cargo\\\")";'), [])

    def test_enclosing_command_aliases_and_executable_constants_are_resolved(self):
        for context, body in [
            ('use std::process::Command as Process;', 'Process::new("cargo");'),
            ('const EXECUTABLE:&str="cargo";', 'Command::new(EXECUTABLE);'),
            ('const EXECUTABLE:&str=env!("CARGO");', 'Command::new(EXECUTABLE);'),
            ('type Process=std::process::Command;', 'Process::new("cargo");'),
        ]:
            with self.subTest(context=context):
                self.assertIn('cargo-command', [row['kind'] for row in scan_text(body, context)])

    def test_enclosing_grouped_and_crate_aliases_retain_compiler_api_identity(self):
        for context, body, expected in [
            ('use trybuild::{TestCases as Cases};', 'let cases=Cases::new(); cases.pass("yes.rs");',
             {'trybuild', 'trybuild-pass'}),
            ('use trybuild as tb;', 'tb::TestCases::new();', {'trybuild'}),
            ('use cargo_metadata::{MetadataCommand as Metadata};', 'Metadata::new();', {'cargo-metadata'}),
        ]:
            with self.subTest(context=context):
                self.assertEqual({row['kind'] for row in scan_text(body, context)}, expected)

    def test_unused_production_context_does_not_become_a_test_invocation(self):
        self.assertEqual(scan_text('fn claim() {}', 'fn ordinary() { Command::new("cargo"); }'), [])

    def test_same_line_compiler_invocations_remain_distinct_sites(self):
        rows = scan_text('Process::new("cargo"); Process::new("cargo");',
                         'use std::process::Command as Process;')
        self.assertEqual([row['kind'] for row in rows], ['cargo-command', 'cargo-command'])

    def test_sleep_poll_and_timeout_locations_are_retained(self):
        sites = scan_text('fn wait() { loop { thread::sleep(Duration::from_secs(4)); } }')
        self.assertEqual({s['kind'] for s in sites}, {'sleep', 'polling-loop', 'fixed-timeout'})


if __name__ == '__main__':
    unittest.main()
