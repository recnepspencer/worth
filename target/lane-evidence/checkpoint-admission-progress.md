# Checkpoint admission implementation progress

## Resume at3d7ccf16b0: D12 admission cutover is NOT committed; D16 remains blocked

Final fresh review: admission_chunked_data_final_review APPROVE for the source
slice3d7ccf16b0..d61b52fe28, no severity findings. This is not full-task approval.
The reviewer independently confirms that operational consumers are still absent.
See admission-data-independent-review.md for its verdict and resolved findings.
No Rust change followed the approved source. No build/test process is running.

Latest correction: d61b52fe288f7a116530573dc19dc87c78167611. Fresh review of
b8c3d86727 rejected a P2 polling violation on unrestricted family strings.
The correction keeps the existing MessagePack string wire grammar. It reuses
the Native slice cursor for streaming bounded scalar fields; validates and
copies UTF-8 families in <=64KiB steps; compares two text windows whose combined
size is <=64KiB; and validates fixed field identifiers without whole unbounded
UTF-8 conversions. Cursor visibility expands only inside the Native crate; no
public facade or boundary-config entry changes. No source body, Home package,
canonical, transition or checkpoint format changes. All current Homes retain
their current opening behavior; the selected predecessor cutoff is unfinished.
The final bounded-family check passes and the nine image public courts pass9/0/0,
including the long UTF-8/tight-stop/cancelled-request case. All28 resume lanes
are terminal, with full runtime logs in admission-data-lanes.md. The first family
check failed on a new test's misuse of run's lease argument as a kernel context;
the fixture now follows the existing nested-control pattern, without changing
the stop assertion. Fresh review of3d7ccf16b0..d61b52fe28 is pending.

Committed prerequisite source: b8c3d867273ef8c1c0eb5d72962f3c2cbfb76c04,
fix(native): bound checkpoint image encoding and selector decoding. It is not
the D12 operational-admission cutover commit. All23 executed resume lanes are
terminal; see admission-data-lanes.md for exact logs and counts. Final scoped
checks pass; corrected pure codec lanes pass3/0/0 plus1/0/0, streaming public
lane7/0/0, and the later affected in-order/publication cases2/0/0. The in-order
check avoids repeatedly charging long ancestor keys; the public fixture now
uses three genuine installed-owner publications and retains the original
single-publication membership assertions. No source code changed after the
last formatter check/test. Fresh review of3d7ccf16b0..b8c3d86727 is pending.

This resume follows the coordinator's queue order. No integration merge, Host
stack fix, D13 restore or remaining provider migration has been performed.
The current batch addresses prerequisites inside the unreleased Native image:

- WRTHCAI1 now encodes fixed byte arrays (including nested digests, roots,
  seals and Home identities) as MessagePack binary through an allocation-free
  serializer adapter. Ordinary sequences and non-byte tuples keep their types.
  The cut's canonical-form check uses the same explicit image policy. Landed
  canonical5, transition2, CHK18 and Home section formats are unchanged.
- Image framing requires the full contiguous covered row count and distinct
  canonical commit keys. A funded lookup index follows the actual input owner.
- The artifact decodes and retains immutable required-output selector DATA,
  including unresolved coverage, without manufacturing a root or publication.
  Decode refuses unknown/out-of-root-prefix commits, duplicate root or route
  keys, malformed fields/tags, unordered keys, excessive depth and non-AVL shape.
  Actual executable root/source/schema admission is still the next boundary.
- Selector parsing uses one exact framing pass and one streaming DeserializeSeed
  pass. Each node is consumed once. The bounded outer roster scans are charged
  explicitly. Actual route strings and Arc layouts are admitted before allocation;
  their grants stay on the actual nodes. Existing live insertion keeps its
  original mandatory grant, carried as Some; image DATA uses the SDK's original
  request retention operation, with None only for native-only accounting mode.

Review/failure resolutions in this batch:
1. Our byte probe initially used Serde's default human-readable flag. A reviewer
   rejected this because a mode-sensitive tuple child could change meaning.
   The probe now receives the actual serializer mode; a pure regression covers it.
2. Our first selector decoder repeatedly scanned child subtrees before recursion.
   A reviewer correctly rejected its n-log-n marker work and missing traversal
   charge. It was replaced by the streaming seed, with explicitly charged scans.
3. native-image-selector-check failed because our code treated OriginalResultMemory
   as a resource lease. The product implementation was wrong; the corrected owner
   uses the existing request retention API rather than inventing a memory account.

Checks so far: native-image-fixed-binary-check PASS; initial two pure codec tests
PASS2/0/0; native-image-selector-check FAIL101 then corrected PASS; initial public
image courts PASS7/0/0; streaming and final funded selector checks PASS. Final
corrected codec/public reruns and fresh review are still being collected.

All D12 operational consumers remain unfinished: root.rs still requires the full
canonical envelope; current-root restore, checkpoint_images, associated effects
and recorded-component admission still demand originals. Store's selected
package and predecessor cutoff are not changed. No current development Home
stops opening as a result of this batch. Native's missing-image guard stays.
The unchanged <=40-body filesystem and zero-member ACK assertions have NOT been
proved. No phase-completion enforcement gate has run for this unfinished phase.

Useful verified seams for the next constructor/package batch:
- Existing proof is worth-proof/src/home_recovery/relational_readmitted.rs; it
  exposes consume, not a value accessor. Consume the concrete private owner once.
- Native directory already owns canonical_read and actual member custody; retain
  its original protected source, never capture a later physical read.
- Store Application checkpoint/preparation.rs is World-first. Store Relational
  section/checkpoint/{complete,publication}.rs currently frames Complete before
  the coupled World cut; the new package must preserve both strong captures.
- Store's actual source reader is checkpoint/selected_source/read.rs and its
  AnchoredRecordReadMembership intrinsically retains the original session, reader,
  protected root and grants. validate_read_membership verifies same serving owner
  and live protection. The package handoff must also retain exact section/range
  correspondence with that same verified read.

Last observed C: free241,707,917,312 bytes. Builds have been sequential; no target
cleanup or stack/profile overrides. No ignored, scale, exhaustive or scenario run.

Approved design:2d33c53b37. Pinned integration ff3fbf9b7f merged as284bd3215a.
Current committed HEAD2fb3d638e1 fixes three public World retirement court
callers with the actual fixture Native owner; no production lifecycle edits.

Mandatory initial checks:
- Root initial FAIL101: three missing Native-owner arguments; fixed in2fb3d638e1.
  admission-start-root-check-tests.log; green successor
  admission-root-check-tests-issued.log (0 tests passed/failed/ignored).
- Query initial FAIL101: missing issued application-build identity in raw cargo
  invocation. Existing scripts/application_build.py supplies it; no product fix.
  admission-start-query-check-tests.log; green successor
  admission-query-check-tests-issued.log (0/0/0).
- Store PASS0: admission-start-store-check-tests.log (0/0/0).
All logs carry exact command/HEAD/clean state/times/exit.

Current batch: repair six held Native courts through actual initialized Homes
and real coupled World/Native checkpoint publication. Stale-capture control
must consume a genuine PreparedCompleteRelationalSection, not discard its input.
Controls stay on the actual receiver. No Bank, retired-ports doc test, or
World/Query lifecycle admission/fencing edits.

Remaining: grants11/11a/11b image producer/admission and all constructor consumers;
Store selection transport; D13 capture-issued registry proof and receiver binding;
grant12 five private assertion retirement and full external provider court
migration; focused proofs, phase gates, independent review. None claimed done.
Builds remain sequential; latest C: free320,046,723,072 bytes; stop below25GiB.
No ignored/query-scale/exhaustive Home oracle/Store scenario lane has run.


2026-11-04 implementation update:
- f16bedf663 retires the five private claims per grant12 and moves the cold-index
  refresh court to an external immutable-member provider. Root scoped
  cargo check --tests is green: public-observation-check-tests.log (0/0/0).
  Runtime execution of this new court/value unit is still pending.
- 503e31d904 and766cf768e7 migrate the six held Store courts to genuine initialized
  or coupled Homes. Corrected scoped Store compile green:
  held-native-courts-check-final.log (0/0/0).
- held-native-courts-first.log:3 pass/3 fail/0 ignored. Three passes are the two
  construction/attachment release cases and original-budget catalog case.
- held-native-courts-three-rerun runtime log: stale capture and foreign runtime
  pass; interrupted ACK remains red at the UNCHANGED zero-member-read assertion.
  Repeating that assertion at Native-only admission still observes5 reads.
  This is a Native checkpoint-admission-image dependency, not World-only reads;
  do not weaken/reset/relabel the counter. Next rerun only after Native repair.
- The overlap failure was an invalid court staging choice: a prepared Store
  Section already holds the single publication-inventory fence. Keep that fence.
  Capture a strong Native Complete early, then frame it at the actual later
  coupled publication call. New narrow control caller entry replaces
  Store Application home_controls_for_test.rs with the exact
  world_section/readmission/controls_for_test.rs for worth-store-relational.
- The foreign speculative fork is refused by Complete's exact
  Native(Capture(OriginalAuthorityUnavailable)); no selected generation changes.
  Release the deliberately substituted owner before closing its source Home.

Native producer/codec is committed in 4215bd9518 and e9a74cafd5, with the
Query denial caller completed in 8e06ce9e12. Complete capture owns the verified original prefix/fold
through a named proof, captures independently hashed role groups, and emits the
new WRTHCAI1/v1 DATA frame under a Home cut. Final output public courts and pure
units pass2/0/0 each; earlier producer and logical-reference courts passed8/0/0
and7/0/0. Fresh fourth independent review APPROVE covers the producer only;
see native-image-producer-independent-review.md. Root admission is NOT implemented.
Original-envelope reads remain at
root_inventory/checkpoint_images.rs, recorded_component_readmission.rs,
associated_effects/readmission.rs and current-root restore. No format has
changed in these implementation commits. Selected predecessor refusal, D13
capture proof, bounded-tail and Host reopen/CDC/recovery qualification are unrun.
All three local adapters and runtime_attachment baseline migration remain open;
no baseline row or root set has grown. Direct holder inventories are saved.
Native/D13 and remaining adapter migration are P-A work, not reassigned debt.

Implementation points to preserve from source tracing:
- RequiredOutputPublicationPositions is a private persistent AVL, not Serialize.
  Add its owner's funded snapshot/readmission; do not reset it to unresolved_at
  on an image root. Capture positions only for actual retained root images;
  canonical row output group keeps original delta/origin/materialization/codec facts.
- Per-row schema can authenticate a carrier reference + schema/version/descriptor
  meaning, with separately authenticated exact schema directory; avoid copying a
  complete schema snapshot N times. All references must close over the same CHK18.
- Scope must stay explicit: Home selected image admission; never-selected Home
  journal originals; explicit standalone Native full-original-frame recovery.
  Never use an original-frame arm as selected Home/image fallback. D13 interim
  movement projections require the separate capture-issued origin proof.
- Foundation/proof contracts are now applied as part of producer work in progress.
- Producer check native-image-producer-check-tests failed on non-Clone row
  backing and missing exhaustive capture-denial projection. Fixed by sharing the
  actual funded artifact backing and preserving typed AdmissionImage recovery
  stops; native-image-producer-check-corrected passed (0 executed tests).
- Fresh producer review REJECT found consult-only encoding/copy/row work. Agreed;
  explicit charged serialization and pre-copy/scan charging now use the same
  original request; focused compile and public role-tampering/binding courts pass.
  Later reviews found runtime-affine DTO slots in predecessor/root headers and
  nested output sources. Logical durable references replace every such occurrence;
  genuine publisher and value-only retirement observations pass. Scope remains
  partial; there is no full-task approval.
- Query caller compile found one missing exhaustive capture-denial arm in
  application_checkpoint/denial.rs; fixed in8e06ce9e12. Corrected Query and Store
  caller checks pass on 8e06ce9e12. The fresh final reviewer APPROVE covers the
  producer and caller step only; see native-image-producer-final-independent-review.md.
- Query D13 concrete seams are application_checkpoint capture/transport,
  primary_graph/bootstrap/{preparation,prepared_native}.rs and
  execution_runtime/product_world/bootstrap.rs, plus the approved original
  publication affinity comparison. Avoid lifecycle admission/fencing and Bank.

Next implementation facts from tracing (not implemented admission):
- rebuild_runtime_from_plan constructs a new restored Native runtime. Bind image
  ownership to that actual receiver, not the builder state that recovery replaces.
  A Weak<RelationalRuntimeState> also makes Arc::get_mut fail while recovery still
  requires unshared_state. Existing owner_binding() carries separate lifecycle
  custody; do not introduce an early state Weak that breaks construction.
- Actual selected directory already owns canonical_read and member basis. Reuse
  its original SelectedCheckpointRead and existing directory retirement/edge
  contracts. Do not recapture the current physical root or create a cycle between
  a directory and its admitted image, or untracked extra business custody.
- canonical_commit_artifact/decoded_quote already quotes semantic, branch,
  required-output, patch and validation owners. Reuse that codec ownership and
  actual nested funding rather than a bytes-times-constant decode quote.
- Native-only ACK refusal observes five MEMBER reads, not a proved breakdown of
  five canonical bodies. home_preflight::preflight_covered_versions loops all
  selected canonical prefixes; trace counters and current-root admission before
  claiming the distribution. Bounded-tail's156 canonical BODY reads are separate
  saved evidence. Preserve both unchanged acceptance observations.
- Logical source references are also nested in required-output origin and codec
  retirement. Their projection is now explicit. Genuine Home codec-retirement
  image capture/admission is still unrun; the pure retirement test claims only
  value encoding, not real publication or recovered custody.
