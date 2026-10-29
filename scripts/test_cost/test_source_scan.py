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

    def test_async_sleeps_are_found_and_literals_cannot_impersonate_calls(self):
        self.assertEqual([s['kind'] for s in scan_text('tokio::time::sleep(delay).await;')], ['sleep'])
        self.assertEqual(scan_text('let text = "Command::new(\\\"cargo\\\")";'), [])

    def test_sleep_poll_and_timeout_locations_are_retained(self):
        sites = scan_text('fn wait() { loop { thread::sleep(Duration::from_secs(4)); } }')
        self.assertEqual({s['kind'] for s in sites}, {'sleep', 'polling-loop', 'fixed-timeout'})


if __name__ == '__main__':
    unittest.main()
