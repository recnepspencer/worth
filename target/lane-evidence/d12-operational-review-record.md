# Operational admission targeted reviews

Base:15730e6b767f21e8a2d2982849cf9ed2876d16cf; uncommitted cutover slice.
These are targeted source reviews, not full-task certification.

## Earlier findings and resolutions

1. High: replacing a selected directory lost the original binding required by
   retained checkpoint roots. Agreed. The original admission provenance remains
   held separately from the current selection and follows existing retirement.
2. High: a detached recovered fork inherited the original receiver identity and
   shared a mutable retirement edge. Agreed; this was a design flaw. Immutable
   `AdmissionCustody` now shares provenance only. Each runtime's concrete
   `CheckpointAdmissionBinding` owns its own `NativeCheckpointDirectoryEdge`.
   Genuine forks clone that edge. The operational court checks actual fork
   export, foreign-basis refusal, associated effect readmission, another Complete,
   and use of retained bases after the replacement.
3. High: cold cell/kind controls and detached symbol-directory backing were not
   funded by the invoking request. Agreed. Actual controls retain exact layout
   reservations; directory maps/arrays use request-backed owners; symbol pages
   share the actual admitted immutable directory/member custody.
4. High: root construction laundered request stops into corrupt checkpoint.
   Agreed. The existing typed root-stop projection now preserves the request,
   allocation and storage outcomes. The four-region memory/cancellation court
   asserts typed stops and complete release after owner drop.
5. Medium: recovered current-head collection used an ordinary uncharged set.
   Agreed. It is now a request-backed `ExecutionOrderedMap`, with charged/polled
   insertions.
6. Medium: current head restore rescanned every canonical index per head.
   Agreed. It now uses the actual admitted history catalog/source placement.

## Latest Native review

Reviewer: `/root/d12_current_custody_review`, independent read-only source
review. Governing input: milestone3 D12/D17, grants11/11a/11b, approved image
design and bounded-history review. Raw evidence: d12-receiver-custody-check.log
and d12-receiver-custody-courts.log. No implementation reasoning or report draft
was supplied. No builds, tests or mutations performed by the reviewer.

Verdict: **APPROVE — targeted Native operational admission/custody cluster.**

Findings: **No material findings.** The reviewer confirms independent receiver
edges, retained provenance across replacement, actual funded root controls, and
both real publication bases and retention leases. Raw evidence shows check0
and2/2 named courts passed. Explicit limitation: excludes full filesystem
qualification and independent execution of source-close/fork-retirement cases.

## Store selection review

Reviewer: `/root/d12_store_selection_review`, independent read-only source
review of the protected Store package/selection cluster and its focused compile
log. Verdict: **APPROVE**, no findings. It checks original protected backing and
section range, cut/authentication joins, strong capture revalidation, predecessor
cutoff, unchanged single selected attempt/ACK and funding. It excludes accepted
outputs and D13.

Fresh full-slice review after qualification and phase gates is still required.

## Additional consumer review during qualification

`/root/old_key_source_boundary_review` found the old-key failure after actual
Query/Native/World receipt reconstruction: Native's no-pending-record settlement
repair demanded a resident positioned original. Agreed. It now uses the actual
bound admitted row/source plus the catalog's existing CheckpointReadmitted
outcome; no performed settlement or positioned original is manufactured. The
review also identified envelope-only cold-index maintenance. Its factual root,
authoring, predecessor and semantic changed-record consumers are cut over.

`/root/image_receipt_index_consumer_review`: targeted **REJECT**, one Medium
finding. Projected validation's case lookup used `.ok()?` inside checked
arithmetic and could erase an invoking stop. Agreed; this is a propagation flaw.
The fallible case/text lookup now uses `?` before checked arithmetic. Scoped
search `rg -n '\.ok\(\)\?' .../decoded_quote/validation` leaves only an
allocation-layout checked-arithmetic conversion, not a request lookup. The
review found no further material source issue in the named receipt/index
consumers. New code/courts still require runtime qualification and fresh review.

The corrected four affected Native image courts now pass4/0/0 in
d12-final-image-consumer-courts.log. This includes genuine cold and incremental
indexes, mismatched before-root refusal, real lookup values and zero canonical
body reads. It does not certify the later Store binding API change.

## Owned Store image review

Fresh `/root/owned_home_image_review`: targeted **APPROVE**, no material findings.
It checks the current strong Complete preparation's privately retained emitted
image, removal of caller-supplied Native DATA at binding, original-request
funding/drop and partial-error behavior, and the same actual selection/ACK owner.
It explicitly excludes behavioral qualification: d12-owned-home-image-check.log
is a compile result only, and earlier behavioral logs predate this API change.

## Single-emission DATA and accounting review

Fresh `/root/single_emission_data_review`: targeted APPROVE, no material
findings. It compared the full emitted grammar, directory and ranges against
the unchanged untrusted reader, and checked authentication, funding and stop
propagation. It explicitly did not certify the8m or<=40 acceptance result.

Fresh `/root/single_emission_accounting_review`: targeted APPROVE, no findings.
The replacement minimum counts actual serializer traversals, size scan, all
meaning/directory hashes, all output copies and fixed header replacement. The
removed charge expectations correspond to operations no longer performed;
authentication tampering assertions and resource ceilings remain unchanged.
Source review only. The corrected accounting case subsequently passed1/0/0 in
d12-single-emission-accounting-corrected.log.

## Complete operational source review (read-only)

Fresh `/root/d12_operational_source_adversarial_review`: REJECT, one P2 finding.
The actual Home path encoded and charged obsolete WSRCSRC1 before replacing it
with WSRCAI01. Agreed: capture/selection custody and final framing were coupled
incorrectly. d12-home-single-framing-plan.md records the typed-phase correction.
The reviewer withdrew a raw-companion API suspicion after tracing the private
strong Home publication constructor; it found no demonstrated authority bypass.
It verified all131 source snapshot hashes and <=400-line scoped Rust files.
No other material source issue was found. Its review does not qualify the red
bounded-tail proof or later source corrections; a fresh review remains required.

Fresh `/root/home_single_framing_source_review`: APPROVE, no material source
findings in the correction. It independently traced the unframed owning phase,
exact original source/member/selection/replacement custody, consuming final
framing, unchanged private Home validation/install and Native-then-World ACK.
Public runtime type placement/facade and all seven callers are appropriate.
Compile check passes; six-case behavior/bounded-tail/gates remain unqualified.

## Retained-index and fixture resource corrections

Fresh d12_current_slice_final_review REJECT: bounded automatic acceptance had
not passed, and explicit reconstruct_for_commit still demanded originals.
Both agreed. bounded_work_boundary_review approves the capacity-derived Home-
only fixture policy design/implementation, with no runtime qualification claim.

Historical image case first exposed absent recovered-retired-head locators;
existing Weak basis registry supplies the actual held basis without fabricating
a retirement or another root table. Historical case then passed.
retained_image_reconstruction_review REJECT2P2: cardinality shrink produced
incomplete seal and len underpaid physical HashMap traversal. Corrected exact
cardinality/paid bucket shape, typed Changed, no owner callback/strong drop under
lock; historical and public sparse-registry courts pass2/0/0.

Fresh retained_snapshot_final_source_review REJECT1High: root() copied unrelated
mutable fork names and visibility name copy was unfunded. Corrected root Arc-
only state access and separate prior payment for each actual name copy. Fresh
source re-review APPROVE, no findings. Final behavioral qualification still due.

## Fresh full landing review and committed-source verification

Fresh d12_landing_independent_review (fork_turns=none), supplied only governing
contracts/design/source snapshot/raw logs: APPROVE, no material in-scope findings.
It independently verifies151 raw hashes/Git blobs, exact selections/counts,
unchanged<=40 filesystem proof, actual roots/pins/leases/source membership,
original-request funding and clean selected-image cutoff. All prior agreed
findings above have corrections and appropriate focused evidence. No rebuttal.

Boundary's3 explicitly deferred BC9002 sites remain reported, not suppressed.
The approval excludes D13, overflow, topology/raw-digest and Phase3 closure.
See d12-admission-landing-independent-review.md for the numbered final findings.

After source commit7b6d17eba74f352fb0f16fa80dabc916a62ff111, the same independent
reviewer verifies all151 committed source blobs with zero mismatches and
explicitly confirms APPROVE applies to that commit. d12-committed-source-
verification.log independently records matching raw working bytes and Git blobs.
Only evidence documentation follows; no code edits after the reviewed snapshot.
