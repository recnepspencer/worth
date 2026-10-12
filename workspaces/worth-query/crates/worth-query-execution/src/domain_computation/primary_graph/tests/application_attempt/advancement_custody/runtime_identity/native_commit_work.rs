//! Invoice for `admitted_program(..., "closed")` and its idempotency record.
//! Operand counts describe the fixture inputs; no ExecutionReport supplies a
//! price. There are two entity intents in two partitions and no relation edit.

const REGISTRATION_VISIT: u64 = 1;
const RECORD_VISIT: u64 = 1;
const INVARIANT_POINTS: u64 = 3;
// IdentityExecutionSchema plus the platform relation contracts, inspected as
// catalog entries, not charged units. One workflow-custody rule is installed.
const NATIVE_REGISTRATIONS: u64 = 107;
const CUSTOM_REGISTRATIONS: u64 = 1;
const NATIVE_PACKETS: u64 = 2;
const CUSTOM_PACKETS: u64 = 1;
const INVARIANT_PACKETS: u64 = NATIVE_PACKETS + CUSTOM_PACKETS;
const INTENTS: u64 = 2;
const PARTITIONS: u64 = 2;
const CUSTOM_SCOPE_CLONES: u64 = INTENTS;
const CUSTOM_KIND_CHECKS: u64 = 1;
const CUSTOM_JOURNAL_VISITS: u64 = INTENTS;
const CUSTOM_INTENT_VISITS: u64 = INTENTS;

// Account has Policy, Membership and Annotations bindings. Only Policy
// changes: its five declared fields are visited, but only Status is written.
// The new idempotency record has one struct binding with four fields.
const ACCOUNT_BINDINGS: u64 = 3;
const IDEMPOTENCY_BINDINGS: u64 = 1;
const ACCOUNT_POLICY_FIELDS: u64 = 5;
const IDEMPOTENCY_FIELDS: u64 = 4;
const STATUS_WRITES: u64 = 1;
const CHANGED_ASPECTS: u64 = 2;
const CREATED_RECORDS: u64 = 1;
const CHANGED_FIELDS: u64 = STATUS_WRITES + IDEMPOTENCY_FIELDS;
const BINDING_PASSES: u64 = 2;
const POSSIBLE_FIELD_PASSES: u64 = 2;
const DESCRIPTIVE_VISITS: u64 = INTENTS
    + CREATED_RECORDS
    + CHANGED_ASPECTS
    + (ACCOUNT_BINDINGS + IDEMPOTENCY_BINDINGS) * BINDING_PASSES
    + (ACCOUNT_POLICY_FIELDS + IDEMPOTENCY_FIELDS) * POSSIBLE_FIELD_PASSES
    + CHANGED_FIELDS;
// Status and the idempotency key each have one single-field equality index.
const INDEX_LOCATORS: u64 = 2;
const INDEX_LOCATOR_BASE: u64 = 2;
const INDEX_LOCATOR_PASSES: u64 = 2;
const INDEX_KEYS: u64 = INDEX_LOCATORS;
const INDEX_KEY_BASE: u64 = 2;
// The idempotency key is 64 hex bytes plus its comparison-key encoding;
// the old/new Status values fit below the next whole 64-byte chunk.
const INDEX_KEY_CHUNKS: u64 = 1;
const TOUCHES: u64 = CREATED_RECORDS + CHANGED_ASPECTS + CHANGED_FIELDS + INDEX_KEYS;
const TOUCH_ORDERING_PASSES: u64 = (u64::BITS - TOUCHES.leading_zeros()) as u64;

const PUBLICATION_PACKETS: u64 = 1;
const PUBLICATION_FRAGMENTS: u64 = INTENTS;
const FRAGMENT_OPERATIONS: u64 = INTENTS;
// One field-set on Account and one whole-aspect scope addition on idempotency.
const SEMANTIC_CHANGES: u64 = STATUS_WRITES + IDEMPOTENCY_BINDINGS;
const SEMANTIC_CHANGE_PRICE: u64 = 8;
const FRAGMENT_ORDERING_PRICE: u64 = 5;
const BUILTIN_RECORD_CHECKS: u64 = NATIVE_PACKETS * INTENTS * 2;
// Only the two-packet boundary needs sorting; mutation and snapshot are singletons.
const REDUCTION_SORT_STEPS: u64 = 2;

const fn quote() -> u64 {
    let planning =
        INVARIANT_POINTS * (NATIVE_REGISTRATIONS + CUSTOM_REGISTRATIONS) * REGISTRATION_VISIT
            + (INVARIANT_POINTS
                + NATIVE_PACKETS
                + CUSTOM_PACKETS
                + INVARIANT_PACKETS
                + INVARIANT_POINTS * INTENTS
                + INVARIANT_POINTS * PARTITIONS
                + CUSTOM_SCOPE_CLONES
                + CUSTOM_KIND_CHECKS
                + CUSTOM_JOURNAL_VISITS
                + CUSTOM_INTENT_VISITS)
                * RECORD_VISIT;
    let descriptive = DESCRIPTIVE_VISITS * RECORD_VISIT
        + INDEX_LOCATORS * (INDEX_LOCATOR_BASE + INDEX_LOCATOR_PASSES)
        + INDEX_KEYS * INDEX_KEY_BASE
        + INDEX_KEY_CHUNKS * RECORD_VISIT
        + TOUCHES * TOUCH_ORDERING_PASSES;
    let preparation = (INVARIANT_PACKETS
        + PUBLICATION_PACKETS
        + INVARIANT_PACKETS
        + INVARIANT_PACKETS * PARTITIONS
        + PUBLICATION_FRAGMENTS)
        * RECORD_VISIT;
    let dispatch = (INVARIANT_PACKETS + PUBLICATION_PACKETS) * RECORD_VISIT;
    let evaluation = BUILTIN_RECORD_CHECKS * RECORD_VISIT;
    let publication = (PUBLICATION_FRAGMENTS + FRAGMENT_OPERATIONS) * RECORD_VISIT
        + SEMANTIC_CHANGES * SEMANTIC_CHANGE_PRICE
        + PUBLICATION_FRAGMENTS * FRAGMENT_ORDERING_PRICE;
    let merge = (PUBLICATION_PACKETS * 2 + PUBLICATION_FRAGMENTS * 2) * RECORD_VISIT;
    let reduction = (REDUCTION_SORT_STEPS + INVARIANT_PACKETS * 2) * RECORD_VISIT;
    planning + descriptive + preparation + dispatch + evaluation + publication + merge + reduction
}

pub(super) const LEASED_COMMIT_WORK: u64 = quote();
