# Selected runtime results

Native: 58 selected cases passed, 0 failed, 0 selected ignored cases. The original prebuild completed, but its first selection refused a renamed name before executing bodies; the corrected run reused that same owned build and issued build UUID. Public: 9 selected cases passed, 0 failed, 0 selected ignored.

Native nextest's libtest-JSON stream also reports 18 ignored/skipped cases outside the selected packet (including performance and seeded CDC names). Those bodies were not selected or executed: the command has exact nonignored58 filters, no ignored-body flag, and the selection asserts every requested row is nonignored. Its stream contains two partial suite summaries (43 and15 passes); individual successful test events sum to58. Do not sum repeated ignored totals or count the reporter started/ignored markers as executed bodies. The wrapper EVIDENCE_TEST_COUNTS third field is selected ignored, not the full binary's ignored census.

Full source-level selection and raw event output: target/test-cost/pa-integration-native-reused/{selection.json,run.log}; copied owned evidence paths are listed in pa-integration-native-reused.log. Public paths are listed in pa-integration-public.log.
