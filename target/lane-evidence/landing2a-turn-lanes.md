# Landing2a turn lane ledger

Source range `2a51552fc5..2db457e903`. Every top-level command below links its full log; owned build/test raw logs are linked inside the packet logs. Static commands have 0/0/0 test counts. Failed prebuilds executed no tests. Failed runs remain recorded beside their focused corrections.

| Lane log | Exit | Pass/fail/ignored | Command |
|---|---:|---:|---|
| [landing2a-cold-public-format.log](landing2a-cold-public-format.log) | 1 | 0/0/0 | `cargo fmt -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency` |
| [landing2a-cold-public-format-fixed.log](landing2a-cold-public-format-fixed.log) | 0 | 0/0/0 | `cargo fmt -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency` |
| [landing2a-cold-public-check.log](landing2a-cold-public-check.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo check -p worth-court-relational-cold-residency --tests` |
| [landing2a-cold-public-format-batch.log](landing2a-cold-public-format-batch.log) | 0 | 0/0/0 | `cargo fmt -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency` |
| [landing2a-cold-public-proof-root.log](landing2a-cold-public-proof-root.log) | 0 | 6/0/0 | `python target/lane-evidence/landing2a_cold_public_proofs.py root` |
| [landing2a-cold-public-boundary.log](landing2a-cold-public-boundary.log) | 1 | 0/0/0 | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` |
| [landing2a-cold-public-line-caps.log](landing2a-cold-public-line-caps.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-cold-public-context.log](landing2a-cold-public-context.log) | 0 | 0/0/0 | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` |
| [landing2a-cold-public-line-caps-final.log](landing2a-cold-public-line-caps-final.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-cold-public-format-final.log](landing2a-cold-public-format-final.log) | 0 | 0/0/0 | `cargo fmt -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency` |
| [landing2a-cold-public-boundary-final.log](landing2a-cold-public-boundary-final.log) | 1 | 0/0/0 | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` |
| [landing2a-cold-public-line-caps-final2.log](landing2a-cold-public-line-caps-final2.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-lineage-format.log](landing2a-lineage-format.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo fmt -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency` |
| [landing2a-lineage-public-proof-root.log](landing2a-lineage-public-proof-root.log) | 1 | 0/0/0 | `python target/lane-evidence/landing2a_lineage_public_proofs.py root` |
| [landing2a-lineage-format-fixed.log](landing2a-lineage-format-fixed.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo fmt -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency` |
| [landing2a-lineage-public-proof-round2.log](landing2a-lineage-public-proof-round2.log) | 0 | 16/0/0 | `python target/lane-evidence/landing2a_lineage_public_round2.py root` |
| [landing2a-lineage-clippy.log](landing2a-lineage-clippy.log) | 101 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency --all-targets` |
| [landing2a-lineage-line-caps.log](landing2a-lineage-line-caps.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-lineage-format-final.log](landing2a-lineage-format-final.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo fmt -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency` |
| [landing2a-lineage-clippy-final.log](landing2a-lineage-clippy-final.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency --all-targets` |
| [landing2a-lineage-boundary.log](landing2a-lineage-boundary.log) | 1 | 0/0/0 | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` |
| [landing2a-lineage-line-caps-final.log](landing2a-lineage-line-caps-final.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-lineage-context.log](landing2a-lineage-context.log) | 0 | 0/0/0 | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` |
| [landing2a-current-format.log](landing2a-current-format.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo fmt -p worth-court-relational-cold-residency` |
| [landing2a-current-public-proof-root.log](landing2a-current-public-proof-root.log) | 1 | 0/0/0 | `python target/lane-evidence/landing2a_current_public_proofs.py root` |
| [landing2a-current-format-fixed.log](landing2a-current-format-fixed.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo fmt -p worth-relational -p worth-court-relational-cold-residency` |
| [landing2a-current-public-proof-round2.log](landing2a-current-public-proof-round2.log) | 0 | 15/0/0 | `python target/lane-evidence/landing2a_current_public_round2.py root` |
| [landing2a-current-format-final.log](landing2a-current-format-final.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo fmt -p worth-relational -p worth-court-relational-cold-residency` |
| [landing2a-current-clippy.log](landing2a-current-clippy.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational -p worth-court-relational-cold-residency --all-targets` |
| [landing2a-current-format-composition.log](landing2a-current-format-composition.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo fmt -p worth-court-relational-cold-residency` |
| [landing2a-current-court-clippy-final.log](landing2a-current-court-clippy-final.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-court-relational-cold-residency --all-targets` |
| [landing2a-current-boundary.log](landing2a-current-boundary.log) | 1 | 0/0/0 | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` |
| [landing2a-current-line-caps.log](landing2a-current-line-caps.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-current-format-reviewfix.log](landing2a-current-format-reviewfix.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo fmt -p worth-court-relational-cold-residency` |
| [landing2a-current-court-clippy-reviewfix.log](landing2a-current-court-clippy-reviewfix.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-court-relational-cold-residency --all-targets` |
| [landing2a-current-public-proof-reviewfix.log](landing2a-current-public-proof-reviewfix.log) | 0 | 2/0/0 | `python target/lane-evidence/landing2a_current_public_reviewfix.py root residency_failure_matrix_declared_current_doors_preserve_every_failure_class residency_failure_matrix_aspect_history_doors_cover_every_subject_and_class` |
| [landing2a-current-line-caps-reviewfix.log](landing2a-current-line-caps-reviewfix.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-current-boundary-reviewfix.log](landing2a-current-boundary-reviewfix.log) | 1 | 0/0/0 | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` |
| [landing2a-current-context.log](landing2a-current-context.log) | 0 | 0/0/0 | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` |
| [landing2a-retention-public-root.log](landing2a-retention-public-root.log) | 100 | 15/1/0 | `python target/lane-evidence/landing2a_retention_public.py root` |
| [landing2a-retention-public-corrected.log](landing2a-retention-public-corrected.log) | 0 | 2/0/0 | `python target/lane-evidence/landing2a_retention_public.py root fork_capture_preserves_every_original_published_event_in_its_admitted_directory a_cold_lineage_point_read_loads_one_node_page_and_no_region_history` |
| [landing2a-retention-native-coverage.log](landing2a-retention-native-coverage.log) | 0 | 3/0/0 | `python target/lane-evidence/landing2a_retention_public.py native` |
| [landing2a-retention-clippy.log](landing2a-retention-clippy.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency --all-targets` |
| [landing2a-retention-line-caps.log](landing2a-retention-line-caps.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-retention-clippy-final.log](landing2a-retention-clippy-final.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational --all-targets` |
| [landing2a-retention-functions.log](landing2a-retention-functions.log) | 0 | 0/0/0 | `python scripts/quality/scrutinize_rust_functions.py --dirty .` |
| [landing2a-retention-line-caps-final.log](landing2a-retention-line-caps-final.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-retention-boundary.log](landing2a-retention-boundary.log) | 1 | 0/0/0 | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` |
| [landing2a-retention-context.log](landing2a-retention-context.log) | 0 | 0/0/0 | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` |
| [landing2a-retention-format.log](landing2a-retention-format.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo fmt -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency --check` |
| [landing2a-envelope-public-first.log](landing2a-envelope-public-first.log) | 1 | 0/0/0 | `python target/lane-evidence/landing2a_envelope_public.py root` |
| [landing2a-envelope-public-corrected.log](landing2a-envelope-public-corrected.log) | 0 | 12/0/0 | `python target/lane-evidence/landing2a_envelope_public.py root` |
| [landing2a-envelope-clippy.log](landing2a-envelope-clippy.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency --all-targets` |
| [landing2a-envelope-line-caps.log](landing2a-envelope-line-caps.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-envelope-functions.log](landing2a-envelope-functions.log) | 0 | 0/0/0 | `python scripts/quality/scrutinize_rust_functions.py --dirty .` |
| [landing2a-envelope-boundary.log](landing2a-envelope-boundary.log) | 1 | 0/0/0 | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` |
| [landing2a-envelope-context.log](landing2a-envelope-context.log) | 0 | 0/0/0 | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` |
| [landing2a-envelope-format.log](landing2a-envelope-format.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo fmt -p worth-relational -p worth-court-relational-cold-residency --check` |
| [landing2a-full-index-symbol-public-initial.log](landing2a-full-index-symbol-public-initial.log) | 1 | 0/0/0 | `python target/lane-evidence/landing2a_full_index_symbol.py root` |
| [landing2a-full-index-symbol-public-corrected.log](landing2a-full-index-symbol-public-corrected.log) | 100 | 8/1/0 | `python target/lane-evidence/landing2a_full_index_symbol.py root` |
| [landing2a-full-index-symbol-historical-target.log](landing2a-full-index-symbol-historical-target.log) | 0 | 1/0/0 | `python target/lane-evidence/landing2a_full_index_symbol.py root residency_failure_matrix_historical_index_builds_preserve_full_source_failures` |
| [landing2a-full-index-symbol-clippy.log](landing2a-full-index-symbol-clippy.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency --all-targets` |
| [landing2a-full-index-symbol-line-caps.log](landing2a-full-index-symbol-line-caps.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-full-index-symbol-functions.log](landing2a-full-index-symbol-functions.log) | 0 | 0/0/0 | `python scripts/quality/scrutinize_rust_functions.py --dirty .` |
| [landing2a-full-index-symbol-clippy-final.log](landing2a-full-index-symbol-clippy-final.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational -p worth-court-relational-cold-residency --all-targets` |
| [landing2a-full-index-symbol-clippy-clean.log](landing2a-full-index-symbol-clippy-clean.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational --all-targets` |
| [landing2a-full-index-symbol-boundary.log](landing2a-full-index-symbol-boundary.log) | 1 | 0/0/0 | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` |
| [landing2a-full-index-symbol-context.log](landing2a-full-index-symbol-context.log) | 0 | 0/0/0 | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` |
| [landing2a-full-index-symbol-fmt.log](landing2a-full-index-symbol-fmt.log) | 0 | 0/0/0 | `cargo fmt -p worth-relational -p worth-court-relational-cold-residency -- --check` |
| [landing2a-cold-final-public-initial.log](landing2a-cold-final-public-initial.log) | 1 | 0/0/0 | `python target/lane-evidence/landing2a_cold_final_public.py root` |
| [landing2a-cold-final-public-corrected.log](landing2a-cold-final-public-corrected.log) | 0 | 17/0/0 | `python target/lane-evidence/landing2a_cold_final_public.py root` |
| [landing2a-cold-final-public-selected-members.log](landing2a-cold-final-public-selected-members.log) | 0 | 2/0/0 | `python target/lane-evidence/landing2a_cold_final_public.py root checkpoint_restart_cycles_preserve_full_reads_and_retention a_reserved_candidate_published_after_another_branch_checkpoint_keeps_its_pages` |
| [landing2a-cold-final-public-clippy.log](landing2a-cold-final-public-clippy.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency --all-targets` |
| [landing2a-cold-final-public-line-caps.log](landing2a-cold-final-public-line-caps.log) | 0 | 0/0/0 | `"C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty` |
| [landing2a-cold-final-public-functions.log](landing2a-cold-final-public-functions.log) | 0 | 0/0/0 | `python scripts/quality/scrutinize_rust_functions.py --dirty .` |
| [landing2a-cold-final-public-clippy-clean.log](landing2a-cold-final-public-clippy-clean.log) | 0 | 0/0/0 | `python scripts/application_build.py cargo clippy -p worth-relational --all-targets` |
| [landing2a-cold-final-public-boundary.log](landing2a-cold-final-public-boundary.log) | 1 | 0/0/0 | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` |
| [landing2a-cold-final-public-context.log](landing2a-cold-final-public-context.log) | 0 | 0/0/0 | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` |
| [landing2a-cold-final-public-fmt.log](landing2a-cold-final-public-fmt.log) | 0 | 0/0/0 | `cargo fmt -p worth-relational -p worth-testkit-relational -p worth-court-relational-cold-residency -- --check` |

Each linked log records exact command, invocation HEAD, dirty status, UTC start/end, exit and counts. Test packet logs name the immutable owned build/selection/run logs. Evidence precedes the source commit by design; reviewers compared the final source against the logged dirty inputs. No lane is represented as clean-HEAD evidence when it ran dirty.

Unrun in this turn: ignored tests, Query-scale, exhaustive Home oracle, Store scenario suite, fresh Query/Store workspace checks, final broad Tier3 qualification, standalone Complete/ACK gate qualification. Existing green evidence for unchanged inputs is reused. The two retained merge-history courts were not selected in the new packets.

Validation operations: **1 cargo check, 13 Clippy runs, 17 focused builds, 12 selected test runs**. All79top-level lane commands are listed above; nested raw build/test commands are linked by their packet logs.
