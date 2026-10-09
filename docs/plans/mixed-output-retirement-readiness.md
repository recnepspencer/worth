# Mixed live and retired output readiness

## Confirmed boundary and final contract

The starting supply is `b20a840959b80ee573cf3ca58abcd0208e9e81ad`.
A genuine installed producer invokes the existing vertex-replacement handler:
it preserves an anchor, creates a replacement, and retires the previous vertex.
An actual performed source command starts its typed required output and fails
with `RetainedBasisUnavailable`, subject `settled output is not current at the
selected observation`, while baseline witness preparation rejects every Retire
role. This reproduces the boundary reported by four receiving positive journeys.

The bounded repair admits and seals a native witness for performed mixed
outputs. Every declared role remains counted and stored. Live roles retain all
installed aspect revisions; retired roles retain the exact native EntityId
including generation, installed kind, creation version and deletion version.
First sealing requires deletion at the performed snapshot's own version.
Later comparison checks the original tuple at the selected observation.
Retired roles provide no live entity or aspect content coverage. No source
currentness comparison, authority issuer, resource ceiling or caller override
changes.

Retirement checkpoint reuse remains Fresh/ineligible. A native tombstone proves
a completed deletion, but the existing portable accepted-output description
has no original producer publication/role issuer binding. The final patch
preserves the existing Retire reconstruction refusal and refuses its native
fact projection. Capture therefore carries descriptive prior identity without
an eligible mixed retirement payload. A fresh demand must attempt the ordinary
producer; it cannot issue readiness from the descriptions alone.

## Placement and implementation sequence

The Entry fixture adds `checkpoint_recovery/mixed_retirement` with producer,
conditional readiness, program and installation modules. Its own test schema
is registered through the existing test-only contribution pattern and leaves
other programs' contracts intact. Both required program roots are declared so
the existing conditional inventory and required inputs are fully admitted.
The created role supplies the installed Length readiness projection.
The ordinary caller is:

```rust,ignore
let performed = request.mutate(source_edit).expect_source(observed)
    .idempotency(&command).execute_performed::<MixedProgram, MixedRoot>(&app, allocation_policy)?;
let outputs = performed.start_required_outputs(&app, &request, controls)?;
// Ordinary bounded progression and currentness produce the settlement.
```

`output_lineage/native_output_witness/preparation.rs` extracts the existing
preflight and admits every role backing and name before publication. Retire
roles own no live aspect backing, and their native sealing probes are prepaid.
`native_output_witness/retirement.rs` projects existing exact native lifecycle
metadata; it introduces no issuer and retains no speculative locator or wire.
The witness owner seals exact role/posture/kind identity and later compares the
original native lifecycle. `fact_coverage.rs` excludes dead roles from content
attribution. Existing checkpoint construction only initializes the added
posture and empty retirement slot for its already eligible live roles.

Production checkpoint capture/encoding and generated-payload restoration are
unchanged. The explicit generated restoration Retire refusal is distinct and
remains closed.

## Recovery scope control and limitation

A diagnostic used two actual same-kind native deletions at different commits,
captured opaque application bytes, released the runtime, and freshly installed
the checkpoint. The provisional retirement reconstruction and currentness
owner predicates accepted a coordinated unrelated tombstone role/fact splice.
This is a fresh native admission and recovery-owner result, not proof of an
ordinary SDK demand issuing Ready. It is sufficient to reject the proposed
capability. The final regression requires both descriptions to remain
ineligible despite their genuine matching native tuples.

The existing installed-producer checkpoint registry verifies declared role
meaning; restored lineage records a supplied correspondence without a witness;
the first reader/demand verifies original facts against the admitted native
snapshot. Accepted-output descriptions retain source, producer, partition,
idempotency, roles and facts, but no original performed publication binding.
The outer checksum detects byte damage and does not establish that binding.
Native public history summaries also do not identify Query's producer role.
A future retirement recovery design needs original owner-backed publication
provenance. New hashes, locators or current-head expectations cannot substitute.

An attempted external-cycle producer fixture was correctly refused by ordinary
source expectation with `ForeignModel`; it is not retained as a workaround.
The final fixture keeps its real same-scope source contract. Its fresh ordinary
producer cannot repeat the original retirement and stops with typed
`ProducerUnavailable`, while provider contact proves it attempted Fresh work.
No cold no-contact retired recovery is claimed.

## Acceptance and evidence

- Record authentic RED and GREEN at the final same-scope fixture source.
- Verify actual performed required-output settlement and currentness, typed
  Preserve/Create/Retire correspondence, retired absence from live queries,
  and live preserved/created members.
- Reject live output drift and same-value ABA; keep the original retained
  observation exact.
- Owner controls use real native deletion and verify generation, kind, actual
  deletion version, role/posture coverage, original tuple at later observations,
  foreign/live/released basis refusal, and no dead content attribution.
- Preserve external retired-source refusal and original decision-read locators
  in the existing retirement owner tests.
- Stop one work unit below complete preparation and at zero comparison budget
  without publishing partial authority or increasing allowances.
- Opaque fresh install must retain retirement checkpoint ineligibility; genuine
  unrelated tombstone descriptions cannot gain an original producer witness.
- Run the full existing Entry checkpoint court and doctests, relevant execution
  owner suites, formatting, source caps, whitespace and repository guards;
  obtain independent review before committing a candidate.
- The receiving owner runs the same four native positive before/after controls
  on the reviewed local supply. Shared public owner tests do not replace those
  actual receiving journeys or GUI pixel evidence.

Only Rust 1.94 is qualified here. The demonstrated baseline Rust 1.98
compatibility gate and six unchanged baseline UI boundary diagnostics remain
separate. They do not justify changing budgets in this increment.

## Qualified validation record

Rust 1.94.0, offline, unoptimized test dependencies and debug information disabled:

- The original pre-production Entry fixture reproduced the exact required-output
  `RetainedBasisUnavailable` denial. Restoring the baseline Retire preparation
  refusal against the final same-scope fixture reproduced that denial again;
  preparation was then restored byte-for-byte and rebuilt.
- Final Entry court: 34 tests and 26 doctests passed.
- Execution lineage owner court: 57 passed, one existing scheduled 100k-scale
  case ignored; retirement owner court: all 24 passed.
- The provisional fresh native recovery-owner diagnostic admitted both original
  and coordinated unrelated actual tombstone descriptions (`current=true`).
  The retained final test requires both to be ineligible (`current=false`).
- Formatting, dirty Rust source caps, agent-context freshness and diff whitespace
  passed. Whole-repository boundary reports exactly the same six baseline UI
  source-reachability diagnostics, with no additional finding.

Reproduction commands use the repository's Rust 1.94 toolchain:

```sh
cargo test --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml -p worth-query-topology-entry --offline -- --test-threads=1
cargo test -p worth-query-execution output_lineage --offline -- --test-threads=1
cargo test -p worth-query-execution retirement --offline -- --test-threads=1
```

The local court supplied `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0`,
`CARGO_BUILD_JOBS=2` and `--config 'profile.test.package."*".opt-level=0'`.
Receiving before/after integration remains with the native application owner;
no public shared fixture result claims those after controls have run.
