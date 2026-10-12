//! Work quote for this fixture's two-intent, two-fragment native commit.
const REGISTRATION_VISIT: u64 = 1;
const RECORD_VISIT: u64 = 1;
const NATIVE_REGISTRATIONS: u64 = 84;
const CUSTOM_REGISTRATIONS: u64 = 1;
const INVARIANT_POINTS: u64 = 3;
const INVARIANT_PACKETS: u64 = 3;
const INTENT_VISITS: u64 = 6;
const PARTITION_VISITS: u64 = 6;
// The two journal slots and two merged intents exclude workflow custody's
// declared kinds. Its shared footprint replaces that rule's scope capture.
const CANDIDATE_JOURNAL_VISITS: u64 = 2;
const CANDIDATE_INTENT_VISITS: u64 = 2;
const INCLUDED_NATIVE: u64 = 2;
const INCLUDED_CUSTOM: u64 = 1;
const BUILTIN_RECORD_CHECKS: u64 = 8;
const PUBLICATION_PACKETS: u64 = 1;
const PUBLICATION_FRAGMENTS: u64 = 2;
const FRAGMENT_OPERATIONS: u64 = 2;
const SEMANTIC_CHANGES: u64 = 4;
const SEMANTIC_CHANGE_PRICE: u64 = 8;
const FRAGMENT_ORDERING_PRICE: u64 = 5;
const TOUCH_VISITS: u64 = 38;
const INDEX_LOCATORS: u64 = 4;
const INDEX_PATH_FIELDS: u64 = 1;
const INDEX_LOCATOR_BASE: u64 = 2;
const INDEX_LOCATOR_PASSES: u64 = 2;
const INDEX_KEYS: u64 = 4;
const INDEX_KEY_BASE: u64 = 2;
const INDEX_KEY_CHUNKS: u64 = 1;
const TOUCHES: u64 = 14;
const TOUCH_ORDERING_PASSES: u64 = (u64::BITS - TOUCHES.leading_zeros()) as u64;
const PACKET_SCOPE_PARTITIONS: u64 = 6;
const REDUCTION_SORT_STEPS: u64 = 2;

pub(super) fn quote() -> u64 {
    let planning =
        INVARIANT_POINTS * (NATIVE_REGISTRATIONS + CUSTOM_REGISTRATIONS) * REGISTRATION_VISIT
            + (INVARIANT_POINTS
                + INCLUDED_NATIVE
                + INCLUDED_CUSTOM
                + INVARIANT_PACKETS
                + INTENT_VISITS
                + PARTITION_VISITS
                + CANDIDATE_JOURNAL_VISITS
                + CANDIDATE_INTENT_VISITS)
                * RECORD_VISIT;
    let descriptive = TOUCH_VISITS * RECORD_VISIT
        + INDEX_LOCATORS * (INDEX_LOCATOR_BASE + INDEX_PATH_FIELDS * INDEX_LOCATOR_PASSES)
        + INDEX_KEYS * INDEX_KEY_BASE
        + INDEX_KEY_CHUNKS * RECORD_VISIT
        + TOUCHES * TOUCH_ORDERING_PASSES;
    let preparation = (INVARIANT_PACKETS
        + PUBLICATION_PACKETS
        + INVARIANT_PACKETS
        + PACKET_SCOPE_PARTITIONS
        + PUBLICATION_FRAGMENTS)
        * RECORD_VISIT;
    let dispatch = (INVARIANT_PACKETS + PUBLICATION_PACKETS) * RECORD_VISIT;
    let evaluation = BUILTIN_RECORD_CHECKS * RECORD_VISIT;
    let publication = (PUBLICATION_FRAGMENTS + FRAGMENT_OPERATIONS) * RECORD_VISIT
        + SEMANTIC_CHANGES * SEMANTIC_CHANGE_PRICE
        + PUBLICATION_FRAGMENTS * FRAGMENT_ORDERING_PRICE;
    let merge = (PUBLICATION_PACKETS * 2 + PUBLICATION_FRAGMENTS * 2) * RECORD_VISIT;
    // The reducer sorts packets, collects results and forms decision records.
    // Strategy diagnostics retain memory but charge no invariant work.
    let reduction = (REDUCTION_SORT_STEPS + INVARIANT_PACKETS * 2) * RECORD_VISIT;
    planning + descriptive + preparation + dispatch + evaluation + publication + merge + reduction
}
