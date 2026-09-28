# Milestone 9.16 Runtime Phase 9 Closure Ledger

**Owner:** Runtime Hardening Track, Phase 9

**Canonical specification:** `milestone-9.16.md`

**Status:** corrected closure candidate submitted as one frozen source packet.
The independent acceptance verdict is an external review record and does not
mutate the packet after its fingerprint is published.

This ledger governs the primary-graph conditional-operation slice. A row is
closed only when its production owner, public host proof, hostile proof, cost
posture, and lifecycle evidence agree. Runtime Bridge or Signal unit evidence
alone cannot prove the Query host contract.

## Requirement closure

| Requirement | Production owner | Evidence | Status |
|---|---|---|---|
| Host-only installation | `WorthQueryConditionalApplicationRuntimeInstallation` binds the installed operation, node, predicate, clock, reconstruction, invoker, and admission source before publication | `worth-query-host/tests/temporal_conditional_operation.rs`; host has no Bridge, Signal, Relational, or Foundational dependency | Proved |
| Authoritative reconstruction | `temporal_reconstruction` executes the bounded installed application query, projects stable intent identity/revision/due/input/lifecycle/idempotency, and resolves exact source records | active, cancelled, completed, and successor-revision host courts | Proved |
| Signal-owned eligibility | Query lowers the exact semantic dependency into the Bridge-owned Signal runtime and retains Signal decision evidence without reevaluation | satisfied, suppressed/reconsidered, predicate panic, and due-wake courts | Proved |
| Fresh operation authority | `application_operation_reentry` performs fresh principal/scope admission, current-intent inspection, ordinary authorization, invariant projection, idempotency, and compare-and-commit | public operation success; real governed query and operation authorization tests | Proved |
| Stale/terminal non-contact | authoritative changes refresh and reconcile before clock promotion; re-entry also inspects current identity/revision/active lifecycle before callbacks and seals the same facts into commit | cancellation court asserts zero predicate, precondition, operation projection, and apply contacts | Proved |
| Active successor handling | current authoritative revisions replace predecessor candidates and Bridge wakes; stale retained wakes are removed without completing the successor | changed revision/due/input court commits only the successor payload | Proved |
| Exact dependency disclosure | host observations expose explicit absence and a projection-bound `scalar()`/`field(...)` view rather than the raw validated aspect | adversarial host predicate cannot read undeclared effect field | Proved |
| Relevant-work routing | commit publication synchronously refreshes the temporal-intent due index; bounded route-local journals retain only commits for installed exact records, with a separate whole-graph route only when declared | focused courts prove exact-record selection, same-kind unrelated exclusion, 100,001 unrelated commits with zero retained route entries or overrun, real per-route overrun, route-inventory replacement, and whole-graph admission; a 2,048-unrelated-row host court holds inventory and callback work constant | Proved |
| Compatible reinstallation | reinstallation prepares the complete retained binding inventory against a fresh Bridge-owned Signal runtime, reconstructs current truth, reconciles all bindings, then swaps owners; a successor generation fails closed with typed `RebindRequired` until the host supplies fresh typed bindings | active-before-observation, after-eligibility retry, cancelled/completed omission, post-commit no-duplicate, and successor-generation courts | Proved |
| Panic isolation | predicate, precondition, projection, apply, and reconstruction callbacks are isolated; conditional registry and Bridge ownership are restored on unwind | predicate-panic, precondition-panic, and reconstruction-panic retry host courts | Proved |
| Lifecycle inventory | public inspection reports exact providers, bindings, clocks, wakes, intents, attempts, leases, scheduler tasks, scheduler queues, and the owned Signal graph; explicit close reports an empty runtime, while ordinary Rust `Drop` is observed through weak liveness tokens attached to the actual Query, Bridge, and Signal owners rather than a self-published answer | close/inspect court plus an externally retained probe court that begins with live provider/clock/wake/intent/attempt/lease/Signal owners and observes exact zero after Drop | Proved |
| Typed execution provenance | every accepted clock receipt exposes descriptive lineage joining intent identity/revision, derived Signal wake ordinals, Signal decision, application-attempt presence, and terminal posture without exporting lower-runtime authority | successful host court asserts the exact committed lineage | Proved |
| Cost separation | Ordinary clock receipts report relevant authoritative commits independently from due-wake fan-out; cold install/reinstall carries typed Foundational binding/runtime identities and exact canonical work; each fresh wake admission prepares typed idempotency once and carries it through commit | baseline-versus-2,048-unrelated-row courts; exact two-digest installation and two-digest fresh-admission counters; zero execution/provider/projection/delivery/retry/recovery/publication derivation or text work; private-digest residue | Proved |
| Signal decision and oracle parity | Query records the class minted by Signal and classifies effects without a second predicate evaluation or eligibility restamp | exhaustive Signal-class mapping plus a certification parity matrix for satisfied, unsatisfied, failed, future, due, cancelled, superseded, completed, duplicate/reordered clocks, provider replacement, and generation change | Proved |
| Constitutional topology | host vocabulary comes through Query facades; pure meaning remains Query-agnostic; ordinary code does not import replay; dirty Rust files remain within the line cap | boundary check, agent-context check, dirty line-cap guard | Proved |
| Documentation and migration | canonical AI orientation and the conditional feature guide describe the production host path, temporal authority model, reinstallation, inspection, and prohibited lower-runtime imports | `AI_README.md`; `conditional-installed-operations.md` | Proved |

## Independent review closure

The first fresh GPT-5.6 Sol high review rejected the checkpoint for eight
reasons. Their current dispositions are:

1. The host courtroom compile-path defect is closed and the complete target runs.
2. Missing reinstallation is closed by the typed retained-truth progression and four interruption/terminal courts.
3. Stale callback contact is closed by pre-Signal refresh, pre-callback current-intent inspection, commit-sealed decision facts, and zero-contact assertions.
4. Active successor loss is closed by authoritative reprojection, Bridge supersession, local predecessor retirement, and changed due/input evidence.
5. Unrelated commit poisoning is closed by record-indexed retained deltas and an over-retention-bound hostile court.
6. Undeclared struct disclosure is closed by the projection-bound host view; explicit current absence comes from the authoritative Relational snapshot read posture rather than an inferred patch cache.
7. Inspection and acceptance evidence now includes reinstallation, terminal omission, exact callback contacts, inventory, closure, and host-only dependency discipline.
8. Reconstruction refresh and runtime rebinding were split into named semantic owners; the remaining re-entry decomposition advisory remains part of the final composition rereview.
9. Global commit-sequence gaps no longer masquerade as journal overrun: exact
   routes retain only their own commits, have their own overrun frontier, and
   route replacement bounds identity inventory. Whole-graph retention is a
   separate explicitly installed route.
10. Reinstallation reconstructs and reconciles candidate state before any incumbent binding is swapped, so a later binding denial cannot partially mutate the published owner.
11. Authoritative clear is represented as `current == None`, reverted-clean is non-invoking, and public clock receipts now carry typed end-to-end lineage.
12. Lawful snapshot absence no longer reaches present-only accessors: those
    accessors were deleted and every materializer or fingerprint consumer must
    exhaustively handle `Option`.
13. Ordinary re-entry is decomposed into fresh authority, current-intent,
    admitted-projection, and same-commit sealing owners.
14. Reconstruction work now preserves the structural application-query receipt
    and a baseline-versus-2,048-unrelated-row court proves exact counter parity.
15. The named host/internal-oracle matrix is independently identifiable,
    including provider replacement as fresh publication (distinct from
    generation rebind denial), and runtime abandonment publishes externally
    observable exact-zero Drop inventory.
16. Temporal binding, runtime-binding, and idempotency meaning no longer use
    private SHA grammars. Named Foundational bases derive typed fixed-width
    digests in the installation or fresh-admission lane, retain exact work, and
    are carried through warm execution and commit.

## Final verification

- `worth-query`: 2,571 library tests, 319 installed operating-world tests, 37 public declarative journeys, 22 runtime journeys, and documentation tests;
- `worth-query-execution`: 727 tests plus 26 documentation tests;
- `worth-query-host`: all owner, host-surface, 20-test temporal courtroom, and
  documentation tests;
- Runtime Bridge: 955 tests plus complete compile-fail and documentation lanes;
- Relational: 1,040 tests (25 ignored by owner policy) plus complete integration,
  compile-fail, and documentation lanes;
- certification: all 15 hostile, residue, and compiler-boundary tests plus the
  12-case temporal host/internal-oracle parity matrix;
- focused Signal conditional execution target: 18 tests;
- boundary check, agent-context check, dirty Rust line-cap guard, formatting,
  and `git diff --check`: green;
- final acceptance gate: one source-bound GPT-5.6 Sol high
  requirements/tests/composition verdict over the published fingerprint.
