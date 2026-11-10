# Original-input attribution: minimal immutable owner observation

Boundary evidence: history-compiler-claim-disposition.md (three actual merge
recovery roots) and other-consumer-claim-disposition.md (aggregate logical
checkpoint). `tests/support/truth_comparison.rs:60` compares both complete durable
body projection and `recorded_original_inputs()`. Public envelope Serialize
exposes body/semantic result but omits that optional attribution. Source
`history/data/canonical_commit_envelope/original_inputs.rs` reads the OnceLock;
`durability/source_initial_inputs/recorded.rs` defines all four recorded fields.
The existing source commitment-leaf sealed test door hashes another object and
is not an equivalent observation. Native Memory would vacuously lose evidence.

Proposed class-3 public data observation, no authority or mutation door:

```rust
// Native history/data owner, exported by facade::history.
#[derive(Debug, PartialEq, Eq)]
pub struct RelationalOriginalInputObservation<'a> {
    prestate_schema: worth_foundational::CanonicalDigestId,
    result_schema: worth_foundational::CanonicalDigestId,
    introduced_schema: Option<RelationalIntroducedSchemaObservation<'a>>,
    cascade_delete_policy: &'a CascadeDeletePolicy,
    adjacency_policy: &'a AdjacencyPolicy,
    cross_context_policy: &'a CrossContextPolicy,
}
// Borrowed, private-field owner view; no public constructor or wire traits.
#[derive(PartialEq, Eq)]
pub struct RelationalIntroducedSchemaObservation<'a>(
    &'a DurableBranchRootSchemaImage,
);
impl RelationalIntroducedSchemaObservation<'_> {
    pub fn carrier_digest(&self) -> worth_foundational::CanonicalDigestId;
    pub fn schema_root(&self) -> worth_foundational::CanonicalDigestId;
    pub fn descriptor_semantics_version(&self) -> DescriptorSemanticsVersion;
}
impl CanonicalCommitEnvelope {
    pub fn original_input_observation(&self)
        -> Option<RelationalOriginalInputObservation<'_>>;
}
// Read-only named accessors for every OriginalInputObservation field.
// Projection destructures MutationConfig exhaustively (no ..), retaining all
// three policies; AdjacencyPolicy is borrowed, never cloned.
```


This is Native-specific descriptive inspection of the actual envelope's frozen
attribution, borrowing its original DATA. It has no cross-runtime authority,
clock/table/Drop, copied schema owner or reconstructed commitment. Existing named
CanonicalDigestId is already used for schema carriers in checkpoint admission;
Native already depends on Foundation. Both original MutationConfig and durable
schema image are crate-private. They remain private: only existing public policy
types and the borrowed semantic image view cross the facade. The view exposes
stored carrier/root/version metadata; its derived Eq delegates to exact actual
whole-image equality, including every registration/contract/domain/content field.
It does not publish a schema wire/Clone/Deserialize or owned-image constructor.
Its Debug prints semantic metadata only, never private wire/reservation state.

Projection and metadata getters are O(1), two fixed-width digest copies and
borrowed references. They perform no allocation/body demand. Envelope acquisition
remains on the existing request-funded replay/history read. Optional-image Eq is
O(schema DATA) through the existing exact image equality; no O(1) equality claim
is made. Equality preserves all original fields and optional attribution exactly;
no canonical leaf, hash-only match or body-only proxy substitutes. Public flow
courts retain actual source+receiver media and assert original attribution is
present before comparing both observations. Value-only encoding units may remain
true owner units without claiming physical reopen.

Add the inspection capability to the index. Port affected actual public journeys
with whole-body MessagePack projection excluding only diagnostic entries and
derived cache DATA, exactly as the original helper; reuse independent DATA spans
in the existing certification support, not lossy JSON (structured keys/binary
values matter). Keep typed/fate refusals and all semantic_result fields.
Independent review verifies each body field is serialized or independently
compared; attribution covers optional whole image, both schemas and all three
original mutation policies. This is the classification-3 observation recorded
before code, no second merge/transaction/installation/publication/recovery path.

Independent design P1 correction: initial proposal referenced private field
 types directly. It is replaced by these borrowed semantic views; no persisted
 type visibility or codec surface changes. Cost wording separates projection
 from whole-DATA Eq, and non-Copy adjacency policy is borrowed.
