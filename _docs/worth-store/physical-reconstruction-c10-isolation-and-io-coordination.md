# C.10: Stable Reads, Scheduled I/O, And Maintenance Interference

## Goal And Decision

An admitted physical reader keeps reading the exact bytes of its selected
root while foreground mutations, checkpoint, scrub, physical rewrite, and
reclaim run through the same Store instance. Maintenance cannot retire those
bytes, change the reader's basis, bypass scheduler admission, or export its
cost into unbounded memory, retained files, queues, or retries.

The decisive proof is the case matrix below, not the existence of isolation
types or a successful scheduler simulation. Architecture is designed backward
from those cases. Implementation starts with the smallest real stable-reader
journey; full maintenance machinery is not a prerequisite for first feedback.

C.10 is physical isolation, not semantic MVCC. It never selects a logical
snapshot, grants a branch writer, decides record liveness, or interprets Query
visibility. Its maintenance rewrite preserves every selected physical record
identity and payload. Semantic deletion, indexed layouts, native blob policy,
backup workflows, repair authorization, and tier migration remain successors.

Governing sources are the [reconstruction roadmap](physical-foundation-reconstruction-roadmap.md),
[S.5 isolation contracts](storage-foundation-s5.md),
[S.6 I/O contracts](storage-foundation-s6.md),
[C.7 durability](physical-reconstruction-c7-durable-publication-join.md),
[C.8 recovery](physical-reconstruction-c8-fresh-process-recovery-and-reopen.md),
and [C.9 integrity](physical-reconstruction-c9-integrity-and-offline-truth.md).
All eight documents in `../coding_guidelines/` govern this design. This spec
rebinds existing mechanisms to production; historical S.5/S.6 simulations do
not establish that join.

## Current Boundary And Required Cutovers

The current code provides useful foundations, with specific integration gaps:

| Present boundary | C.10 decision |
| --- | --- |
| `ServingPhysicalRuntime::records()` captures a `DurablePhysicalRootManifest`; `RecordReadSession` owns bounded frame/allocation and lifecycle leases. | Preserve bounded streaming. Replace the unprotected root capture with owner-registered root protection acquired before traversal. Lifecycle counters and a cached manifest alone are not retention authority. |
| `PhysicalCurrentRootOwner` owns current/previous roots and namespace-durable advancement. | Keep it as the single root-publication authority. Add the atomic root-capture/retirement interlock here; do not create an isolation-owned current-root register. |
| `RetainedPhysicalRoot::supporting_artifacts()` lists immediate support. | Do not mistake this list for a transitive reachability proof. Reader and recovery protection must cover referenced descendants and physical generations. |
| `worth-store-physical-isolation` has epochs, hazards, latches, plans, reclaim laws, and adapters importing `worth-store`. | Establish a downward mechanism boundary before Store imports it. Move Store-chunk adapters and their byte-execution/security join into Store; move reconstruction-specific adapters to Recovery. Live tables are instantiated and owned by the one Store instance. |
| `PhysicalWorkConcurrencyScope::relation` compares ranges; several non-range work scopes project to empty coordinates. | Replace the incomplete projection with exhaustive effect-footprint lowering. Root, allocator, WAL, namespace, whole-artifact, and retirement scopes must never become disjoint merely because no byte ranges were returned. |
| `PhysicalSchedulerAdmissionOwner` and `PhysicalInstanceForegroundCapacity` already perform live reservation; scrub reserves background capacity leaving half available to foreground. | Extend that owner with admitted class policy and bounded ready-queue selection. Do not add a maintenance scheduler or a second capacity ledger. Preserve the current foreground reservation as the default floor. |
| C.9 scrub already has bounded managed work, and C.8 has `RecoveredPhysicalRuntimeHandoff`. | Schedule scrub through its existing lifecycle; initialize fresh stability state from the recovery handoff, never from serialized leases or reports. |

The ordinary call chain remains:

```text
Store facade and concrete platform admission
  -> Store stable-root / exact-effect admission
  -> physical Signal dependency readiness
  -> existing I/O scheduler resource admission and selection
  -> Store executor -> qualified filesystem media
  -> exact backend completion -> owning physical settlement
```

Pure format/integrity interpretation and warm resident reads stay direct when
no I/O or async boundary is crossed. They do not acquire ceremonial Signal
jobs. A cold read, sync, retirement effect, or maintenance write cannot use
that exception.

## Non-Fake Acceptance Setup

### Production subject and roles

Ordinary subjects are `ServingPhysicalRuntime::records()`, `record_submission()`,
`checkpoints()`, the existing managed scrub facade, and the new Store-owned
stable-read and maintenance facades defined below. The real participants are
`worth-store`, `worth-store-physical-isolation`, `worth-store-io-scheduler`,
`worth-store-buffer-pool`, `worth-store-wal`, `worth-store-physical-format`,
`worth-store-physical-integrity`, and `worth-store-physical-backend`.

A responsibility-named `physical_store_interference` executable in the existing
physical-certification crate drives producer and serving-child roles through
those facades. It owns only workload generation and process coordination, not
an alternative Store. A separate `physical_store_recover` process uses C.8;
`physical_store_integrity_observer` remains the separate read-only observer.
Build each executable once per feature/profile lane and discover its path from
Cargo, rather than nesting Cargo inside individual tests.

The independent input model maps producer-returned stable record identities to
deterministically generated bytes. It also records which operations the parent
actually observed completing. For controlled reader cases it fixes the expected
record membership before and after each gated publication; identical payloads
and reported root tags alone cannot prove root isolation. It does not compute
expected truth through Store's placement, retirement, or recovery classifier.
The verifier receives only the inert root and its declared protocol/configuration
inputs; the parent compares its output with expectations afterward.

Permitted test layers are production-boundary media observation/interposition,
bounded pause gates at named transitions, a parent that kills a child without
cleanup, and independent read-only OS inspection after ownership is released.
Private table edits, counterfeit leases, fake scheduler completions, manually
constructed persisted roots, and live/offline concurrent mutation are forbidden.

### World and scale axes

The canonical world uses C.5-C.9 admitted formats on a real buffered filesystem:

- 16 KiB pages and 64 KiB resident-frame budget; metadata, per-operation scratch,
  pins, dirty frames, leases, and queued commands have separate hard caps.
- At least 2 MiB of actual reachable payload (32 times resident bytes), at least
  eight populated segments, inline and multi-chunk extent records, a multilevel
  routing tree, two WAL rotations, and a namespace-durable checkpoint.
- Include partially occupied segment tails and a subsequent append to those
  tails; do not select a fixture shape that avoids the C.9 supersession case.
- Three simultaneous read roles: a paused old-root reader, a current reader,
  and a slow extent/scan session whose next chunk is not resident. At least
  four successive publications occur before the oldest reader finishes.
- Two independent mutation capabilities, one overlapping root publication,
  disjoint physical data work, and checkpoint/scrub/rewrite/reclaim pressure.
- A canonical scheduler profile of four dispatch permits, two protected for
  foreground, at most two background dispatches in flight, and eight bounded
  ready slots. A background quantum is at most one admitted frame/window or
  one indivisible sync/delete action. Actual backend limits may require a
  stricter admitted profile, never a silently weakened guarantee.
- The canonical policy admits 64 protection slots, 16 retained root descriptors,
  8 MiB of excess obsolete/staged artifact bytes, 1 MiB of simultaneous candidate
  bytes, four managed maintenance handles, and eight deferred cleanup entries.
  Each rewrite selects at most 256 KiB of source payload; routing/encoding
  overhead is charged separately within the admitted candidate/scratch limits.
  The progress headroom defined below is withheld inside these limits, not added
  above them; report both hard ceilings and the remaining growth allowance.
  Run the limit and one-over-limit twins independently; a rejected world is not
  evidence for the valid case. These fixture values are not hard-coded product
  maxima or a claim that the whole process fits the resident-frame budget.

The producer must grow the world until the actual payload/topology predicates
hold. Reuse C.9's production-building mechanisms where lawful, not its runtime
heap or copied post-run root. Every crash seam gets a freshly produced root.
Keep payload generation, setup, measured action, and verification costs separate.

Focused cases use smaller worlds with the same relevant invariants. Independent
scale sweeps vary persisted size at 8/32/128 resident budgets; live leases at
1/8/64; retained root generations at 1/4/16; ready work at 1/4/8; and rewrite
scope at 1/4/16 selected artifacts. Use corresponding admitted budgets and
record actual topology. These are separate axes, not one exponential Cartesian
campaign. Dedicated 32/64 KiB page variants and Linux/Windows runs establish
only their exercised platform/format claims.

### Decisive interleaving

1. Produce and checkpoint a baseline; reopen in the serving child.
2. Begin a stable read, pause after root protection but before a cold descendant
   load, and separately hold a borrowed chunk. Record the selected root.
3. Publish foreground appends and a record-preserving rewrite of a selected
   segment/extent through real WAL, data, and root effects. Advance the root
   more than once. A new reader sees the newer root; the old reader does not.
4. Run checkpoint and one bounded scrub window while attempting to retire the
   rewritten old artifacts. Saturate background ready slots and delay a real
   background effect. Admit disjoint foreground reads/writes concurrently.
5. Resume the old cold reader and verify all chunks against the original input
   model. Release one of two protectors: reclaim remains blocked. Release the
   last relevant protector, advance the recovery retention basis as required,
   then execute bounded eligible retirement.
6. Repeat the targeted publication/retirement crash seams below. Recover in a
   fresh process and inspect offline after recovery closes. No process receives
   writer leases, Signal state, cached roots, or expected recovered records.

Gates choose ordering; sleeps do not establish correctness. A timeout is a
failure to finish the case, not proof that reclaim was blocked. Observe the
actual typed blocker and exact lack of destructive backend calls.

## Required Test-Case Matrix

Rows are obligations for coherent test families, not a prescribed test count
or a generated evidence registry. A family may cover several rows if each
relevant failure remains distinguishable. Every hostile case first establishes
the production-valid setup needed to reach its disputed boundary.

| Case family | Sequence / boundary | Required observation | Plausible defect it must expose |
| --- | --- | --- | --- |
| Stable-root control | Pin R, append a new record in R+1, then point-read and finish a paused scan through old and new handles. | Old reader returns `RecordNotFound` for the new identity and exactly R's scan membership; new reader includes it. Existing bytes match the input model. | Reader silently refreshes its root while unchanged payloads and descriptive root tags conceal it. |
| Capture/publication race | Pause immediately before protection, then immediately after it; advance current root in each schedule. | Capture linearizes wholly before or after advancement; never an unprotected old root. | Snapshot first, register protection later. |
| Cold descendant after many swaps | Hold R across four swaps, evict descendants, resume an extent and a routing-tree scan. | Old-generation files really refault and return exact bytes; current readers still progress. | Protect only resident frames or only the immediately previous root. |
| Mixed generation / reuse | Reuse a numeric coordinate under a different artifact generation; present the old locator. | Exact generation denial or explicitly reacquired new plan; no silent redirection. | Filename/range equality used as identity; epoch wrap or ABA ignored. |
| Binding drift | Change Store, incarnation, root, security scope, capability generation, and policy generation one at a time. | Intended typed stale/denied/rebind result before unauthorized effect. Valid counterpart executes. | One binding axis omitted; checksum or report promoted to authority. |
| Stepwise traversal | Pause after parent admission before acquiring child protection, including an extent chunk boundary. | Parent/root protection bridges traversal; child frame validates before exposure; bounded scratch. | Broad preloading, unprotected look-ahead, or mixed-root scan. |
| Multiple protectors | Exhaust protection slots; drop a reader while its session remains, then release the session. Separately release two readers of one retired generation in order. | `ProtectionLimit` persists until the final descendant releases its slot, then acquisition succeeds; owner lookup tracks the exact reader-protected root. One of two readers cannot enable deletion. | No-op registration, premature slot reuse, boolean reader-live flag or double release. |
| Expiry and abandonment | Deadline, dropped handle, explicit cancellation, and borrowed chunk during close. | No newly issued view after revocation; already borrowed memory remains valid; releases happen once. | Timer forcibly frees borrowed bytes; drop skips lifecycle settlement. |
| Retention pressure | Publish to the growth allowance while an old root is pinned; reject one more request, release blockers, then checkpoint/reclaim and retry. | Reader stays safe and denial precedes effects; reserved progress headroom permits real cleanup and a successful retry under unchanged hard budgets. | Unbounded retained files, eviction of protected bytes, or a bounded but permanently wedged Store. |
| Exact conflict algebra | Range read/write, whole-file delete, truncate, WAL append/barrier, allocator, root and namespace scopes. | Read/read sharing and truly disjoint effects proceed; each actual overlap is serialized at its own key. | Empty coordinates imply disjointness; unrelated writes use a Store-wide effect lock. |
| Latch order and handoff | Request reversed multi-key acquisition; exhaust resources between readiness and dispatch. | Reject/retry before waiting in an inverted order; zero structural latches held across blocking I/O or readiness waits. | Acquire a second latch opportunistically or hold root lock while awaiting capacity. |
| Fair dispatch | Fill all four permits with foreground I/O, continuously refill its queue, and ready one bounded background head; also test already-dispatchable queues. | Background earns its turn within three foreground dispatches; once owed, refills cannot steal returning capacity. It dispatches by release of the existing blocking leases, at most four completions in the canonical case. Foreground floor survives. | Fairness tested only after manufacturing eligibility; permanent starvation under foreground saturation. |
| Shared-device interference | Hold a real background sync while foreground reads/WAL work queue. | Account actual in-flight non-preemptible work and wait; preserve capacity; report unsupported/stalled QoS honestly. | Reserved queue slots presented as a hard device-latency guarantee. |
| Saturation and cancellation | Fill ready slots, bytes, worker, dirty, pin, and maintenance budgets; cancel before/after dispatch. | Exact exhausted dimension; no effect before admission; post-dispatch fate settles once; all charges released or retained by a named obligation. | A second hidden queue, leaked permit, or cancellation treated as rollback. |
| Class and security preservation | Group, retry, yield and resume checkpoint/scrub/rewrite/reclaim beside foreground work. | Operation, class, security, durability, and allocation lane survive every transfer. | Background relabeling as foreground; grouping across security or durability boundaries. |
| Rewrite control and competing writer | Pause append A after its WAL barrier but before root publication; attempt rewrite B, finish A, reject B's stale plan, then freshly plan B. Race a later append against B's reservation too. | Pending A blocks B without B WAL/data effects or an exclusive claim. After A publishes, B consumes the new source; later root-changing work stays pre-effect until B settles. Identities/payloads and both successful publications survive recovery. | Current-root equality ignores an already-durable append; post-effect work is mislabeled no-effect or replay resurrects its older placement. |
| Integrity during maintenance | Damage a declared source frame before rewrite; corrupt a candidate before publication through the interposer. | C.9 localization before use; no damaged source treated as deletable/rebuildable; no unchecked candidate published. | Maintenance bypasses integrity or treats scrub damage as repair permission. |
| Reclaim/read race | Pause after eligibility analysis, then admit a competing reader or advance a root. | Retirement claim and protection are atomically exclusive; stale eligibility cannot authorize media deletion. | Cloneable snapshot of hazards used as a live deletion permit. |
| Recovery retention | Drop all live readers but retain old checkpoint/WAL/previous-selector dependencies. | Deletion remains blocked by the specific durable dependency; C.7 idempotency evidence survives. | No live reader mistaken for no recovery obligation. |
| Transitive/shared reachability | Two protected roots share a routing block, segment or extent; retire one root. | Shared descendant survives; unrelated eligible artifact can retire without full-store scanning. | Immediate manifest list or root age used as transitive ownership. |
| Delete/namespace uncertainty | Fail before delete, after delete, or during directory sync; retry after reopen. | No-effect, partial/indeterminate, and completed states stay distinct; reuse waits for settled durable retirement. | Missing file treated as success for the wrong identity or rollback reported after unlink. |
| Crash/recovery matrix | Kill at each named seam below; fresh recover and offline inspect. | Exact allowed root/fate, record parity, bounded cleanup, and idempotent second reopen. | In-memory lease/publication state needed for recovery; unsupported bytes guessed current. |
| Protocol coexistence and downgrade | Open a C.9 Store, perform its first rewrite/retirement, checkpoint away old WAL, then reopen with supported and unsupported protocol consumers. | Legacy input remains readable by new software; required maintenance capability survives compaction; incompatible open/recovery refuses before effects. | New payload treated as append, silently skipped, or its version requirement lost when WAL is reclaimed. |
| Unrelated-scope locality | Grow unrelated segments, roots and queues independently. | Point-read/retire-selection counters depend on touched paths and actual overlaps, not all stored objects/readers. | Global hazard scan, full-store reachability set, or broad sync hidden behind one call. |
| Successor admission | Exercise class lowering and denied uninstalled producer capabilities. | Backup/blob/repair labels alone create no work; added producer consumes the same protected-read/effect contracts. | Future-looking enum variant becomes a raw generic maintenance escape hatch. |
| Public and dependency boundary | Consolidated compile-pass/fail consumers plus dependency/source guards. | Cannot construct live plans, substitute copied facts, outlive chunk/session, access raw media, or import reconstruction through ordinary isolation. | Public guard constructor, generic authority marker, upward crate cycle, or alternate executor. |

### Crash-seam matrix

The complete physical rewrite is redo-described before candidate data dispatch.
Root visibility and physical acknowledgment retain C.7's ordering. The parent
kills the serving process at each of these explicit boundaries. Repeat the
competing-writer case with a kill while B is blocked behind durable A: healthy
C.8 recovery resolves A under its existing redo contract and finds no B effects.
Then complete A, freshly admit B, and exercise B's WAL/data/root seams; A's
published membership must survive every recovery and the second reopen.

| Kill point | Recovery obligation |
| --- | --- |
| Before rewrite WAL dispatch; during a strict WAL prefix | The settled source remains authoritative, with earlier unfinished appends governed by C.7/C.8. An incomplete rewrite group cannot supply redo or successful fate. Absence alone is not positive no-effect evidence. |
| After complete WAL barrier, before candidate data | The valid control recovers the sealed rewrite from complete admitted redo; an injected integrity/capability failure must produce its exact typed denial. Never lose acknowledged foreground data. |
| During candidate data; after candidate sync, before root replacement | No partial root is serving truth. Replay validates the complete source/target generation relation and exact group membership. |
| After root replacement, before namespace sync | Preserve C.7/C.8 ambiguity handling; choose only a root admitted from actual persisted protocol state. No speculative reclaim. |
| After namespace durability, before caller observes completion | Recover the published physical state and unobserved completed fate; do not duplicate the rewrite on retry. |
| After durable retirement intent, before delete | Reconstruct remaining physical cleanup from durable authority, without writer hazard state. |
| After delete, before namespace sync; after retirement completion | Reconcile exact artifact absence and namespace durability; no generation reuse based on an incomplete cleanup observation. |

For pre-ack seams the expected allowed set is fixed by persisted C.7/C.8
evidence, not by assuming that a process kill models a power loss. A normal
kill may leave unsynchronized bytes visible. Injected media faults and named
backend qualification establish only their corresponding stronger claims.

## Architecture And Authority Lock

### Stable-root protection

Keep one `PhysicalCurrentRootOwner`. Root capture and retirement registration
share its linearization boundary: acquire a bounded protection slot and sample
the current root under the same root/retention synchronization. Release the
structural lock before I/O. If protection capacity cannot be reserved, return a
typed denial before returning a root-bound reader.

Each reader acquisition reserves one protection slot, including another reader
of the same root. Descendant sessions share that registration until its final
owner releases it; unique root descriptors are accounted separately. The live
owner indexes the registration by its issued root, not merely by a reader count.

A `PhysicalRecordReader` has a protected root, not a periodically refreshed
catalog. Its `RecordReadSession` and scan session retain the same root protection
even if the outer reader is dropped. Each chunk additionally carries its real
C.6 frame lifetime and C.9 exact-source admission. A root lease protects cold
reachable artifacts; a frame pin protects resident memory. Neither substitutes
for the other. Advancing a session cannot release the root between chunks.

Separate immutable root identity from live protection. A copied
`DurablePhysicalRootManifest`, `RetainedPhysicalRoot`, epoch vector, descriptor,
hash, report, or old `StablePhysicalReadPlan` cannot acquire the new serving
capability. Freshness is sampled by the owning instance, not passed in by a
caller. Old-but-protected is a lawful state, not a stale-current error.

Unknown traversal is admitted stepwise under the retained root. Resolve and
validate each bounded child before use; do not materialize the complete graph
to predeclare a read plan. External serialized scan cursors carry descriptions
only and require fresh admission; cross-process durable read leases are not a
C.10 promise.

Use the existing proof substrate for legal progression and exact binding axes.
Concrete platform grants gate Store entry. Sealed Store types retain the actual
live lease, instance reference, and effect scope; `worth-proof` must not contain
live tables, clocks, counters, or media. Portable descriptive identity remains
in format/Foundational as appropriate. Public generic marker bounds or a
caller-supplied borrowed byte slice must not mint protected bytes.

### Conflict and latch policy

Lower each work intent once into a bounded canonical `PhysicalEffectFootprint`:
Store/incarnation, admitted security scope, artifact identity and generation,
byte interval where meaningful, access mode, and named coordination keys.
Use closed variants for read/write range, whole artifact, truncate/delete,
WAL-range/barrier, allocator partition, root publication and namespace entry.
Containment matters: deleting a file conflicts with any range of that exact
file. Different security scopes are not a disjointness proof for shared files.

An exhaustive intersection function handles every pair. New effect variants
must break lowering/conflict tests until explicitly classified; an empty range
projection is never a default-disjoint result. Admission belongs to Store and
its owning mechanisms. The scheduler consumes the conflict admission; it does
not infer concurrency from a digest or branch label.

Use ordered acquisition with fail/retry before an inverted wait, not a second
deadlock graph. Order multi-key acquisitions by Store instance, coordination
class, artifact identity/generation, then range start; the concrete class order
is root/retention, allocator, artifact metadata, frame. Avoid holding multiple
classes where phase separation suffices. No structural latch survives a media
call, Signal wait, scheduler wait, callback, or user yieldpoint. A long-lived
logical publication/retirement reservation may cross I/O but blocks only its
declared conflicting scope; it is not a mutex held across the call.

### Rewrite and publication

C.10 installs bounded, record-preserving copy-on-write of selected current
record artifacts: inline pages within a selected segment and extent-backed
records. It changes physical placement/generation, not record identity,
payload, semantic liveness, or tenant/key meaning. Preserve unrelated root
entries. C.11 adds indexed/blob producer adapters beneath these boundaries.

The rewrite producer resolves exact sources, validates them through C.9,
reserves peak scratch and worst-case retained/candidate storage, and lowers a
complete bounded candidate plan before effects. A plan too large for one
admitted window is rejected or partitioned into separately atomic operations
before starting; an atomic operation is never silently split after admission.

Join the existing mutation identity, WAL, data, checkpoint, current-root and
namespace owners. Add a distinct physical rewrite member to their shared
publication progression; do not fabricate an append of duplicate records or
create a maintenance root publisher. Every root-changing producer registers its
pending publication obligation with the current-root owner before its first WAL
effect, including foreground appends. This is a bounded index of existing
operation identities, not a second operation/fate registry. Registration and
rewrite exclusion share one atomic admission boundary.

Before rewrite WAL/data effects, the root owner must establish both exact source
equality and absence of unsettled conflicting publication obligations. An earlier
WAL-durable append produces `ScopeConflict` with its exact pending blocker;
the rewrite takes no exclusive reservation and cannot obstruct that append's
settlement. A published predecessor invalidates the old rewrite plan with
`SourceChanged`; retry requires fresh bounded planning. A post-effect cancelled,
partial or indeterminate predecessor is not a completed/no-effect obligation:
it blocks rewrite until settled or resolved through recovery. Reader counts and
current-root equality alone cannot discharge it.

On success, acquire the root owner's logical publication reservation atomically
and keep it through namespace durability and exact settlement. While held, later
conflicting producers cannot cross their first WAL/data effect; queued source
bindings must be revalidated when admission resumes. Revalidate the rewrite at
advancement too. Earlier durable work therefore settles before rewrite sealing,
and later work cannot seal against a source that the rewrite will displace.

That reservation is shared-root coordination, not a structural latch or a
whole-Store submission borrow. Disjoint reads, bounded pre-effect preparation
and admitted effects without that publication dependency continue. Root-changing
foreground WAL/data work waits at admission, not after durable side effects.
Bound the rewrite window first; count publication-conflict wait and barrier cost.
No structural latch or dispatch permit is held while waiting for a predecessor.

WAL v1 framing remains unchanged. Add a separately versioned
`store.physical.rewrite-redo.v1` payload, not a reinterpretation of
`store.physical.wal.canonical-redo.v3`. Its canonical declaration fixes operation
and group identity, source-root basis, bounded source artifact identities and
integrity bindings, exact destination generations/ranges and pageLSNs,
record-identity-preserving placement deltas, candidate bytes needed for redo,
and the resulting root transition. It carries no semantic delete or branch
meaning. Bounds are checked before retaining payloads; one complete admitted
group and its WAL barrier precede destination writes. Target pageLSNs reflect
the rewrite's physical WAL coverage, not an invented logical update.

Add this payload to C.8's typed redo dispatch and C.9's independent WAL
observation. Existing append payloads retain their old interpretation. New
software reads both; old software must reject the unsupported payload before
recovery effects, never skip it as irrelevant. C.10 does not promise writable
downgrade after the first rewrite. Golden vectors cover both payload kinds,
malformed lengths, unknown versions and mixed append/rewrite tails. Required
maintenance protocol capability is also retained in versioned root/checkpoint
metadata, so WAL compaction cannot accidentally permit an unsafe downgrade.
An incompatible reader must reject that metadata before serving or mutation;
do not append new fields while retaining an old envelope's interpretation.
New software admits the C.9 legacy profile and the maintenance-capable profile;
the first maintenance publication makes the latter requirement durable through
the existing publication protocol. Format mechanics remain canonical; recovery
alone selects and applies the result.

### Retirement and physical retention

Publication produces exact retirement candidates, not deletion permission.
The retirement owner intersects them with all live root/range protections,
current reachability, previous-selector obligations, checkpoint/recovery
sources, unresolved physical operations, and retained idempotency bindings.
Shared descendants remain protected while any such obligation exists.

Use indexed root/epoch and artifact-generation obligations with bounded
publication deltas. Root capture must not walk the graph. Discovery or rebuild
of retirement information is an explicitly bounded maintenance/recovery task;
missing or exhausted information blocks deletion. Do not retain a full-store
reachability set in ordinary memory or scan every reader on every point read.

Eligibility analysis emits a candidate, not a reusable deletion permit. Under
the owning protection/retirement interlock, revalidate and consume it into a
linear `PhysicalRetirementClaim` bound to exact artifact generations. New
protection and retirement cannot both win. Keep that claim charged across
scheduler waiting and media completion. No caller-built snapshots, copied
`ReclaimEligibilityProof`, expiry, age, absence from the current root, or scrub
report may bypass this transition.

C.10 physically deletes whole obsolete immutable artifact generations. It does
not punch holes, trim live files, reuse offsets in place, or collect records on
semantic liveness grounds. C.11/S.10 may add appropriately admitted adapters.
Reclaim only artifacts proven obsolete through the physical publication chain
or attributable exclusively to a failed physical operation. Unknown residue
and damaged authority remain quarantined/indeterminate, never cleanup guesses.

Record retirement intent and completion as
`store.physical.retirement.v1` maintenance payloads inside the existing
WAL/checkpoint retention protocol, with exact
operation, source publication, artifact generation, action and outcome. No
sidecar journal or filename scanner becomes authority. Persist an intent before
deletion; complete the backend delete and required namespace synchronization
before durable completion/reuse eligibility. C.8 reconstructs only these
physical obligations. A retry is idempotent for the exact identity; an already
missing file needs admitted intent plus verified parent namespace posture,
not merely `NotFound`. Candidate and retirement records cannot be reclaimed
from WAL until checkpoint compaction preserves every unresolved obligation.

Retained bytes, active roots, protected slots, staged candidates and cleanup
debt have separate limits. Charge unique physical generations, not every
reference to shared bytes; include uncollected garbage and in-flight worst-case
growth. Accounting and pre-effect growth admission ship with the first rewrite,
not with the later delete executor. Exhaustion backpressures new growth before
WAL/data effects; never invalidate a live reader to force progress.

Each admitted profile withholds enough byte, obligation-entry, queue-storage and
scratch headroom for one bounded checkpoint/retirement progress cycle, derived
from canonical encoding and operation bounds. Deduct it from existing hard
budgets in their existing owners; no overflow allowance or second ledger.
New growth cannot consume this headroom, and only the owning checkpoint/cleanup
workflow may use it. Reject a profile that cannot support this cycle. Dispatch
permits remain per-quantum, not held idle for future cleanup. Once external
blockers are released and the backend progresses, cleanup must restore growth
admission without increasing limits. Bounded release/wakeup belongs to the
existing lifecycle; a non-releasing reader may still block growth indefinitely.

### Scheduler service and interference

Extend `PhysicalSchedulerAdmissionOwner` and the existing scheduler's live
capacity/queue machinery. The sole Store operation registry retains lifecycle
and fate; ready queues contain bounded references to those operations, not
second operation objects. Signal remains the readiness owner. Pure scheduling
selection performs no media effects, no topology discovery and no settlement.

Configuration admits resource ceilings, per-class reservation, maximum quantum,
FIFO order within each class, foreground/background weights, and cancellation
and deadline policy together. The ordinary default preserves at least half of
each scarce dispatch resource for foreground. Do not hold all permits for an
entire multi-window maintenance job; charge each dispatched quantum until its
actual completion, while separately charging the managed job and retained data.

Queued work charges bounded queue/semantic storage, not dispatch permits.
Selection and live dispatch reservation occur together after dependency
readiness; a job waiting for a dependency or a protection blocker cannot hoard
the worker/byte permits needed by its prerequisite. Permanently oversized
quanta are denied rather than left forever at a queue head.

Distinguish **ready** (dependencies, authority and physical-conflict admission
hold) from **dispatchable** (live capacity also fits). Canonical class selection
is weighted round-robin 3:1, FIFO within each class. A continuously ready
background head earns a turn after at most three foreground dispatches;
temporary capacity shortage cannot erase that turn or restart its weight.

For an owed background turn, the same scheduler withholds new foreground refill
dispatches until returned capacity can admit that bounded quantum while leaving
the foreground floor intact. Already-dispatched work continues. This is a
selection obligation, not a fake active permit or another queue. Atomically
reserve and dispatch when capacity fits, then resume weighted service. A
cancelled head or lost non-capacity prerequisite releases the selection hold;
it must not block its own prerequisite. Oversized work is denied as above.

With four occupied canonical permits and a one-permit background head, once its
turn is owed it dispatches by the return of the existing blocking leases, at
most four actual completions; continuous foreground arrivals cannot extend that
bound. Also prove the four-selection bound when both classes already fit. Other
resource dimensions obey the same holdback rule. Genuine external protection or
backend stalls suspend the bound with a typed blocker, but scheduler-caused
capacity starvation is not an exemption. A deadline may cancel pre-effect work;
it cannot undo an in-flight filesystem call.

Reservations are not device preemption. At most the admitted background
in-flight quanta may precede a newly ready foreground request at a shared
physical device. Count those quanta and their actual service/sync delay. A
backend without a qualified service-time bound provides structural fairness
and observed latency, not guaranteed milliseconds. Reject a requested hard
latency promise as `UnsupportedQos` when it cannot be supported. A stalled
device yields typed deadline/stall/indeterminate fate, never a fabricated
latency success or weakened durability.

Checkpoint, scrub, record rewrite and reclaim receive concrete producer
adapters now. Backup-read, verifier-read, blob and repair classes retain typed
declarations as successor insertion points but no executable generic entry:
they require their actual installed owner and authority. The independent
offline observer does not join the live scheduler or take live mutation
ownership. Repair and semantic retention remain uninstalled capabilities.

### Lifecycle and outcome topology

Use existing Store cancellation, deadlines, close/drain and exact-effect fate.
New handles are bounded, incarnation-fenced, non-forgeable and framework-owned.
Dropping a mutation/maintenance observer abandons observation only; dispatched
effects still settle. Dropping a reader releases its own protection, not that
of sessions or chunks still alive. Stale completions release their exact old
charges without touching a new operation or replacement incarnation.

Public outcomes distinguish `StalePlan`, `RebindRequired`, `ProtectionLimit`,
`RetentionPressure`, `ScopeConflict`, `SourceChanged`, `UnsupportedQos`,
`IntegrityDenied`, pre-effect cancellation/deadline, completed effect, partial
effect with cleanup pending, and indeterminate effect requiring recovery.
Reuse existing precise outcomes where equivalent; do not flatten them to
strings or booleans. A successful rewrite with deferred retirement is not a
failed publication, and partial cleanup is not rollback.

Close stops new admission, revokes future read access, cancels safe queued
work, settles or classifies dispatched work, and waits for actual borrowed
resources before releasing their backing storage. Bounded close may report
retained protection or inspection-required posture; it cannot claim drained.
Recovery discards dead-process leases and scheduling state, reconstructs durable
retention/cleanup obligations from admitted media, and issues new handles.

## Public DX Target

Retain `records().open(...)`, bounded chunk streaming, record submission,
checkpoints and scrub as the ordinary roots. The change to root-protection
admission is explicit and fallible; do not hide allocation denial in a panic.
`ServingPhysicalRuntime::records()` becomes a fallible protected-reader
acquisition, with all existing callers migrated in the same cutover.

The following is the normative caller shape, using proposed names for new
maintenance types. Every shown call must become a compile-tested public example;
it is a target, not a claim that these methods exist today.

```rust
// Store was opened/admitted through the ordinary serving or C.8 recovery path.
let old_reader = store.records()?; // atomically selects and protects one root
let mut old_read = old_reader.open(record_id, read_limits)?;
let protected_root = old_reader.root_observation(); // descriptive, not authority

let writes = store.record_submission(); // independent of the retained reader
let maintenance = store.maintenance();
let rewrite = maintenance.plan_rewrite(
    PhysicalRewriteRequest::preserve_records(selection, work_limits),
)?;
let mut work = maintenance.submit_rewrite(rewrite)?;

// One bounded unit. Existing Store execution/lifecycle drives effects.
match work.advance()? {
    PhysicalMaintenanceProgress::Pending(progress) => observe(progress),
    PhysicalMaintenanceProgress::Completed(result) => observe(result),
    PhysicalMaintenanceProgress::RecoveryRequired(recovery) => inspect(recovery),
}

while let Some(chunk) = old_read.next_chunk()? {
    consume(chunk.bytes()); // borrow cannot escape the session
}
drop(old_read);
drop(old_reader);

// Caller requests bounded reevaluation, never supplies deletion eligibility.
let reclaim = maintenance.reclaim_retired(reclaim_limits)?;
observe(reclaim);
```

The selection is a bounded physical request; it cannot supply current-root,
security, source-integrity, or reclaim authority. Policy/deadline/cancellation
values enter typed requests and are frozen in admitted plans. Observation APIs
expose selected root, protection, outstanding progress, pressure, actual cost
and recovery posture without exposing internal tables, executor or raw files.

## Required Destination Topology

Legend: **E** existing owner extended; **N** new populated responsibility;
**M** move/cut over existing responsibility; **R** remove or narrow obsolete
surface; **F** committed future insertion, no empty file now. Paths below are
relative to `workspaces/worth-store/crates/` unless stated otherwise.

```text
worth-store/src/physical_runtime/
  mod.rs                                      E facade exports only
  instance/
    parts.rs                                  E exhaustive lifecycle composition
    scheduler_admission/                      E producer -> one scheduler
      scrub.rs, checkpoint.rs, reclamation.rs E current adapters cut over
      rewrite.rs                              N exact physical rewrite demand
    executor/                                 E sole media-effect boundary
  stability/                                  N Store-owned live physical protection
    mod.rs                                    N narrow capability facade
    admission.rs                              N instance/policy/protection admission
    read_plan/                                N immutable protected-root observation
      mod.rs, binding.rs
    byte_guard/                               M Store chunk + integrity + lease join
      mod.rs, scope.rs, execution.rs
    retention/                                N indexed live protection lifecycle
      mod.rs, registrations.rs, lease.rs, observation.rs
      obligations.rs, pressure.rs             F durable obligations and growth budgets (Phases 3/4)
    lifecycle.rs                              N revoke, release, close propagation
  maintenance/                                N orchestration, not another runtime
    mod.rs                                    N caller facade
    rewrite/                                  N record-preserving producer
      mod.rs, planning.rs, progression.rs
    retirement/                               N physical deletion lifecycle
      mod.rs, eligibility.rs, claim.rs, execution.rs
    observation.rs                            N progress, debt, effect fate
    backup/, repair/, tier_movement/           F separately authorized workflows
  work/
    concurrency_scope.rs                      R incomplete coordinate-only relation
    effect_footprint/                         N exhaustive scope algebra
      mod.rs, lowering.rs, intersection.rs
    submission/, execution/, cancellation/     E same identity and settlement
  durability/
    publication/                              E sole root advance / retention join
      current_root_owner.rs                   E atomic capture/registration and publication
    wal/                                      E rewrite and retirement payload producers
    checkpoint/                               E bounded outstanding-obligation compaction
  record_serving/
    lifecycle/serving_runtime.rs              E fallible protected records() cutover
    access/                                   E bounded traversal and read/scan sessions carry protection
    publication/                              E append and rewrite share root owner
  integrity/                                  E exact resident/source binding stays here

worth-store-physical-isolation/src/
  lib.rs                                      E mechanism facade, no Store import
  epoch/, hazard_lease/, latch/                E mechanisms; Store owns live instances
  physical_read_plan/                          E pure/local protection law, no raw-root admission
  reclaim_reachability/                        E eligibility analysis, not media permit
  byte_guard/, stable_read_execution/          M Store-dependent byte execution moves upward
  security_scope_propagation/                  M byte/Store joins move; pure scope law stays
  generation/reference.rs                    M chunk adapter upward; generation law stays
  readiness/                                  R report-hash-derived entry authority removed
  physical_semantic_boundary/root_epoch_basis.rs M current-root correlation only; no recovery proxy
  publication/                                E intent, ordering and local completion only
    completion.rs                             N local plan result; no executed-evidence export
    receipt.rs, foundational_evidence.rs       R synthetic publication execution removed
    crash_matrix.rs, recovery_replay.rs        R synthetic recovery self-test path removed
  compaction_interlock/recovery_evidence.rs    R unrelated checkpoint-product gate removed
  compaction_interlock/read_plan_completion.rs N exact local pre/post plan correlation only

worth-store-io-scheduler/src/
  foreground_reservation/physical_instance/    E one live capacity owner
  background_pacing/                          E bounded per-quantum reservation
  queue_execution/
    policy/                                   E class, fairness and backend QoS admission
    dispatch/                                 N bounded eligible selection / wakeup
      mod.rs, selection.rs, cancellation.rs
    observation/                              E actual queue/service/interference evidence

worth-store-physical-format/src/
  maintenance_record/                         N canonical bounded versioned payload declarations
    mod.rs, rewrite.rs, retirement.rs
  wal_frame/, checkpoint/, manifest/          E versioned capability and obligation support
worth-store-recovery-physics/src/
  redo_replay/                                E typed append/rewrite selection and idempotence
  maintenance_recovery/                       N pure retirement reconciliation law
    mod.rs, rewrite.rs, retirement.rs
worth-store-recovery-runtime/src/
  orchestration/                              E real performed effects and fresh handoff
  progression/discovered/selection.rs         E recovered checkpoint-product selection stays here
worth-store-offline-integrity-observer/src/
  integrity_observation/families/             E independent maintenance payload interpretation

worth-store/tests/
  physical_record_journeys.rs                 E existing integration target
  physical_record_journeys/
    stable_reads/                             N root/lifetime/binding scenarios
    maintenance_interference/                  N real rewrite/reclaim/scheduler scenarios
  physical_runtime_authority_ui.rs            E consolidated external compile boundaries
worth-store-physical-certification/src/
  bin/physical_store_interference.rs           N process entry only
  maintenance_interference/                    N process and larger-than-memory scenarios
    mod.rs, world.rs, schedule.rs, observation.rs
    publication_crash.rs, retirement_crash.rs, resource_pressure.rs
```

Directories with listed children have one named responsibility per file;
`mod.rs` exposes the narrow boundary rather than collecting behavior. Refine a
child where ownership or lifecycle differs; obey the 400-line limit, including
tests. This tree commits semantic placement, not empty scaffolding.

Important boundary decisions:

- **Stability** is classified by live protection lifecycle. It owns the Store
  binding and registration; lower isolation owns mechanisms, not the Store
  current root or a new serving facade. No query/MVCC/replay imports.
- **Maintenance** is classified by physical operation family. It composes
  existing authorities above them; it owns neither media nor another scheduler.
  C.11 rewrite producers are siblings beneath the same family, not forks of
  `RecordPublicationDirector`.
- **Durability** stays the sole publication and durable-obligation owner.
  Adding protection cannot move its facade or put namespace sync in a latch.
- **Scheduler** is classified by resource policy and dispatch, not arbitrary
  caller labels. Backend support and security determine admissibility; later
  producer adapters cannot mint admission through class selection alone.
- **Recovery** is downstream of persisted declarations and upstream of the
  fresh serving handoff. Remove isolation's normal dependency on
  `worth-store-recovery-physics`; its three recovery-coupled source families
  (`readiness`, compaction recovery evidence/denials) lose ordinary authority.
  The existing selected checkpoint compaction product is an operation-binding
  index, not proof of rewritten physical bytes. Remove its unrelated ordinary
  read gate; do not relocate that false join into a new recovery adapter.
  Lower compaction completion checks the sealed publication, exact pre/post
  roots and both footprint bases. It proves local plan correlation, never
  byte execution, live retention, or recovered Store admission.
  No feature flag may retain the upward cycle or import reconstruction on the
  ordinary path.
- **Portable law** uses current `worth-proof` binding/progression substrate and
  current format/Foundational identity. Do not invent a generic platform
  coordinator, flatten differing scopes, or promote diagnostic records into
  cross-crate grants to avoid a visibility problem.

Update affected ordinary callers and the existing physical-isolation/security
certification families in the same owning cutover. Remove obsolete executable
readiness/publication routes; preserve genuinely independent local mechanism
tests. `CopyOnWritePublicationPlan::complete_plan()` returns only
`PhysicalPublicationPlanCompletion`; it performs no root swap or replay and
cannot issue executed Foundational evidence. Actual publication consumes
namespace-durable mutation members in Store's sole current-root owner and
returns `CompletedPhysicalRootPublication`. Do not perform a repository-wide rename or reopen unrelated legacy
S.7/S.10 implementations merely because their vocabulary is nearby.

## Cost Contracts

Counters observe real boundaries, not receipt assembly. Required dimensions:

| Boundary | Bound and useful evidence |
| --- | --- |
| Root capture/release | Bounded slot acquisition plus indexed root bookkeeping; no descendant I/O/materialization, all-reader scan or historical-root walk. Count slots, index probes, lock waits and acquisitions/releases. |
| Read/scan | Touched routing path and current frame/window; C.6 memory and C.9 cold/warm validation costs preserved. Count frame faults, bytes, checked ranges, copies and live protections. |
| Conflict admission | Canonical bounded footprint plus indexed overlapping active scopes; no all-Store traversal. Count footprint members, lookups, actual overlaps and retries independently. |
| Rewrite | Selected source/candidate bytes plus affected metadata paths, WAL and mandatory barriers. Charge source validation, retained candidates, scratch and write amplification explicitly. |
| Retirement | Selected candidate batch plus actual indexed obligations/overlaps. Count eligible/blocked candidates, blocker kind, scanned entries, delete calls, namespace syncs and deferred bytes. |
| Queue/service | Ready versus capacity-blocked work, owed service turns, refill holds, in-flight slots/bytes per class, foreground floor, actual dispatch/completion, cancellation and residual charges. |
| Interference | Foreground wait separated into dependency, physical conflict, capacity, scheduler and backend service/sync delay; maintenance service and retained debt remain visible. |

Required zeros include deletion of protected/current/recovery-needed artifacts,
effects for denied pre-effect requests, background bypasses, media inside Signal
evaluation, structural latches across blocking I/O, whole-store read-plan
materialization, unclassified post-close effects, and stale-generation reuse.
Assert exact deltas where deterministic and bounds where legitimate indexing
or backend variation exists. Never replace structural evidence with a generous
wall-clock timeout. Report latency distributions and environment; no benchmark
claim transfers to an unmeasured OS, device or workload.

## Phase Plan And Fast Feedback

The broad design is frozen here; execution reaches a real MVP quickly.
Necessary foundations land immediately before the behavior they enable.
No phase creates public operational shells backed by flags, `Unavailable`,
test-only owners, or copied evidence. Phases describe acceptance dependencies,
not a mandate to serialize every implementation task. Each new effect-bearing
path ships its cancellation, close and recoverable/indeterminate fate with its
owning phase; Phase 5 strengthens combined evidence rather than supplying a
missing lifecycle. Operational cutovers likewise occur in their owning phase.

### Phase 1: Protected ordinary reads — the first working MVP

Install the minimal downward isolation boundary, Store-owned root protection,
and fallible `records()` admission together. Consume current root, lifecycle,
security, residency and integrity authority. Cut over the Store-chunk adapter
and affected callers; initialize/drain the new owner exhaustively.

First feedback is a real read held across ordinary append/root publication,
including a cold second chunk and the new-record absence/scan-membership twin.
With one protection slot, a surviving session must keep a second acquisition
denied after its parent reader drops; final release permits acquisition again.
Owner-local tests query the actual reader-obligation index for the issued root
across publication/release: bypassing registration or releasing early must fail,
not merely change a counter. Actual deletion interlock proof remains Phase 4.
No rewrite, generic dispatcher framework or complete recovery campaign is needed
for this MVP; capture and lifetime tests establish real protection first.

### Phase 2: Exact effects and resource-governed dispatch

Consume the existing work lifecycle and qualified backend. Replace lossy scope
projection, install ordered conflict admission, and extend the one scheduler's
class/reservation/selection policy. Route current foreground, checkpoint and
scrub producers through it. Prove disjoint progress, root/whole-file conflicts,
capacity exhaustion, class preservation, continuously refilled foreground
saturation and the owed-background-turn bound with actual I/O. No later producer
may bypass these contracts.

### Phase 3: Recoverable record-preserving rewrite

Consume protected source reads, exact work admission and C.7 publication.
Install the minimal retained/candidate accounting owner, growth admission and
reserved progress headroom before any rewrite WAL/data effects; include all
publication producers that can consume those budgets. Add pending-publication
admission, the bounded rewrite payload, producer, root-member join and C.8/C.9
consumers together. Ship one selected-segment rewrite, budget/one-over twins,
the already-WAL-durable competing append and fresh-recovery cases. Prove stable
identities, byte parity, exact source ordering and owned candidate cleanup fate.
This supplies authoritative retirement candidates, never deletion permission;
physical reclamation is not required to demonstrate honest bounded backpressure.

### Phase 4: Atomic retirement and bounded retained storage

Consume durable publication and recovery-retention facts plus live protection.
Extend Phase 3's accounting/obligation owner with indexed retirement eligibility,
linear claims and durable cleanup intent/completion. Prove multiple readers,
cold descendants, recovery pins, transitive sharing and eligibility/delete races.
At the growth limit, release blockers, checkpoint and actually delete/synchronize
eligible artifacts, then successfully retry growth with unchanged hard budgets.
This phase enables reclamation, not the first retention limit.

### Phase 5: Cancellation, shutdown and crash completion

Exercise the already-installed pre/post-effect cancellation, timeout, abandoned
observer, close/drain, stale completion and fresh-incarnation semantics together.
Run remaining publication/retirement crash seams, mixed append/rewrite tails,
exact retry reconciliation and idempotent second reopen. No persistent hazard
table or scheduler reconstruction is permitted.

### Phase 6: Interference and resource-scale proof

Run the canonical larger-than-memory interleaving, independent scale axes,
supported OS/page variants and real delayed-sync pressure. Prove bounded
foreground/background selection, honest unsupported QoS, bounded retained
storage, and locality of protection/retirement. Focused mechanism tests diagnose
failures; do not rerun every historical C.7-C.9 portfolio for each local edit.

### Phase 7: Final cutover, documentation and successor handoff

Verify owning-phase cutovers left no alternate C.10 execution/admission path;
remove dead adapters and finish public examples, owner docs and enforcement.
Run final affected owner, integration, process, compiler and repository checks.
Review the matrix against actual behavior and disclose unqualified backend
claims. C.11 receives live protected reads, safe publication/retirement and
scheduled producer contracts, not a certification receipt or semantic authority.

## Parallel Work And Integration Triggers

After a short shared contract agreement, independent work may overlap phases.
Freeze root/protection binding, conflict variants, scheduler demand, rewrite
payload/retention semantics and typed outcomes before consumers implement them.
This is a spec/API agreement, not a demand for empty public types in production.

| Lane | Ownership | Start trigger | Integration gate |
| --- | --- | --- | --- |
| Stable reads | Store `stability`, read/scan adapters, isolation de-cycle | Contract agreement; coordinator assigns moved files | Phase 1 real MVP and compile boundaries |
| Scheduler/scope | Existing scheduler and Store effect-footprint/admission | Effect variants, foreground floor and owed-turn capacity policy agreed | Phase 2 actual saturation and foreground/checkpoint/scrub tests |
| Rewrite producer | Store maintenance rewrite and publication join | Pending-publication exclusion and candidate/payload contracts frozen; protection, dispatch and retention interfaces agreed | Phase 3 budgets and admission integrated before rewrite effects, including the durable-predecessor race |
| Recovery/format | Canonical maintenance records, C.8 dispatch/reconciliation | Payload and durable-retention rules frozen | Mixed-tail and crash cases against producer output |
| Offline observation | Independent new payload parsing and raw artifact observation | Canonical declarations frozen | Disagreement/corruption tests; no runtime-parser dependency |
| Retention/retirement | Store retention/retirement and isolation eligibility mechanics | Protection, growth/headroom budgets and durable-obligation contracts frozen | Minimal accounting/admission joins Phase 3; deletion and pressure recovery join Phase 4 |
| Scenario evidence | Existing integration targets and process driver | Exact facade/protocol/seed contracts frozen | Real executable journeys when adapters become available |
| Persistent review | Read-only architecture, test honesty and scoped code quality | First coherent MVP; incremental updates afterward | Material findings resolved at each affected boundary |

The coordinator owns manifests, shared exports, moved-file sequencing and final
integration; other writers do not race those files. Reviewers are read-only.
Use available native subagents or explicitly requested CLIs/models; prompts
state deliverable, file ownership, contracts, start trigger and acceptance.
No custom CLI configs, fixed agent quota, or new orchestration system is needed.
Run as many lanes concurrently as actual dependencies and tool capacity permit;
five to eight assignments can be queued without pretending all are independent.

A contract defect pauses only its consumers; the owner repairs it and its
boundary tests before those consumers resume. An adapter may translate
representation, not invent source truth, reinterpret outcomes or widen scope.
Integrate runnable checkpoints throughout. Do not save every join for a final
connector. Worktrees, if used, need a named integration owner and cleanup after
accepted changes are preserved; no accumulation of abandoned milestone trees.

## QA Considerations And Verification

Architecture review must inspect the root-protection linearization, exact
non-range conflict coverage and sole publication/settlement owners. Lifecycle
review must cover borrowed bytes during cancellation/close and retirement
eligibility becoming stale before media. Persistence review must examine durable
append predecessors, mixed rewrite tails, positive cleanup authority and old
checkpoint retention. Performance review must expose capacity-induced starvation
and prove recovery from retention pressure, not just bounded refusal. Test review
must distinguish exact old-root membership from unchanged payloads, verify cold
refaults, and reach the intended protection interlock rather than a setup denial.

Test organization follows the matrix's responsibility families, not one Rust
integration binary per row. Keep pure intersection/transition/property tests
owner-local. Reuse `physical_record_journeys` and consolidated UI targets.
Use the process executable only when process identity or real death matters.
Mutate the highest-risk seams (protect-after-observe, live-lease deletion,
empty-scope disjointness, early permit release and background sync bypass) at
their smallest honest boundaries; do not build a mutation-receipt system.

Execution products remain explicit:

- Owner/fast feedback: focused changed-owner tests plus the Phase 1/3 vertical
  specimen; target seconds to under one warm minute on the recorded developer
  machine, with setup/compile cost reported separately.
- Integration: bounded real-filesystem matrix cases, affected C.6-C.9
  regressions and consolidated public compile boundaries.
- Release/manual: canonical interference world, full listed crash seams,
  separate scale axes and supported platform variants; start with a 30-minute
  per-family budget, measure it, split by responsibility if needed. A timed-out
  or unrun case is not closure evidence.
- Hardware qualification: any claimed hard service-time or media guarantee
  on its named deployment, separately from development-process kills.

GitHub CI is intentionally line-cap-only. C.10 must not restore removed jobs
or claim these tests ran because CI is green. Use direct commands and ordinary
test output for implementation/closure verification. Retain the existing test
runner selectors where appropriate; any added selector must reject zero cases.

Before implementation closure run affected owner and scenario/compile/process
tests, formatting, the dirty Rust line-cap guard, and:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
```

Add focused boundary rules for isolation's downward dependencies, live-plan and
retirement constructor privacy, exhaustive effect lowering, and executor-only
maintenance effects. Do not police incidental private spelling instead of the
authority boundary. Do not hand-edit generated `AGENT_CONTEXT.md` files.
No proof ledger, generated case registry, source fingerprint or recursive
test-authentication layer is a deliverable.

## Documentation Deliverables

Implementation must revise these authoritative documents against real APIs:

- `bounded-physical-record-access.md`: caller-facing fallible root acquisition,
  root-bound scans, cold old-generation reads, chunk lifetime, denial, pressure,
  cancellation and close behavior. Compile the examples against the facade.
- `physical-durability-and-checkpoints.md`: record-preserving rewrite, physical
  acknowledgment, retention blockers and checkpoint/idempotency obligations.
- `physical-recovery-and-reopen.md`: mixed payload support, physical maintenance
  reconciliation, uncertain deletion, fresh leases and unsupported downgrade.
- `physical-integrity-and-offline-verification.md`: scheduled scrub remains
  observation; independent maintenance payload inspection grants no repair or
  reclaim authority.
- Existing Store, isolation, scheduler, recovery and format READMEs: actual
  owner direction, stable public entry points and supported backend posture.
  Remove examples made false by the adapter/constructor cutover.
- This roadmap's C.10 entry: current contract links and the exact C.11/C.12/C.13
  handoff when implemented, without a new closeout/status document.

## Closure And Successor Foresight

C.10 closes when every matrix obligation has adequate direct evidence at its
named boundary, no known material scoped defect remains, required checks pass,
the sole production path implements the design, and the public docs agree.
This document establishes requirements; it does not assert implementation or
reopen historical completed phases.

| Successor | Adds | Must not force a redesign of |
| --- | --- | --- |
| C.11 layouts/indexes/blobs | Bounded artifact-specific traversal, rewrite and physical-retention producers; corresponding format/integrity/observer families | Root protection, conflict algebra's ownership, scheduler lifecycle, Store executor or publication authority |
| C.12 formal rebinding | Direct models/tests of the actual acquire/publish/retire and dispatch transitions | Runtime authority; modeled verdicts never grant capabilities |
| C.13 integration | Joined production workload and sealed physical-platform handoff | Public facade placement or lifecycle composition |
| S.10 backup/repair | Explicit persistent retention and authorized recovery/repair workflows | Reader expiry law, sole durable obligation protocol, exact effect fate |
| Part II | Semantic writer/visibility admission and adapter correlation | Branch-agnostic Store scopes; no branch registry or MVCC hidden in isolation |

Foresight is paid for at the shared contracts, atomic interlocks and directional
module boundaries. It is not permission to implement all successors now. The
first shipped feedback remains a protected production reader surviving a real
root publication; later phases strengthen that same path to the full matrix.
