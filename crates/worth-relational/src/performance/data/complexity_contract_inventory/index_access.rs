use super::{ComplexityContract, ComplexityStatus};

pub(super) const ENTITY_FIELD: ComplexityContract = ComplexityContract {
    id: "runtime.query.index_entity_aspect_field_equals",
    function_path: "indexes/access/execution.rs::execute_entity_index_lookup",
    declared_time_complexity: "O(index_hits + matched_record_materialization)",
    budget_summary: "Index-backed field equality reads must resolve candidate IDs from derived index entries, must avoid whole-snapshot materialization, and must report index attempts, parity verification, and emitted record counts explicitly.",
    status: ComplexityStatus::Verified,
    proof_tests: &["tests::complexity::contracts::visibility_budgets::complexity_budget_index_entity_field_equals_avoids_snapshot_materialization"],
};

pub(super) const RELATION_FIELD: ComplexityContract = ComplexityContract {
    id: "runtime.query.index_relation_aspect_field_equals",
    function_path: "indexes/access/execution.rs::execute_relation_index_lookup",
    declared_time_complexity: "O(index_hits + matched_relation_materialization)",
    budget_summary: "Index-backed relation field equality reads must resolve candidate relation IDs from derived index entries, must avoid whole-snapshot relation materialization, and must report index attempts, parity verification, and emitted relation counts explicitly.",
    status: ComplexityStatus::Verified,
    proof_tests: &["tests::complexity::contracts::visibility_budgets::complexity_budget_index_relation_field_equals_avoids_snapshot_materialization"],
};

pub(super) const GENERATION_SELECTION: ComplexityContract = ComplexityContract {
    id: "runtime.index.generation_selection",
    function_path: "runtime/state/subsystems/indexing/generation_catalog/mod.rs::GenerationCatalog",
    declared_time_complexity: "Exact/latest/commit selection: O(log H), one payload; query routing: O(I * log H), at most one candidate per matching definition; commit inventory: O(log H + K * log H), K selected generations",
    budget_summary: "Retained generation count H must not induce an ordinary history inventory or payload copy. Native selection counters charge payload reads and cold inventory enumeration separately. Cold snapshot/fork/checkpoint work and index entry construction are not covered by this selection contract.",
    status: ComplexityStatus::Verified,
    proof_tests: &[
        "runtime::state::subsystems::indexing::generation_catalog::tests::selection_reads_one_payload_independent_of_retained_history",
        "tests::query::indexes::generation_selection_locality::bounded_generation_selection_does_not_enumerate_retained_builds",
    ],
};

pub(super) const PATCH_LOCAL_MAINTENANCE: ComplexityContract = ComplexityContract {
    id: "runtime.index.patch_local_maintenance",
    function_path: "indexes/authority/maintenance/mod.rs::IndexAuthority::refresh_for_basis",
    declared_time_complexity: "O(I + P + affected_adjacency + derived_join_pairs + entry_edits * (log K + log R)); cold reconstruction separately scans bounded live record slots",
    budget_summary: "Canonical patch records and exact roots determine work. Native outcomes count reads, adjacency, row edits, comparisons and persistent path reservations. Missing generations enter an explicit cold slot budget; no ordinary full-root build is selected.",
    status: ComplexityStatus::Verified,
    proof_tests: &[
        "indexes::authority::maintenance::entry_edits::tests::one_edit_in_large_key_and_bucket_population_charges_search_paths",
        "tests::query::indexes::maintenance::patch_local_field_maintenance_matches_independent_rebuild_and_preserves_prior_reader",
        "tests::query::indexes::maintenance::ordering_join_and_relation_field_updates_match_rebuild_after_local_changes",
    ],
};
