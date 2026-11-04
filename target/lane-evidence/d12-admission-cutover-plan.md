# D12 operational admission cutover

Starting source: 15730e6b76; clean worktree. The accepted image DATA codec is
unchanged unless an operational consumer needs a concrete addition.

Boundary review: CHK18 directory admission already owns the original selected
read and member custody. Today cold_checkpoint reads distinct heads, and
checkpoint_images, recorded_component_readmission and associated_effects demand
covered originals. These consumers must instead use a distinct admitted-image
commit. Full-frame export, CDC and journal recovery retain their original path.
Image framing alone cannot issue that commit. Native must join its independently
authenticated groups to the actual selected directory and source; Store must
validate the exact package backing/range against the original anchored read.
Existing root identities, executable schemas, actual regions, basis registration,
retention and partial-refusal custody remain the authorities. Missing recovered
root images remain refusals. Image projections cannot mark effects settled.

Implementation order:
1. Native admitted-image owner and exhaustive root evidence; factual consumers
   use receipt/schema/visibility facts, preserving original-only operations.
2. Selected source decoding and current/historical root construction without
   canonical-body demands; publication semantic/created/predecessor consumers.
3. Protected Store package producer/selection and typed predecessor refusal.
   Never-selected Homes retain original-input-plus-Journal recovery.
4. Query accepted-output image section and typed installation handoff. Uncovered
   acceptance must be reconstructed from authoritative journal input or lawful
   derived regeneration, never inferred from locator-only priors.
5. Scoped compile before the unchanged bounded-tail/ACK and named reopen, kill,
   CDC and recovery courts. Record full lane evidence, phase gates, independent
   review, and commit only the proved cutover.

Out of scope this turn: decode_payload frame repair, D13 memory registry,
BC9002, and the queue file. Avoid Lifecycle's named installation/session files.
No integration merge is required by this turn's scope. One build at a time;
disk floor 25 GiB. Initial C: free 237330407424 bytes.

## Consumer audit extension during filesystem qualification

The first Host lane passes6 named cases but fails the two old-key controls and
the bounded writer's shared authentication deadline. The latter is a fixture
lifetime error: distinct requests now receive distinct scopes; workload and
the unchanged <=40 observation are preserved. Independent read-only source
review found the old-key cause in Native receipt-only settlement repair, after
successful Query receipt reconstruction. Native still required a resident
PositionedCanonicalCommit. Cut over only its no-pending-record branch to actual
bound image/catalog custody; preserve pending/closed-owner semantics and
CheckpointReadmitted versus PerformedSettled. No receipt/idempotency file edit.

The same review found cold index refresh requiring an envelope. The direct
root-envelope inventory (`rg -n '\.canonical_envelope\(' crates/worth-relational/src
-g '*.rs'`) identifies maintenance admission, authoring and changed-record
consumers. Prepared candidate-only, original frame/patch export and history-body
operations retain their original contract. Index refresh now uses exact root
receipt/authoring facts, authenticates any actual before root against the image's
original predecessor, and takes changed identities from verified semantic
meaning. Both original and image request stops keep their category. Native
public court proves missing-generation reconstruction and zero-population
incremental refresh, a mismatched before-root refusal, real lookup values and
zero covered canonical body reads. It is compiled, runtime qualification pending.

New exact consumer files: publication/authority/pending_settlement.rs;
indexes/authority/maintenance/{basis,changes,refresh_preparation,mod}.rs;
checkpoint_admission/admitted_commit/index_predecessor.rs. These are causal
operational-image consumers; no retired original-frame contract is emulated.

## Coordinator scope amendment read2026-11-04

The appended WITHDRAWN accepted-output ruling in p3_reopen_queue.md supersedes
implementation step4 above: Phase3 directory Homes are Relational-only; Phase6
owns accepted-output reuse/restoration/regeneration. No carrier was implemented.
This does not affect Native canonical required-output provenance in the image.
The next-turn queue remains untouched in this worktree.
