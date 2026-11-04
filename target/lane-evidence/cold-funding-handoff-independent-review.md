# Independent cold funding handoff review

Reviewed 2026-11-04 against HEAD `852154d4c4bfeca63b61654a3eec125341139cb5`
and the actual dirty source. The requested `bf5e74e0c3..852154d4c4` range has
no implementation changes under crates/workspaces/tools; the funding
implementation being reviewed is carried dirty work. No builds, tests or
source edits were performed by this reviewer. Only this review file was
written.

Authority: AGENTS.md; all coding guidelines; M3 D11, D12 and D17; the approved
`cold-region-funding-design.md`, including its final ownership, traversal and
test-placement corrections; and the supplied exact scope ruling. The qa-loop
review skill was applied. I did not read the coordinator's draft report,
failure map or progress document, or another reviewer's conclusions.

## Numbered findings

1. **HIGH — Cold selected DATA still cannot reach Complete emission.**
   `storage/residency/checkpoint_capture/complete.rs:68-70` only asks
   `lease_resident_full()` and returns SourceUnavailable when absent. It never
   demands ordinary Full materialization. In `cell/resident_leases.rs:96`,
   even a materialized backed Full is excluded by `self.backing.is_none()`.
   Consequently the actual same-cell, counted Full lease required by the
   approved design cannot be acquired for the changed cold region. Flat covered
   reuse in `checkpoint_complete/regions/covered.rs` avoids this only for an
   exact covered leaf; it cannot handle the current changed region after a tail.
   The existing raw `funding-finish-native-selected-courts.log`,
   `funding-finish-host-bounded-reopen-kill-ack.log` and
   `funding-finish-store-refusal-and-ack.log` contain SourceUnavailable failures
   at this boundary. The resume's three green Host cases do not execute the
   failing image-plus-nonempty-tail recapture or bounded-tail case.

   Root cause: the approved cold materialization/emitter cutover is still
   unimplemented, rather than a Query binding or topological defect. This
   prevents whole FUNDING completion under M3 D12 and MENTALITY 2 and 4.
   Correction: complete the granted ordinary owner integration after the exact
   private-file placement handoff; demand funded Full on the actual cell and
   emit from that exact counted lease. Do not bypass the backing guard while
   funding is absent. Verify the previously failing Native recapture/retry/index
   cases, Host killed image+tail and unchanged <=40 bound, Store observer
   recovery, and the cold filesystem zero-covered-canonical-read case.

2. **HIGH — Ordinary cold Head/Full reconstruction still escapes its request's
   retained-memory and work accounting.** `region_load.rs:120,210` and
   `resident_history.rs:121` still return `_memory: None` after decoding,
   cloning, folding and reconstruction. Both Full loaders still call the
   unbudgeted commitment capture. `history_fold.rs:44,54,78` holds/returns raw
   `Arc<RegionHistoryImage>` and an ordinary BTreeMap cache; its row/chunk/result
   vectors and metadata clones lack reservation custody. `charge_output`
   limits serialized length and consults the request, but does not admit actual
   native allocations or charge reconstruction work. The image returned from
   `read` also outlives the explicit `drop(folder)` in `region_load::load_full`.

   Root cause: the concrete preflight and funded backing owners specified in
   the approved design do not exist yet. A request can therefore fund emitted
   bytes without funding the simultaneously retained Head, image, folds and
   Full DATA used to emit them. This directly violates the user's explicit
   every-load funding requirement and Performance Laws I's allocation lifecycle
   and transient/retained/peak distinction, as well as MENTALITY 2 and 3.
   Correction: pre-admit typed allocations and actual work at both ordinary
   loaders and HistoryFolder; retain reservations with each escaping backing
   through its last owner; use the existing funded Full commitment owner
   without double charging. Keep independent read/expanded-output safety
   bounds. Verify denial before growth, retained-through-last-owner, refund and
   retry, work denial, and zero live publication after a failed load through
   the existing external courts. The four proposed private files remain
   ungranted and absent; this review grants no additional source scope.

## Local corrections and preserved boundaries

The `existing.rs:56-65` correction is sound within the inspected recovery
boundary. It checks the actual Native latest catalog commit OR any recovered
index definitions. Only a zero-commit, zero-definition partial Home skips
Query's exact recovered-index binding. A nonempty recovered catalog with
missing definitions still takes RequireRecovered and fails exact binding;
there is no Register fallback. The predicate does not depend on whether a
checkpoint supplied the state. Schema readmission, native-family checks and
the subsequent World boundary remain in order. The real initializer-refusal
court still requires the missing-World-root refusal, prohibits initializer
replay, checks no provider contacts and retains mutation-lease contention.
No new material defect was found in this guard.

The growth court now lives under the existing top-level
`cost::relational::source_initial_inputs::checkpoint_admission` mount. Its
finite budgets, equal retained main-state workload, real deleted branch,
independent discarded-canonical-byte contrast, broad output/work comparisons
and zero covered-body-read assertion remain meaningful. Its evidence measures
discarded canonical payload bytes; it does not prove arbitrary history graph
shape or every cold Home axis. The two nested-cost findings are gone from the
completed final boundary run. No new material defect was found in this move.

The inspected predecessor carrier is bound to the actual selected directory,
coverage, original source/checkpoint digests and owner, rather than constructed
from an unrelated descriptive image. Prepared publication installation accepts
the sealed verified-original phase and checks the exact publication Arc.
Selected Embedded refusal occurs before original body protection/demand;
carried members require the destination's supplied actual selected membership.
Region reuse remains flat-only and checks captured leaf commitments/counts.
The lineage changes carry selected immutable members and append tail rows;
the inspection found no new public authority or alternative wire-format lane.
No production format cutover was introduced by the guard/topology correction.

In the affected tracked test diff, no tests were deleted or ignored and no
existing assertion was weakened. Embedded-to-External fixture changes retain
their semantic assertions and provide actual filed membership, consistently
with the explicit selected-Embedded refusal. The new cold-recapture case adds
an assertion. The <=40 Host bound, cadences and timeouts were not relaxed.

## Completed evidence and limits

Raw logs were inspected, including the completed final boundary footer:

- Query final owner check and Query Clippy: exit 0, with recorded warnings.
- Query owner formatting and Root court formatting: exit 0.
- Root court check: exit 0.
- Top-level growth: 1 passed, 0 failed, 0 selected ignored.
- Host partial/refusal, journal-only restart, selected-Complete restart:
  3 passed, 0 failed, 0 selected ignored. These ran after the final guard check.
- Dirty line caps: exit 0, 63 Rust files checked.
- Final generated-context check: exit 0 after tool generation.
- Final boundary: exit 1, exactly three BC9002 Native source-court locations:
  `cold_source.rs`, `cold_source/protected_reads.rs`, and
  `support/durable_medium/runtime_attachment.rs`. These files are untouched in
  both the reviewed commit range and dirty source. They are reported as queued
  repository debt under AGENTS.md, not attributed to the guard/topology change.
  The complete boundary lane is nevertheless not green.

The ordinary funding implementation and its memory/work controls have not
run. The previously failing cold Native/Host/Store cases, cold filesystem
zero-covered-read proof, final CDC/ACK/refusal/recovery selections and full
Root/Store Clippy have no new successful evidence in this resume. Earlier
SourceUnavailable failures remain unresolved by the actual storage source.
Green local correction evidence cannot establish whole FUNDING readiness.

Local guard and cost placement: acceptable within the inspected boundary.
Whole FUNDING implementation and certification: incomplete, with findings 1
and 2 still open.

Overall: **REJECT**.
