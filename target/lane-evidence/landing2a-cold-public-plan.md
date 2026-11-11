# Cold member public-flow migration boundary and plan

Base b736be6f47. Governs Landing 2a BC9002 removal. The source trait is already
implemented externally by ImmutableCheckpointMemberSource. A Native lib-unit
cannot consume that implementation because it has a separate crate identity;
no bridge, runtime lookup or memory substitution is allowed.

First coherent public slice: move the sticky damaged-member transaction read/write
and damaged-tail recovery courts from cold_members.rs. Keep all original typed
subject/cause, sticky retry, read-count, publication-head and untouched receiver
assertions. Court setup owns an explicit external journal medium. Capture uses a
real durable producer, genuine original artifacts and actual external members.
The existing cold_recovery testkit owner will expose unrecovered source custody;
its existing recovered fixture will use that same constructor. No observer door
or product API changes. Add a cold-residency area court: current certification
court already has35,909 Rust lines, so extending it with the reviewed80-file
cold family would violate the15,000-line split law. Exact frontier owner entry,
parent delegation, workspace membership and module lines are merge seams.

Then apply the independent86-test claim map: preserve public observations in
this public court; move private resident-state/accounting claims to C2 with exact
restore references. Class3 absent APIs require a typed owner design, not aliases.
The local providers remain mounted until all their actual consumers have moved.

Verification after the coherent batch: Root-only check/tests, selected cold cases
and existing cold-recovery public courts affected by the shared fixture change;
no Query/Store public input changes. Boundary, caps, fmt and context at completion.

The same batch also moves all three public envelope read/stream/inspection courts.
All original five surface variants (including ChipSimulation) and their typed
failures, warm equality, selected-body read counts and zero publication remain.
The local EnvelopeSurface routing helper stays until its matrix consumers move.
The source fixture admits an explicit producer profile for that existing case.
Selected proof packet: the five moved courts plus the existing nonhead custody
court (the only other caller of recovered_with_history). One initial check is
green; expanded batch is compiled by its one owned focused build before running.
Four unambiguous private-cost/residency courts move toC2 under the new ruling,
with restoreb736be6f47:path; their names are frozen in the companion JSON.
Other reviewed C2/mixed claims remain untouched pending complete decomposition.

Review correction: two proposed whole-test C2 moves were mixed. Restored
replay_retention_releases_from_owned_resident_custody_after_cache_clear and
concurrent_replay_references_share_one_physical_pin_set in full; public successful
retains/releases, extra-release false, concurrency and provider no-load claims
will migrate before their private pin/cache parts move. No production/testkit or
public court input changed after the green6-case packet.

Fresh review correction: restored the whole discarded_visibility counter court.
Its successful nonempty partially resident scan/incoming warm-equivalence claims
also need a public port before only accounting assertions move. The original
86-test classification cannot authorize wholesale movement of a mixed test;
implementation now checks every independent claim, not its primary counter.
Only four entirely private courts move in this candidate. Baseline unchanged.
