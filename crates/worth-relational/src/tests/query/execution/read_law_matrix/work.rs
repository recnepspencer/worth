//! Quotes from fixture cardinalities and the owners' one-unit prices.
// Preparation prices each partition/key/seed/copy and its integer-bit sort.
// Execution prices each examined record, seed and adjacency candidate.
// Completion prices a captured 4096-byte block, each retained node and digest.
const PREPARED_UNIT: u64 = 1;
const EXAMINED_UNIT: u64 = 1;
const CAPTURED_BLOCK: u64 = 1;
const FRAGMENT_NODE: u64 = 1;
const RECORD_NODE: u64 = 1;
const OUTPUT_BUFFER: u64 = 1;
const DIGEST_BLOCK: u64 = 1;
// Visit hashing admits 4 * record backing + 1024 bytes. The fixture's two
// entity aspects fit one 4096-byte block; its four relation aspects need two.
const ENTITY_VISIT_DIGEST_BLOCKS: u64 = 1;
const RELATION_VISIT_DIGEST_BLOCKS: u64 = 2;

pub(super) struct Quote {
    pub preparation: u64,
    pub examined: u64,
    pub records: u64,
    pub captured_blocks: u64,
    pub output_buffers: u64,
    pub traversal: bool,
    pub relation_records: u64,
}

impl Quote {
    pub(super) const fn total(&self) -> u64 {
        let reduction = if self.traversal {
            // One visit digest, one visit node and two deduplication claims per row.
            self.records
                * (ENTITY_VISIT_DIGEST_BLOCKS * DIGEST_BLOCK + RECORD_NODE + 2 * RECORD_NODE)
                + self.relation_records
                    * (RELATION_VISIT_DIGEST_BLOCKS - ENTITY_VISIT_DIGEST_BLOCKS)
                    * DIGEST_BLOCK
        } else {
            self.records * RECORD_NODE + self.output_buffers * OUTPUT_BUFFER
        };
        self.preparation * PREPARED_UNIT
            + self.examined * EXAMINED_UNIT
            + self.captured_blocks * CAPTURED_BLOCK
            + FRAGMENT_NODE
            + reduction
            + self.records * DIGEST_BLOCK
    }
}

// Separate pre-refactor serial-backing literals, in the same named-unit scale.
// These move when serial backing enters with its serial-backed lease.
pub(super) fn serial_literal(name: &str) -> u64 {
    match name {
        "explicit-mixed" => 21,
        "entity-kind" | "entity-aspect" => 13,
        "entity-kind-unscoped" => 14,
        "relation-kind" | "relation-aspect" => 10,
        "entity-equals-failing-rows" => 9,
        "entity-equals-empty" => 7,
        "entity-any-of" => 19,
        "relation-equals-failing-row" => 8,
        "relation-any-of" => 18,
        "outgoing" | "incoming" => 25,
        "connectivity" => 38,
        "depth-zero-cut" => 12,
        other => panic!("missing independent serial quote: {other}"),
    }
}
