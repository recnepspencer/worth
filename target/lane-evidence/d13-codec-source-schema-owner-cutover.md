# D13 codec source-schema consumer cutover: exact owner seam

## Evidence

Nine selected Query courts in `d13-head-work-query.log` stop before transition
publication with SchemaMismatch / `current source root image is unavailable`.
The unchanged Native function is
`durability/authority/recovery/codec_transition/source_schema.rs::readmit_current_source_schema`.
It selects an inline `checkpoint.branch_roots` row. D13 interim admission instead
returns a selected directory with its authentic admission image and root/schema
roster; it intentionally does not fabricate original root images.

Both `codec_transition.rs` and `codec_transition/source_schema.rs` return1 from
`git diff --quiet 7d0e5189bc codex/m3-p3-lifecycle -- <path>`.
The latter's landed change binds recorded content formats to the available
reader registry. That behavior must remain. No change to this reader is made in
this correction batch; no placeholder inline roots, eager expansion or replay
fallback is proposed.

## Concrete cutover for owner coordination

The two already-distinct Native inputs must remain distinct:

```rust
fn readmit_current_source_schema(
    checkpoint: &DurableCheckpoint, // explicit original-frame demand only
    main: &BranchId,
    available: &RelationalContentFormatRegistry,
) -> Result<RelationalSchemaRegistry, RelationalRecoveryStop>;

fn readmit_interim_current_source_schema(
    request: &ExecutionRequest<'_, '_>,
    origin: &CapturedNativeInterimOrigin,
    checkpoint: &DurableCheckpoint, // produced by decode_origin(origin)
    main: &BranchId,
    available: &RelationalContentFormatRegistry,
) -> Result<RelationalSchemaRegistry, RelationalRecoveryStop>;
```

The image consumer requires its selected directory and exact capture-issued image,
then selects the actual current branch cell's canonical target. Its source schema
role is the authenticated RootDirectory row for that commit/component, not an
arbitrary member of the CHK18 schema union. Match commit, current component axes,
root/schema commitments and carrier; retain the existing content-format binding.
Missing image or mismatched binding keeps typed image refusals. There is no
selected-image fallback to `branch_roots` or to full envelopes.

`restore_interim_checkpoint_for_codec_transition_with_authority` must use this
image entry under one original-request recovery scope. The common codec stage
then receives the already-selected source registry; explicit original-frame
restoration keeps its existing selector. Shared transition/consequence/publication
logic remains one owner. Restore must finish with the same exact-image check and
weak original-DATA installation as ordinary interim restoration, including empty
images, on the actual resulting receiver (codec recovery can replace runtime).
No fresh authority, new World movement or synthetic original input is allowed.

Expected changed owner files: `codec_transition.rs`, its `source_schema.rs`
(or a narrowly split private image-selection leaf under that owner), and the
existing D13 `checkpoint_complete/interim/receiver.rs` for shared original-DATA
installation. Root production files, Home open/lifecycle and idempotency stay out.

Memory/work must be original-request funded at actual decoding/registry owners;
reuse already admitted schema custody where possible. Do not add a new aggregate
capacity work quote. The existing content-format binding and typed refusals are
preserved, not approximated.

## Focused verification after the seam is granted

Select only the9 schema-family failures together (exact names in the Query log),
plus any existing source-head/content-format refusal case the new selector
changes. Compile the complete caller/type cutover before running. Do not rerun
unaffected Query proofs or the Home oracle. This note is a proposed owner cutover,
not implemented or verified work.
