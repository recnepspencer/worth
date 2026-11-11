# Round 2: admitted authority and prepaid cleanup storage

Round1 is insufficient for H10: consumer-root passed, all3 genuine examples
still overflow. Its Source changes are sound and retained; the active maximum
variants stayed large. Invariant_reads retains exactly the old176008/116792/
88728/88168 frames. Required/shared progress to producer preparation frames
135016/94088/66152/94632/146456. No stack/profile/example change.

## Direct measured ownership

The real FixedOperation is37920 bytes: state21344, cleanup8880, installed action
3376, description2096. Cleanup embeds managed cleanup7672 even while None.
AdmittedApplicationOperation8912 embeds Core8760, carried in PreparedCommit18096
and RunningCommit25792 and producer preparation/returns. These are actual
compiler members, not inferred string-count correlations. Core and managed
cleanup are the two definition changes. Receipt5264 stays measured, unchanged.

## Admitted core: one unique original-funded box

At authorization/admitted_operation_core.rs, keep the existing move-only Core
name and authority. Move its current fields into a plain private-domain Data
struct. Core owns Box<Data> plus a private original ExecutionMemoryReservation.
No Deref/DerefMut or new public type/getter/door. Every existing named access
method continues to inspect or mutate the same fields through data. Mechanically
update owning field moves and const getters with direct Box field access.
No Clone/Default/Deserialize, no brand/tag/input changes. Typed wrapper stays
unchanged except its embedded Core now has pointer/ticket size.

There are exactly3 Core constructors: conventional.rs, selected_conventional.rs,
and capability_admission/preparation/observation/admitted_access/operation_progression.rs.
Each already has the actual original ExecutionRequest. They build the same Data
and call a single Core owner constructor. That reserves size_of<Data> on that
request before Box allocation, stores the ticket on Core, then returns Core.
Use mandatory_review.rs's existing reserve_retained_memory and
relational_read_denial(stop.into(), operation) translation. No replacement grant.
Rewrap/binding/phase transfers move the original box/ticket unchanged. The
private ticket field prevents external owner code constructing an unreserved
Core literal. Original already-built fields may be dropped on a pre-effect
reservation refusal, as on existing tag/brand refusals; no effect has run.

Expected Core <=128 bytes, typed admission <=512; Data about9KiB (<=16KiB guard).
Update the retained owner census's real fields and Data observation. All old
field/access/legality assertions remain; no compatibility wrapper or alias.

## Managed cleanup: fill a prepaid state slot

At application_attempt/compare_and_commit/cleanup_required.rs, the existing
CleanupRequired::reserve(policy) already reserves/allocates an empty envelope
before effects. Extend this one quote to size_of<Envelope> plus
size_of<Option<ManagedCleanupState>>. Allocate both the existing envelope and
one empty boxed state slot there, with the original ExecutionAllocationPolicy.
Do not allocate, consult or refuse later in fill/into_parts/from_parts/retry.

ManagedCleanup becomes two owned pointers: the initialized boxed state slot
and the existing return_owner envelope. State holds the same failure, original
commit, runtime, Bridge assembly and last_denial. Filling consumes the reserved
slot. Splitting moves it out with the managed owner and retains the emptied
original envelope/funding; recombining removes the back-reference before putting
it into that same envelope, preserving the existing cycle-free transfer.

Public signatures and borrowed failure/last-denial/cleanup observations stay
unchanged. Retry takes actual state from its slot only after the checked scope
and checkpoint; success moves the same Native receipt/original evidence;
refusal refills the same slot and returns the same owner. Scope/issuer refusal
never removes state. No owner is dropped to make a success. Funding stays with
both original allocations until final cleanup disposal. Quote from actual
owner types, never an observed peak or new cap.

Expected managed cleanup <=64 bytes; Publication Cleanup~1.3KiB instead of8880.
The public managed receipt is unchanged. This shrinks the passive None variant
through all Publication operations and owning refusals without late allocations.
Split state/retry responsibilities if required to stay<=400lines. Protected
Native/World files remain unedited.

## Evidence and budget

Round2 settles the retained admitted authority and passive cleanup variant which
round1 left large. Finish all3 constructors, Core accessors/moves, envelope
transfers and census expressions before checking. Add/retain meaningful owner
checks for original box funding, refusal without new backing, exact cleanup
state retention on retry refusal and split/recombine identity. Use existing real
fixture/owner tests; no fabricated runtime or effect. No assertion weakening.
One coherent check batch for reached workspaces; unchanged Root source/green
logs reused. Then one focused build/run batch selecting changed cases, 3 red
ordinary examples, consumer if affected, Bank parents and type guards. Bank
parents run directly with stack env absent. No green unchanged input rerun.
Rerun whole-frame census after final artifacts. Round3 reserved for demonstrated
remaining failures; write its risk before starting. No bigger stack/thread.
