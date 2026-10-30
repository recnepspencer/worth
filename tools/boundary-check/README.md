# Boundary checker

The checker reads Git-listed tracked and untracked inputs once, requests one
resolved offline Cargo graph per governed workspace in scoped threads, then runs
pure rule groups in parallel. It never walks the repository. Only the input
module launches processes. Diagnostics are sorted and deduplicated before human
or JSON rendering.

Prepare both constitutional tools with `scripts/prepare-constitution-hook.ps1`.
Use `WORTH_CONSTITUTION_PREBUILT=1 ./scripts/check-constitution.ps1` for lanes.
The standalone checker accepts `--root`, `--config`, and `--format human|json`.
Input failures retain a separate outcome: exit 2 and the engine code
`BOUNDARY_INPUT_FAILURE`. Checked violations exit 1; successful checks exit 0.
Its JSON output is an array of `code`, `subject`, `message`, and `legal_home`
records; a clean run produces `[]`. The constitution script retains its existing
envelope and prebuilt freshness checks.

Run the in-memory suite with:

```text
cargo test --manifest-path tools/boundary-check/Cargo.toml --target-dir target/constitution-tools
```

## Retained requirements and evidence

The approved scope is the owner's external `orchestration/m3/boundary_triage.md`.
Every rule has a positive and negative in-memory case; tests launch no processes
and construct no disk repositories.

| Requirement | Behaviour tests |
| --- | --- |
| BC2001 graph policy | `dependency_denials_and_allowlists_reject_only_disallowed_edges`, `dependency_contracts_preserve_exact_versions_defaults_and_features` |
| BC2001 identifier denials | `identifier_denials_respect_exclusions_and_token_boundaries` |
| BC7009 root Clippy superset | `local_clippy_configuration_carries_every_root_entry` |
| BC7012 closed control callers | `exported_control_references_require_an_exact_caller`, `raw_identifiers_do_not_bypass_the_closed_control_caller_set`, `crate_visibility_spellings_keep_the_same_closed_callers`, `exported_macros_keep_closed_callers_even_inside_private_functions`, `composed_controls_and_reexporting_owners_keep_their_existing_reach`, `test_only_methods_do_not_seal_unrelated_namesakes`, `macro_controls_require_literal_exported_declarations` |
| BC7013 construction callers | `certification_factory_callers_are_class_and_file_exact` |
| BC8008 compiled regions and source facts | `compiled_regions_compare_exact_code_and_require_unique_delimiters`, `a_compiled_marker_requires_a_following_code_region`, `documented_enum_rows_detect_payload_changes_as_well_as_variants` |
| BC8009 relative links | `relative_links_require_targets_and_unique_heading_anchors`, `shortcut_reference_links_check_defined_targets_without_treating_definitions_as_links` |
| BC9001 one build | `features_are_refused_except_for_declared_signal_profiles`, `feature_predicates_are_detected_without_matching_quoted_examples` |
| BC9002 court topology | `courts_have_one_canonical_target_and_remain_dependency_leaves`, `products_cannot_hide_integration_tests_in_the_file_list`, `product_dependencies_on_testkits_are_dev_only`, `categories_follow_top_level_directory_segments`, `frontier_uses_reachable_product_roots_and_excludes_dependency_dev_edges`, `area_splits_require_explicit_delegation_of_nested_owners` |
| Law 2 runtime roots | `runtime_mentions_are_scoped_to_inline_and_mounted_test_modules`, `inline_test_modules_follow_all_mounted_descendants` |

The runtime-root ratchet records `(source, module, roots)`, where `roots` is the
sorted set of registered type names mentioned as Rust tokens within that test
module. Inline and file-mounted `#[cfg(test)]` modules are covered. This replaces
the constructor tracer: comments and literal bodies do not count; method calls
are not traced through inferred receivers. Removing findings is permitted;
new module/type mentions require an explicit baseline decision. The cutover
baseline contains 2,943 module/root-set records.

The old disk fixtures and integration tests were retired by the approved triage's
explicit instruction to delete the old tool tests and fixtures. They are replaced
by the in-memory cases above. The facade/debt snapshots and their ratchet tests
were likewise retired by that approved scope. Agent-context now lists declarations
from facade source instead of reading the deleted snapshot; glob routes remain
explicit, and the compiler owns their resolution.
