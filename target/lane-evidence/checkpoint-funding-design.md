# Automatic Home checkpoint funding ? boundary/design record

Status: diagnosis and design in progress, based atfab7d6e943. D12 admission
cutover remains accepted. The8.192-billion fixture allocation is rejected and
has been removed from the dirty fixture. Host-only implementation compiles;
Native producer handoffs remain pending. Later dated corrections supersede
provisional values and architecture below; the declared work default is1Gi.

## Adversarial constraint and boundaries

A constant live graph can have an arbitrarily long mutation history. Capture
must use the captured current cells and required external obligations, including
mandatory receipt/occurrence state, without replaying unrelated canonical
bodies/working-state mutations. Capture work must have a bound expressed in
actual captured state and bounded tails. Crossing the threshold must never
charge or refuse the business result because maintenance executed.

Source evidence: Native checkpoint_complete/prepare.rs unconditionally calls
FrozenOriginalInputs::read_canonical for the entire catalog, then
FoldedRoots::derive re-applies every merged mutation plan and reconstructs every
historical root. original_image and CanonicalOutput emit the complete prefix.
The accepted admission image itself currently has dense per-canonical meaning
rows (wire/emission.rs requires rows.len==canonical_through). These roles must
not be conflated: the mandatory durable receipt/occurrence image may grow with
retained authoritative STATE; re-reading/re-applying historical canonical
bodies merely to produce that state is not an acceptable bound.

Host maintenance.rs forwards the invoking ExecutionRequest. Query's
advancement/admitted_execution.rs invokes directory_completion::after_business
inside the caller's completion scope. Execution RunLimits inherits the active
meter and rejects unrelated nested leases. Constructing a fresh serial request
on that same stack would not isolate work, memory or interruption authority.
The open request already carries ApplicationHomeCheckpointLimits unchanged;
no new field on P-C's open request is planned. Branch/root_* and World branch
service remain read-only.

## One diagnostic run (review before instrumentation)

Add temporary private phase samples to the Native Complete and Store Home
capture orchestration, using the existing MapKernelContext remaining_work
observation under a single finite explicit request. Samples neither charge nor
alter a request; they report differences of its actual remaining meter.
Report Native source freeze/index resolution, original frame demand+readmission,
dictionary, root replay, branch/schema/initial-input derivation, history/catalog,
actual region images, CHK serialization, source/image sealing, and Store World
image/companion/package framing. Nested hash/serialization/copy samples form a
hierarchical breakdown; do not double-count parent and child totals.

A named real-filesystem Host court creates a41-canonical-position Home with
normal default4096 scheduling threshold and performs one explicit checkpoint
under its own finite diagnostic lease. No fixture cadence/timeout/read-bound
assertion is changed. Run this single court once through owned Cargo/nextest
components after cargo check. Save full log with exact HEAD/dirty/time/counts.
Remove temporary sampling before functional qualification/commit. The permanent
court will become a public work-isolation or state-growth proof, not an exact
cost assertion or a new diagnostic construction door.

## Proposed host funding contract (requires final review after measurement)

The owning runtime is Host. ApplicationHomeCheckpointLimits, beside entries and
payload thresholds, carries named finite maintenance limits: default64Mi work
units,256MiB retained/scratch memory and60s per maintenance advancement. These
are configurable host allocations, not a measured fixture threshold or a
per-original quote. Manual checkpoint/recovery keep the caller's request.

Automatic work uses a Host-owned root serial lease and cancellation domain,
not a child of the triggering request. It needs a genuine framework execution
boundary outside that request's ambient meter. Candidate: a synchronously joined
scoped Host worker, at most one active operation with no unbounded queue; SAME
Store checkpoint owner retains partial attempts and selected/ACK custody.
The worker's frame/backing allocation must also be admitted by the Host budget.
Creation/panic/exhaustion remain maintenance outcomes; business result is kept.
No test-only meter detachment, unlimited grant, arbitrary executor callback or
fresh publication authority is acceptable. Final design will settle the exact
execution boundary and capture-state path after the quantitative attribution.

Delete application_home/checkpoint_resources.rs; Home fixture uses ordinary
world_resources()8M again. Keep unchanged bounded-tail workload, cadence8,
120s backstop and<=40 canonical bodies, with default maintenance allowance.
Add public proof that maintenance budget/stop cannot consume caller work or
replace a committed mutation; explicit checkpoints must still exhaust caller
allowance. Concurrent triggers coalesce through the SAME actual attempt.

## Proposed exact D12 amendment

After Triggers insert:
"Automatic checkpoint capture, publication and acknowledgement are host-owned
maintenance. Home limits declare finite work, memory and deadline allowances,
with documented defaults. This maintenance executes under its own host request;
it never consumes a triggering application's allowance or replaces its result.
Explicit checkpoint calls use their caller's request. Capture processes current
owner state, required live component obligations and bounded uncovered tails;
it does not replay the covered canonical history to reconstruct that state.
Maintenance exhaustion retains the actual pending attempt or due tail and
reports its typed maintenance posture; it does not refuse a committed mutation."

## Remaining evidence needed before functional edits

Quantitative41-position attribution and dominant term; precise state-size bound
and producer replacement; exact worker admission/lifetime/retirement contract;
full design approval. Do not call this initial diagnostic review approval of an
unfinished producer design. No queue item is started before funding lands.

## Diagnostic result and retirement

One instrumented filesystem run completed1/0/0 at head41/covered0; selected
checkpoint generation1 covers41, manifest payload1,324,405 bytes. Actual charge
16,236,875 units. CHK section529,549 bytes; CompleteSource651,914 bytes;
admission image655,343 bytes; World image16,561 bytes; companion261 bytes.
Raw log: checkpoint-funding-instrumented-head-forty-one.log; structured owner
phase/operation attribution: checkpoint-funding-attribution.json.

Temporary court explicit_home_checkpoint_attributes_original_work_at_head_forty_one
is deliberately retired after the user-required single instrumented run. Its
only claim was diagnostic charge attribution, now retained in the log; it was
not a product regression court. This retirement follows the user's one-run
instruction and qa_review_guide.md instrumentation-removal rule. No existing
regression test is deleted, weakened or ignored. All ten instrumentation source
files are now restored/removed. Source-content/mtime changes invalidate their
Cargo fingerprints; next owned rebuild must compile affected owners and scan
the actually executed artifact for the marker before qualification. Do not reuse
the diagnostic executable as production evidence.

The first bare cargo check failed because it lacked the required application
build identity. Fixed invocation uses scripts/application_build.py; corrected
check passes. This is a runner-invocation mistake, not a product failure.

## Refined functional proposal for independent review

The earlier64Mi-work default above was provisional and is superseded here.
Proposed production default:1Gi charged work units,256MiB shared retained/scratch
memory,60s per automatic advancement, one serial worker. This is an explicit
configurable host allocation, not a fixture multiplier, caller-derived allowance
or a theorem that every possible state fits it. Exhaustion remains a typed due/
pending posture. State larger than the declared allowance requires configuring
the Host allowance; increasing caller request policy never funds maintenance.

### Host execution and resource custody

Reuse ApplicationHomeCheckpointLimits (Store owner) rather than add another
public vocabulary type. Its existing new(entries,payload_bytes) sets defaults;
with_maintenance_allowance(work:NonZeroU64,memory_bytes:NonZeroU64,timeout:Duration)
sets three named fields/accessors. Existing Home open request transports the
same limits value unchanged. No open-request field is required.

HomeMaintenance (Host runtime) installs one owned ExecutionResourceLease<'static>
using serial LeaseRequest with the finite maintenance policy and Home-owned
cancellation. Reserve its worker stack/backing from that SAME root ledger while
installing maintenance, before serving; policy/admission refusal can refuse
installation but never a later mutation. Runtime actual memory reservations
retain this root ledger through partial capture/ACK. No per-caller fresh memory
ledger may multiply retained maintenance quota. Proposed worker stack is the
normal2MiB, not an enlarged decoder allowance; admit fixed framework task backing
before spawn as well. Extra metadata uses its actual Layout and the named OS
allocation granule, not an unaccounted escaping allocation.

Automatic callback ignores its invoking ExecutionRequest. A scoped Host worker
runs the SAME Store operation under a deadline-bounded child of the Host root
lease on its own thread, so no invoking meter or worker affinity is inherited.
A runtime-owned atomic admission guard permits one automatic worker; concurrent
triggers coalesce, without a queue. The existing Store attempt remains the only
capture/select/ACK owner. Request creation is inside the worker. Thread creation
and join errors, panics and cleanup are contained by the Host; none may escape
before Query assigns body_result. preflight_home_retirement returns existing
Busy(HomeMaintenance) while the worker is active. Joined work leaves no detached
thread or separate recovery owner. The trait signature may stay unchanged;
its closed Host implementation delegates to a helper accepting only the Host
resources and actual operation, with NO caller request parameter.

Explicit checkpoint/recover_checkpoint APIs still forward their caller request.
No execution/test-only meter-detachment door is introduced. No P-C file changes.

### Native captured-state producer ? proposed exact handoff

Do not keep the original full-history fold as a Home fallback. Native capture
uses actual current/external roots from CapturedCompleteComponents; their actual
source/schema/retention owners remain held through selection. Region emission
already accepts these real root cells and existing carried member placements;
it need not reconstruct them by replaying merged mutation intents.

Covered canonical meaning groups, normalized original-input records, receipt/
association metadata and member locators come from the SAME previously selected
bound capture image. Only uncovered canonical frames are demanded/verified.
First checkpoint has no covered prefix, so all input frames form its initial
journal tail. Retained mandatory receipt/occurrence/creation facts are explicit
captured state; unrelated historical merged plans/body bytes are not. No image
row becomes an original envelope. Input source is an exhaustive private enum:
  CoveredImageRow(selected_base, ordinal) | OriginalTailFrame(verified_original).
Each emitting role consumes that form directly. Dense current wire rows remain
unchanged: old roles are copied from admitted groups and independently resealed
under the new cut; new roles derive from originals. Old canonical members use
actual retained directory/member custody, never New bytes reread from history.

The existing live ACK directory sets admission_image=None, so the current source
has NO way to preserve the freshly produced image for the next live capture.
Cold reopen does preserve image DATA. Avoid a global/runtime-ID cache or a
second source table. Proposed exact owner state:

  // Owning Native runtime; DATA/custody, not a public authority constructor.
  struct SelectedCheckpointCaptureBase {
      source: RelationalCompleteSourceArtifact,
      image: RelationalCheckpointAdmissionArtifact,
      normalized: Arc<NormalizedOriginalInputs>,
  }
  // Capture borrows the actual directory from the SAME owner snapshot.
  struct FrozenSelectedCaptureBase {
      directory: Arc<RelationalCheckpointDirectory>,
      state: Arc<SelectedCheckpointCaptureBase>,
  }

SelectedCheckpointCaptureBase contains no runtime/service/selection pointer and
cannot create a cycle. Its fields retain intrinsic original bills. Constructors
are private and accept either genuine original-owner capture or the already
readmitted exact source/image graph; raw artifact bytes are insufficient.

Proposed Native owner handoff outside this round's named leaf directories:
- runtime/state/subsystems/durability/checkpoint_selection.rs:
  CheckpointSelectionState.capture_base plus corresponding prepared publication
  DATA. Snapshot directory and base atomically; publish them together only in
  actual selected Native ACK; consume on existing source retirement.
- durability/medium/binding/checkpoint_selection.rs: carry the genuine prepared
  base through the SAME pending acknowledgement and partial progress.
- source_initial_inputs/source_admission.rs and runtime_rebuild/original_inputs.rs:
  preserve the already validated selected CompleteSource/normalized graph and
  install it with the already admitted image/directory during cold recovery.
These files are NOT edited by this proposal. They need an exact ownership ruling
if the latest ownership list excludes them. Alternative protected-provider read
would require a NEW owned selected-checkpoint read method; the current ColdSource
and ReadProtection expose only initial/commit/transition/member reads, not a
selected source/image read. Do not encode a provider callback or global cache as
a way around the owner field.

Native checkpoint_complete takes this atomic selected base and a frozen actual
current component/schema/index/dictionary cut. It emits all required captured
state with only bounded uncovered originals. Remove unconditional read_canonical
of the covered prefix, FoldedRoots replay, and re-hashing normalized original
leaves from covered bodies. Branch/schema checks consume actual owner-issued
captured descriptors and source attribution, rather than reconstructing every
historical branch generation. Missing selected base refuses, never falls back.
Standalone original-frame certification remains an explicit demand scope; it
must not masquerade as selected Home image capture.

### Cost contract

Let S be the actual encoded current component/region/dictionary/schema/index
state PLUS mandatory receipt/occurrence/creation/input metadata required by this
capture; K its entries/keys; T the uncovered frame+transition bytes. Capturing
state traverses/copies/hashes O(S+T), with ordered-index construction bounded by
O(K log(K+1)). Peak new scratch/retained storage is O(S+T+K); shared original
backings keep their existing grants. There is no term in covered canonical body
bytes or historical mutation replay. Mandatory retained facts that themselves
grow remain named S, not hidden discarded body history. The implementation must
trace every source traversal to S/K/T; fixed-graph histories with discarded large
updates must not reread their covered frames or execute old mutation plans.

### Proof plan and scope

Public filesystem controls: ordinary8M fixture/default maintenance unchanged
bounded<=40 court; explicit checkpoint stops on a tiny caller budget; tiny Host
allowance leaves due/pending while mutations and subsequent reads succeed;
automatic default capture succeeds even when its charge exceeds caller8M;
postcheckpoint live recapture and cold reopen+tail capture perform ZERO covered
canonical-body reads; retained historical/external roots remain actual; old image
refusals/kill/ACK/CDC/recovery selected cases remain green. Growth proof compares
same required captured state with larger discarded covered body history, generous
headroom, no exact unit assertion. Capture-source/profile artifacts rebuilt and
marker-free before qualification. Default-stack qualification follows decoder
queue item; no ignored/exhaustive/scenario/query-scale tests.

Delete checkpoint_resources.rs and use ordinary world_resources() again. This
removes the rejected fixture policy, not just its comment. Queue untouched.
No wire format/version change is planned; reuse existing image/CompleteSource/
CHK18 meaning rows and exact member placements. No Home open request, home
transitions/discard, World branch service, durable sibling or Native branch/root_*
changes. Functional edits wait for final design review AND required Native owner
handoff. Questions are exact owner scope/capture-base placement, not arbitrary
budget selection.

## Functional design review corrections (no code yet)

The first refined design review REJECT found two real omissions. Both are agreed;
no claim that its Host direction approval certifies the whole implementation.

1. Dense uncovered rows need visibility even when their historical roots are
   not required. Add the narrow branch-owned DATA operation
   branch::checkpoint_visibility::from_original_commit_data(envelope, descriptor,
   request)->Result<RelationalBranchVisibilityCommitment,RootCaptureDenial>.
   It delegates to the existing branch/root_visibility.rs implementation of
   for_root_for_request with storage/schema descriptor axes and the current
   exhaustive correctness posture. It constructs no root, basis or currentness.
   Native calls it only for an actual verified tail original and its matching
   sealed catalog descriptor. Covered rows copy the original admitted visibility
   meaning instead. Do NOT reconstruct intermediate region roots or duplicate
   the hash grammar. A new branch/checkpoint_visibility.rs and crate-private
   branch/mod.rs export need an exact owner handoff. No excluded root_* file
   is edited. New public consumer door is not proposed.
   Public qualification must include first capture and a postcheckpoint tail
   whose intermediate roots have no current/external obligation.

2. Replace the unit-key complexity statement above. Let Lmax be the largest
   comparison-key byte width in actual captured state (text and whole component
   descriptor, including dynamic parent/name data). The conservative bound is
   O(S+T+K*log(K+1)*Lmax) time and O(S+T+K) additional live storage. It names
   repeated byte comparisons explicitly; no constant-cost str/descriptor claim.
   If implementation prepays comparison bounds, it must use checked arithmetic
   and the same Host request. Qualification also varies long common key prefixes
   in actual captured state; discarded-history growth alone is insufficient.

The review also traced an entry gate that must be cut over: Query
advancement/directory_completion::after_business currently calls
request.run_native_completion BEFORE invoking Host maintenance. Caller stop can
therefore suppress the Host callback. The automatic trigger must be independent
of that gate, not only independent inside the worker. Minimal coordinated
Query trigger changes: admitted_execution.rs stores the exact body_result before
upkeep; directory_completion.rs invokes the closed Host hook without opening or
consulting a caller completion scope. Actual original business execution and
its final completion retain their existing request semantics. All Host worker
creation/join/panic-payload cleanup is caught; an upkeep panic cannot enter the
business unwind path. This is a causal Host-trigger seam, not a change to Home
open-request or P-C lifecycle admission/fencing. Final review must check it.

Scope prerequisites now include the five Native selection/ACK/recovery files
already listed plus branch/checkpoint_visibility.rs and branch/mod.rs's exact
crate-private export. No owner edits have occurred. Coordinator scope question
is pending. Default allowance is a stated Host configuration policy; the bound
is the byte-aware function above, not a claim that all legal states fit any
finite policy. Mutations cannot borrow that allocation or be replaced by its
exhaustion; actual due/pending posture remains observable.

The branch adapter name is descriptive DATA: from_original_commit_data. It does
not claim that accepting a public envelope parameter authenticates it. Actual
verified-tail custody is required by the private capture caller; this operation
only computes the existing checksum. It mints no legality/currentness proof.

Resource initialization is before service: a persistent Host root serial lease
has the finite configured policy, no per-call deadline, and the Home cancellation
source. Its reserve_memory admits the worker stack/backing on the OWN root ledger
while installing maintenance. Per-task child deadline creation happens INSIDE
the new worker, never on a caller's ambient meter. Retained captures and future
child jobs share the root memory cap. Startup refusal retains the owned Home
through its existing installation failure path. No new open-request field.


## Host partial implementation and remaining close boundary (2026-11-04)

The implemented post-advancement Host worker uses the declared1Gi work,
256MiB shared memory and60s deadline default. Ordinary8M fixture is restored;
checkpoint_resources.rs is removed. Explicit checkpoint still uses its caller.
Named tiny-budget/explicit control passes1/0/0; caller-stop automatic selection
and the UNCHANGED bounded<=40 filesystem court pass2/0/0. No producer cost bound
is claimed implemented by those passes. Native history cost remains unfixed.

The first Host review found unsanctioned worker creation and an insensitive
refusal-only test. Both are corrected: exact named one-worker site expectation,
and successful automatic selection after caller cancellation with exact value
retained. A second independent review found that a redundant new Host clippy
file copied the wrong ancestor and removed Tokio bans. That file is deleted;
Host inherits workspaces/clippy.toml in full, with no boundary-config additions.

That second review also traced close-trigger capture: Query Host close.rs:154
passes the close request to home.seal_world; Store close/mod.rs:113 passes it to
close_settlement and line155 to worldless_checkpoint. Close and accepted pending
settlement preserve their existing owner's custody, but their request funding is
not changed by this Host post-advancement worker. D12 names close as a checkpoint
trigger. Whole-funding closure must resolve whether/how the close owner carries
the Host allowance without replacing the original close cancellation/admission
controls. Do not claim this partial Host implementation covers close capture.
No Query/Store close or World branch-service file is edited by this slice.

Scoped Host clippy stopped in Query producer/demand/progression/validated/
execution_preparation.rs:51 on &schedule_progression (needless_borrow). Exact
one-line correction requested separately; no bypass/lint allowance installed.
Mandatory boundary returns exactly the3 queued BC9002 sites (cold_source.rs,
protected_reads.rs,runtime_attachment.rs). Agent-context check passes without
regeneration. Dirty Rust caps pass11files via Git Bash; initial bash resolved to
missing WSL /bin/bash, and its failed log is preserved.
