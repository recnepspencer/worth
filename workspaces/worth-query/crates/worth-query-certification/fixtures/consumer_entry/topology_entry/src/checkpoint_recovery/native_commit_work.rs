//! Independent quote for the fixed checkpoint fixture's native commit shapes.
//! These are operation counts from its declared schema and staged effects,
//! rather than totals read from an ExecutionReport. Changing the fixture or
//! Relational's unit prices requires changing the corresponding named row.

const REGISTRATION_VISIT: u64 = 1;
const RECORD_VISIT: u64 = 1;
const ADJACENCY_FILTER_PASSES: u64 = 3;
const SEMANTIC_CHANGE: u64 = 8;
const FRAGMENT_ORDERING: u64 = 5;
const INDEX_LOCATOR_BASE: u64 = 2;
const INDEX_LOCATOR_PASSES: u64 = 2;
const INDEX_KEY_BASE: u64 = 2;
const KEY_CHUNK: u64 = 1;
const NATIVE_REGISTRATIONS: u64 = 95;
const CUSTOM_REGISTRATIONS: u64 = 2;
const CUSTOM_ASPECT_BACKINGS: u64 = 18;
const CUSTOM_KIND_LOOKUPS: u64 = 18;
const CUSTOM_FIELD_READS: u64 = 2;

#[derive(Clone, Copy)]
struct Shape {
    commits: u64,
    admitted_native: u64,
    admitted_custom: u64,
    invariant_packets: u64,
    intent_visits: u64,
    partition_visits: u64,
    candidate_entity_slots: u64,
    candidate_relation_slots: u64,
    candidate_intents: u64,
    scope_entity_clones: u64,
    scope_kind_checks: u64,
    scope_readable_kinds: u64,
    scope_relation_kinds: u64,
    scope_relation_reads: u64,
    scope_adjacencies: u64,
    scope_adjacency_candidates: u64,
    scope_seeds: u64,
    scope_seed_candidates: u64,
    scope_entity_journal: u64,
    scope_relation_journal: u64,
    scope_intents: u64,
    rule_scope_steps: u64,
    packet_partition_ids: u64,
    builtin_record_steps: u64,
    custom_evaluations: u64,
    publication_fragments: u64,
    patch_operations: u64,
    semantic_changes: u64,
    delta_binding_field_visits: u64,
    index_locators: u64,
    index_keys: u64,
    index_key_chunks: u64,
    touch_counts: [u64; 3],
    reduction_sort_steps: u64,
}

const fn sorting(visits: u64) -> u64 {
    let width = if visits == 0 { 1 } else { visits };
    visits * (u64::BITS - width.leading_zeros()) as u64
}

const fn quote(s: Shape) -> u64 {
    // Plan, boundary, and snapshot each visit the catalog and the two custom
    // registrations, collect intent partitions, and close their scope once.
    let planning =
        s.commits * 3 * (NATIVE_REGISTRATIONS + CUSTOM_REGISTRATIONS) * REGISTRATION_VISIT
            + (s.admitted_native
                + s.admitted_custom
                + s.invariant_packets
                + s.intent_visits
                + s.partition_visits
                + s.commits * 3)
                * RECORD_VISIT;
    // Only PositivePlanarTurn captures a scope. The workflow-custody rule's
    // kinds are absent: a single footprint serves both registrations and pays
    // one unit per journal slot and intent, without an untouched rule meter.
    let narrowing = (s.candidate_entity_slots + s.candidate_relation_slots + s.candidate_intents)
        * RECORD_VISIT;
    let custom_scope = (s.scope_entity_clones
        + s.scope_kind_checks
        + s.scope_readable_kinds
        + s.scope_relation_kinds
        + s.scope_relation_reads
        + s.scope_adjacencies
        + s.scope_seeds
        + s.scope_entity_journal
        + s.scope_relation_journal
        + s.scope_intents
        + s.rule_scope_steps)
        * RECORD_VISIT
        + (s.scope_adjacency_candidates + s.scope_seed_candidates) * ADJACENCY_FILTER_PASSES;
    // An index reads both old and new authoritative keys. The fixture's paths
    // each have one field; encoded keys additionally pay whole 64-byte chunks.
    let descriptive = s.delta_binding_field_visits * RECORD_VISIT
        + s.index_locators * (INDEX_LOCATOR_BASE + INDEX_LOCATOR_PASSES)
        + s.index_keys * INDEX_KEY_BASE
        + s.index_key_chunks * KEY_CHUNK
        + sorting(s.touch_counts[0])
        + sorting(s.touch_counts[1])
        + sorting(s.touch_counts[2]);
    let packet_preparation = (s.invariant_packets + s.commits // packet claims
        + s.invariant_packets + s.packet_partition_ids // scope fold
        + s.publication_fragments)
        * RECORD_VISIT;
    let dispatch = (s.invariant_packets + s.commits) * RECORD_VISIT;
    // Each executed rule reads eighteen aspect backings, eighteen kinds, and
    // two declared fields. The native view charges these separately from its
    // preparation; no rule's nonzero diagnostic minimum is charged as work.
    let custom_evaluation = s.custom_evaluations
        * (CUSTOM_ASPECT_BACKINGS + CUSTOM_KIND_LOOKUPS + CUSTOM_FIELD_READS)
        * RECORD_VISIT;
    let publication = (s.publication_fragments + s.patch_operations) * RECORD_VISIT
        + s.semantic_changes * SEMANTIC_CHANGE
        + s.publication_fragments * FRAGMENT_ORDERING;
    // One stream per commit: count it, seed it, price its items, and pop them.
    let merge = (s.commits * 2 + s.publication_fragments * 2) * RECORD_VISIT;
    // Strategy diagnostics consult and retain storage, without invariant work.
    let reduction = (s.reduction_sort_steps + s.invariant_packets * 2) * RECORD_VISIT;
    planning
        + narrowing
        + custom_scope
        + descriptive
        + packet_preparation
        + dispatch
        + s.builtin_record_steps * RECORD_VISIT
        + custom_evaluation
        + publication
        + merge
        + reduction
}

// Adjust anchor-a: two merged intents, two partition visits per engine,
// four invariant packets, two patch fragments, and ten descriptive touches.
pub(super) const MUTATION_COMMIT_WORK: u64 = quote(Shape {
    commits: 1,
    admitted_native: 2,
    admitted_custom: 2,
    invariant_packets: 4,
    intent_visits: 6,
    partition_visits: 6,
    // Body and idempotency journal slots; the two merged intents name them.
    candidate_entity_slots: 2,
    candidate_relation_slots: 0,
    candidate_intents: 2,
    scope_entity_clones: 8,
    scope_kind_checks: 1,
    scope_readable_kinds: 24,
    scope_relation_kinds: 1,
    scope_relation_reads: 8,
    scope_adjacencies: 8,
    scope_adjacency_candidates: 8,
    scope_seeds: 1,
    scope_seed_candidates: 2,
    scope_entity_journal: 2,
    scope_relation_journal: 0,
    scope_intents: 2,
    rule_scope_steps: 9,
    packet_partition_ids: 8,
    builtin_record_steps: 24,
    custom_evaluations: 1,
    publication_fragments: 2,
    patch_operations: 2,
    semantic_changes: 2,
    delta_binding_field_visits: 30,
    index_locators: 2,
    index_keys: 2,
    index_key_chunks: 1,
    touch_counts: [10, 0, 0],
    reduction_sort_steps: 4,
});

// One root settlement and two producers: their journals have respectively
// two/seven/seven intents and two/thirteen/thirteen partition visits. They
// publish sixteen fragments, fifteen grammar operations and semantic changes.
pub(super) const ADVANCEMENT_COMMIT_WORK: u64 = quote(Shape {
    commits: 3,
    admitted_native: 14,
    admitted_custom: 6,
    invariant_packets: 20,
    intent_visits: 48,
    partition_visits: 84,
    // The root has two journal entities; each producer has four entities and
    // three relations. Their intents number two/seven/seven. Scope retention
    // excludes the untouched workflow rule's two/one/one partition entries;
    // producer rows and links occupy the same primary partition.
    candidate_entity_slots: 2 + 4 + 4,
    candidate_relation_slots: 0 + 3 + 3,
    candidate_intents: 2 + 7 + 7,
    scope_entity_clones: 34,
    scope_kind_checks: 3,
    scope_readable_kinds: 96,
    scope_relation_kinds: 7,
    scope_relation_reads: 32,
    scope_adjacencies: 32,
    scope_adjacency_candidates: 32,
    scope_seeds: 7,
    scope_seed_candidates: 14,
    scope_entity_journal: 10,
    scope_relation_journal: 6,
    scope_intents: 16,
    rule_scope_steps: 63,
    packet_partition_ids: 24,
    builtin_record_steps: 170,
    custom_evaluations: 3,
    publication_fragments: 16,
    patch_operations: 15,
    semantic_changes: 15,
    delta_binding_field_visits: 188,
    index_locators: 27,
    index_keys: 33,
    index_key_chunks: 9,
    touch_counts: [7, 52, 52],
    reduction_sort_steps: 44,
});
