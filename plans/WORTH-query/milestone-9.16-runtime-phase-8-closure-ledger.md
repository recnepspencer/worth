# Milestone 9.16 Runtime Phase 8 Closure Ledger

**Owner:** Runtime Hardening Track, Phase 8
**Canonical specification:** `milestone-9.16-runtime-phase-8.md` (which is
governed by `milestone-9.16.md`)
**Governing finish reference:**
[`milestone-9.16-runtime-phase-8-finish-plan.md`](./milestone-9.16-runtime-phase-8-finish-plan.md)

**Status:** **CLOSED (2026-08-12).** C1-C8 are proved against the final source
candidate recorded below. The final bounded hostile review found and closed one
missing foreign-runtime denial in commit reauthorization; no remaining current
production authority or semantic defect met the closure threshold.
The prior corrective audit is historical evidence and was not bound to a
reproducible final source snapshot. This ledger also preserves the
historical `R8.*`, `Q8.*`, gate, and correction-slice evidence. A historical
`CLOSED` or `PROVED` label is not current product acceptance where the finish
plan classifies the same area as provisional or deferred. In particular, undo
and redo implementation evidence remains provisional pending a future,
separately governed Milestone 9.18. Read the finish plan's
current-state ledger before interpreting any historical row in this file.
Rows that name Relational as the sole product-history owner describe the Phase
8 pre-Milestone-9.17 baseline; they do not override Milestone 9.17's later
composite Runtime Bridge branch/history authority or either component owner's
internal authority.
**Policy:** A requirement is `PROVED` only when its production owner, public
consumer evidence, adversarial evidence, performance posture, and residue
posture agree. A finding is `CLOSED` only when the root cause and every
causally dependent guarantee have been rechecked. A green broad test does not
change a row's status.

This ledger preserves the historical meaning of the `R8.*` and `Q8.*`
identifiers. It was
created under **R8.63**, which requires it to exist while gates are still open
rather than to be assembled after Phase 8 finishes: a ledger written at the end
records outcomes; a ledger written at the start governs them. Rows for Gates
8.1 and 8.2 are backfilled from their per-gate closure ledgers, which were
written before their implementations were inspected and re-verified by an
auditor who re-ran every check rather than reading a report.

New findings and revised product decisions are governed by the finish plan.
Historical rows remain useful evidence, but they do not freeze an implementation
shape or relabel provisional code as accepted.

## Final-source closure record (2026-08-12)

The implementation/test candidate is bound to baseline
`d8424890c6a7dbc7936c983e32d53a107e0216ee` and to a deterministic manifest of
all 1,382 dirty non-document, non-target paths (978 current files and 404
deletions). The manifest is the ordinally
sorted union of `git diff --name-only --diff-filter=ACMRD HEAD` and
`git ls-files --others --exclude-standard`, read through their NUL-delimited
forms, after slash normalization and
exclusion of `plans/` and every `target/` segment. Each line is
`<path><TAB><SHA-256-of-current-bytes>` or the literal
`<path><TAB><deleted>` (including angle brackets); lines are joined with one
LF, with no terminal LF, and the UTF-8 manifest is SHA-256 hashed. Its SHA-256
is `fe88c6e7383dca1fda1f186a88c9ad5fcb5112b8690f6254a4ef15aa75a0e41c`.
Documentation is excluded so this record and final reviewer verdicts do not
make the source fingerprint recursively self-referential.

Final standing verification from that candidate:

- `cargo fmt --all` in both Query and Bank workspaces, followed by final
  formatting checks: pass.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` in
  Query and Bank: pass.
- `cargo test --workspace --all-features` in Query: pass, including the
  2,571-test main crate, all compiler courts, composition guard, and doctests.
- `cargo test -p bank-server --all-features` in Bank: pass, including every
  `bank-server` unit, integration, compiler, and doctest court. The scheduled
  high-operation speed probe remains explicitly ignored.
- `cargo test -p bank-server --test ordinary_mutations --all-features -- --nocapture`:
  91/91 pass, including the complete Phase 8 recovery, undo, safe-retry,
  cross-gate, expiry, and retained-authority matrix.
- `cargo test -p worth-query-certification --test managed_run_compile_fail`:
  pass for the supported lower provider-session protocol, including public
  wrong-phase readmission attacks. The move-only cleanup case is owned by and
  passes in `cargo test -p worth-query-certification --test compile_certification`.
  Application-attempt
  affinity and the private prepared/running/progressed commit phases are not
  public facade contracts; their topology is proved by owner-private fields,
  consuming transitions, source/visibility enforcement, and runtime affinity
  substitution courts rather than a nonexistent consumer import.
- `cargo test -p worth-query-certification --test application_aftermath_compile_fail`:
  pass, including the corrected raw-provider-receipt diagnostic.
- `cargo test --manifest-path tools/boundary-check/Cargo.toml --test authority_sealing_contracts value_gate -- --nocapture`:
  63/63 pass, including owned trait-object producers, a legal borrowed twin,
  standard `PhantomData` metadata, a hostile same-named owning wrapper, and
  public-trait associated-output producers implemented for primitive self
  types with a legal borrowed control, plus a free primitive
  trait-associated projection with qualified-identity and borrowed controls.
- `cargo test --manifest-path tools/boundary-check/Cargo.toml --test query_contracts internal_authority_topology -- --nocapture`:
  19/19 pass. The AST-backed court verifies owner-private fields, semantic
  child placement, owner-only constructors, permit-gated observations and
  splitting, every owned producer of the protected authority set against an
  exact owner allowlist, requires every protected name to resolve to a real
  source declaration, and proves the absence of raw Query approved-elevation types
  from public Bank signatures. Visibility, constructor, raw-getter,
  mutable-axis, and producer-shape mutants prove the rule is not a raw
  substring fence.
- `cargo test --manifest-path crates/worth-proof/Cargo.toml`: pass.
- `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`:
  pass.
- `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`: pass.
- Dirty composition guard: pass; a full PowerShell scan of every Rust source
  against the canonical allowlist found zero unallowlisted files over 400
  lines. `git diff --check` reports no whitespace error.

On Windows, the workspace test commands prepend the bundled Codex Python
directory to `PATH`. The shared composition runner now probes `python` before
the Microsoft Store `python3` alias on Windows, while retaining `python3` first
on other platforms. The first full Query attempt is preserved as a failed
environment run: 2,571 tests passed before the old probe found no executable
interpreter. It is not counted as closure evidence; the corrected full rerun is.
The first non-Docker Bank workspace run found a real integration defect:
`bank-estate-certification` still accepted raw
`WorthQueryApprovedElevation` while the Bank runtime required
`BankApprovedEstateElevation`. Both command and emergency-query certification
consumers were cut over to the Bank-owned move-only wrapper; the focused
certification check and the complete non-Docker Bank workspace then passed.
`bank-courtroom` compiles under strict all-target/all-feature Clippy but its
Docker integration tests remain unavailable because Docker Desktop is not
running in this environment; no dirty `bank-courtroom` source is part of this
candidate.

Mutation/fault record:

- accepting a mismatched snapshot/application-attempt association is caught by
  exact `SnapshotHandle` equality. A real owner/peer/foreign court substitutes
  runtime, installed operation, resource binding, operation attempt, operation
  slot, schema binding, snapshot, graph-work session, and managed run one axis
  at a time and requires the exact singleton mismatch for every substitution;
- stale raw-receipt trybuild output was rejected until the expected diagnostic
  matched the current forbidden facade route;
- returning the generic preview executor publicly made typed next-action
  wrappers bypassable; privatizing it restored the compiler-visible route;
- AST producer mutations through derived `Default`, free functions, direct and
  wrapped aliases, function-pointer constants, arrays/owning wrappers,
  `impl Trait` associated outputs, owned trait objects, public trait returns,
  empty named/tuple enum variants, standard `PhantomData` metadata, and a
  same-named owning `PhantomData` impostor, primitive public-trait associated
  producers, free primitive associated-type projections with
  qualified-identity controls, and their borrowed controls are represented in the 49-case
  value-gate court;
- the real registration failure is injected at
  `register_provider_attempt -> register_application_attempt`; its test proves
  exact victim cleanup while a distinct peer remains registered in the real
  attempt store, observes the count remain one, and then progresses that same
  peer to commit;
- the internal producer court rejected a phantom revalidation observation
  name; it now names both real revalidation observation types, allowlists the
  one broad owner mint, and contains a declaration-completeness mutant;
- the elevation validation reorder was caught independently by the
  `ElevationRequired` and `ElevationTransitionRequired` twins;
- the dirty line-cap gate forced the authorization observation, installed-plan
  binding, delegation observation, and currentness responsibilities into named
  owners; the final dirty scan covers 961 current Rust sources with zero
  non-allowlisted files over 400 lines. The advisory scan inspected those same
  961 files and all 460 greater-than-60-line or at-least-five-argument
  candidates with zero scan errors. The supported capability-refresh finding
  was decomposed into preparation, active-currentness observation, decision
  replacement, and supporting-capability refresh before this record was bound.
- public elevation phases no longer project their raw application commit
  receipts to publication or Bank consumers. Publication derives its closed
  receipt from `WorthQueryApplicationCommitPublicationSource`, and the
  aftermath compiler court attacks all four former raw receipt getters.

Final hostile review produced three materially distinct results: the test and
ledger reviewer accepted the causal courts; the composition reviewer found no
current production semantic or authority defect across the complete dirty Rust
scope; and the independent source critic found the missing commit
foreign-runtime denial, which was then fixed and proved by a two-runtime
regression. A separate suggestion to expand the explicitly protected internal
producer catalog was classified as non-blocking because every named current
type has private fields and owner-only construction and no executable bypass
was demonstrated. Exotic consumer re-export patterns remain unsupported and
are not closure obligations. Closing Runtime Phase 8 did not close Milestone
9.16. Bank Phase 5 subsequently closed on 2026-08-13; Runtime Phases 9-10,
Bank Phase 6, and Closure Phase 1 remain declared work before moving to
Milestone 9.17.

## Current correction ledger (2026-08-08)

These rows control current closure. Every older `PROVED` or `CLOSED` result is
historical until its affected correction row and downstream guarantees are
reproved.

| ID | Exact correction claim | Required evidence | Current result |
|---|---|---|---|
| **C0-AUTHORITY** | Governing documents describe the hostile-QA state and preserve Relational lineage ownership plus provisional undo/redo containment. | Consistent status headers, dependency-ordered correction plan, no current document treating historical closure as authority. | **PROVED.** The specification, roadmap, finish plan, and this ledger consistently close Runtime Phase 8 while leaving undo/redo provisional and Milestone 9.16 itself open. Relational remains the sole Phase 8 lineage/history/head authority and Runtime Bridge gains no lineage role. |
| **C1-FACADE** | Accepted facades cannot reach provisional aftermath or proof-only lifecycle controls, including through compatibility aggregates. | Positive provisional import; negative primary-graph and installed-compatibility imports; execution root contains no wildcard aggregate. | **PROVED; FINAL REBINDING COMPLETE.** Execution's accepted compatibility facade enumerates semantic owners instead of glob-exporting the private root. The aftermath compiler session accepts the provisional import and rejects both accepted-facade substitutions. |
| **C1-WIRE** | Descriptive opaque wire identity cannot mint or readmit a live recovery handle. | Correct receipt-mint positive case; opaque-wire substitution fails with the intended type mismatch; no always-deny public surrogate API. | **PROVED; FINAL REBINDING COMPLETE.** The compiler courtroom accepts minting from the execution-owned receipt, rejects foundational opaque material with the intended receipt mismatch, and contains no always-deny surrogate. |
| **C2-DECLARATION** | Static per-operation aftermath/external-effect cardinality is compiler-enforced and lifecycle evidence uses causal worlds. | Typestate authoring, duplicate compiler attacks, corrupt artifact owner denials, removal of test-only authority constructors. | **PROVED; FINAL REBINDING COMPLETE.** One sealed operation definition requires explicit external-effect and aftermath choices before `finish`; flat slot insertion is no longer public authoring. The compiler court proves zero/one positives plus omission and duplicate negatives, and both ambiguity resolvers retain direct owner denials without fabricating a public reconstruction journey. Query definition authoring is staged across every required axis; principal roles and closure postures are named typed requirements. Installation compiles executable operation contracts and capability-only graph authority through separate doors. Accepted C2 evidence uses production derivation rather than test-only authority constructors. |
| **C3-PROTOCOL** | Portable protocol vocabulary and explicit compatibility evolution are owned below Query without granting authority. | Dependency proof, independent producer/consumer declarations, frozen bytes, v1/v2 window and downgrade posture. | **PROVED; FINAL REBINDING COMPLETE.** Foundational owns validated version-free family identity, exact positive version, compatibility windows, and typed predates/exceeds/retired outcomes; custom deserialization rejects invalid raw values. Query declaration, installation, canonical identity, committed outbox, fresh restore, and transport carry family/version unchanged and own no support policy. Bank owns frozen v1 and v2 producer projections; the separate rail owns its startup compatibility profile and distinct decoders. Real-process proofs cover compatible versions and hostile semantic denials while leaving owner state unchanged. Runtime Bridge has no role in the lane. |
| **C4-OUTBOX** | Observation, dispatch, redispatch, and causal identity remain exact under later mutation and cross-runtime ordinal reuse. | Exact-commit read, complete corruption matrix, runtime-affine identity twin, real retry attempt comparison, structural counters. | **CLOSED; FINAL REBINDING COMPLETE.** Exact-commit and cross-runtime/retry evidence passed again in the final Query and Bank workspaces. |
| **C5-PREIMAGE** | Installed pre-image demand preserves typed entity/aspect/field identity; retained fields and the outbox remain bound to one reusable whole-commit evidence artifact; ordinary no-demand commits perform zero footprint work; aggregate projection has typed same-level phases. | Bare-string compiler denial; cross-axis install/reinstall and canonical drift; duplicate/mixed-record denial; production footprint matrix; scale-sensitive counters and eager-construction mutation; normal/equivalent/lost-response/interleaved bundle twins; aggregate cache/overflow/budget mutations; composition audit. | **CLOSED; FINAL REBINDING COMPLETE.** The final workspace and composition gates reran the typed demand, retained evidence, zero-work, and aggregate proofs. |
| **C6-PUBLICATION** | Publication is a closed projection with exhaustive owner mapping and complete-surface noninterference. | No execution receipt/types/IDs, safe Debug, decision-relevant twins, correct privacy compiler case, boundary evidence owner. | **CLOSED; FINAL REBINDING COMPLETE.** Publication stores publication-owned closed descriptions, and final compiler, noninterference, Bank, strict-check, and composition courts are green. |
| **C7-PHASES** | Provider attempt identity is typed and phase progression is owner-sealed. | Typed affinity; one atomic attempt-state aggregate; private consuming phases; exact authorization contexts; association-preserving capability/effect accumulators; opaque Bank operational wrappers; cross-runtime/snapshot/plan/session/branch and wrong-phase public compiler attacks; internal source/visibility enforcement; interleaved-attempt cleanup matrix; initial/equivalent-retry routing; composition audit. | **PROVED.** One owner-minted affinity binds runtime, operation attempt, installed binding/operation, resource, snapshot, graph-work session/run, branch, and provider terminal binding. Provider and authorization progression consume nested owner-private phases/contexts; registration and commit stores use one affinity-keyed aggregate; capability rules and provider effects retain their per-item associations; Bank recovery exposes only Bank-owned handles, results, and denials. Real interleaved attempts cover preparation, registration, read-set, invariant, pre-transaction, abort/cancel, response-loss, publication, and abandonment cleanup. Supported consumer compiler cases reject public wrong-phase and raw-authority substitutions; internal privacy is checked through AST/source visibility enforcement. The final runtime-axis attack also proves a foreign application runtime cannot reauthorize the exact commit basis. |
| **C8-CLOSURE** | Final evidence is honest, nonvacuous, mutation-sensitive, and tied to the exact final source. | Source fingerprint, exact commands/results, complete ledger attack, standing verification. | **PROVED.** The deterministic fingerprint, command record, environment exception, mutation record, bounded hostile reviews, and final standing checks are current. No current production authority or semantic defect remains open. |

The final code-quality review scope contains 961 current dirty Rust files and
394 deleted Rust paths. The line-cap gate has inspected every current file; the
primary advisory scan inspected all 460 candidates, and the fresh final
composition reviewer must independently inspect that same frozen scope before
C8 closes. Earlier body
inspection classified ordinary typed signatures, exhaustive matches,
declarative inventories, and single-scenario tests as advisory-only. Every
confirmed authority, evidence, or topology defect is mapped in finish-plan
section 0.6 and must be closed or explicitly retained as provisional/deferred
before current closure.

## Finish slice F6A reopening and closure ledger (2026-08-08)

This is the current controlling ledger. It records the affected F6 claims from
reopening through correction while preserving their earlier results below as
evidence history. No row moved to `PROVED` merely because the pre-correction
broad suites were green.

| ID | Reopened claim or defect | Required closure evidence | Current result |
|---|---|---|---|
| **F6A-COMP-1** | The 148-line outer commit coordinator mixes authority validation, retained idempotency, provider preparation, managed-run admission, progression, cleanup, receipt completion, and dispatch. The 205-line provider progression mixes four distinct proof/lifecycle phases. This contradicts F8.11-C1's “linear and small” claim. | Private move-only phase transitions; small same-level coordinators; fatal composition scan over every F6A Rust file; lifecycle and failure-path owner tests. | **PROVED AFTER CORRECTION.** `commit_preparation`, `managed_commit_run`, `provider_session_admission`, `application_attempt_registration`, `invariant_progression`, `authorized_progression`, and `commit_completion` own distinct move-only phases. The entry coordinator now performs prepare/start/progress/finish only; provider progression composes named phases. The dirty idempotency resolver was also split into bounded lookup, authoritative projection, and restoration. All 29 F6A Rust files have zero composition candidates/errors, the dirty 400-line guard passes, and all 44 application-attempt owner tests plus the idempotency branch test pass. Decision-fact binding failure now explicitly aborts its staged session. |
| **F6A-FACADE-1** | `phase8_residue::r8_47_host_facade_exports_aftermath_next_action_surface` searches the whole execution facade and therefore reports provisional undo/redo names as accepted `primary_graph` exports. Raw implementation source is also presented as public reachability. | Compile-pass import from `provisional_aftermath`; compile-fail import of the same experimental surface from accepted `primary_graph`; source residue demoted to supplemental structural evidence. | **PROVED AFTER CORRECTION.** The 35-case consolidated aftermath trybuild session includes a host-facade positive provisional import and an E0432 accepted-facade negative twin. The false Bank reachability test was deleted; its three remaining source scans are explicitly supplemental and pass. |
| **F6A-FIX-1** | The second `BankDisbursementRedoAdmission` doctest imports redo types from accepted `primary_graph`, so it fails at an obsolete import rather than the claimed caller-action substitution boundary. | Import from `provisional_aftermath`; pin and inspect the intended call-contract diagnostic; run the owning rustdoc suite. | **PROVED AFTER CORRECTION.** Both redo substitution examples import their real stable/provisional dependencies, are pinned to E0061, and pass in the 11-case `bank-server` rustdoc suite. |
| **F6A-RETRY-1** | F8.9-C3 says every safe retry has a new attempt identity and that constant ordinal reuse fails an oracle. Existing unit evidence compares caller-supplied ordinals, while the real Bank rail assertion checks only internal retry-ladder adjacency. Neither compares the original production dispatch attempt with its retry. | One runtime-owned fresh-dispatch operation used by initial and retry paths; owner test proving same emission/correlation and distinct attempt identities; constant/reused-ordinal mutation makes the test fail; real rail exactly-once twin remains green. | **PROVED AFTER CORRECTION.** Initial dispatch and admitted redispatch call the same private `dispatch_committed_observation_with_fresh_attempt` operation. Its owner test holds correlation/emission constant and proves unequal attempt identities plus retry completion. Mutating the runtime allocator to return ordinal `1` made the exact test fail on equal attempt identities; restoration passes. All six real Bank safe-retry rail journeys pass and retain exactly-once consequences. |
| **F6A-LEDGER-1** | The prior F6 completeness attack did not test whether source scans were masquerading as facade reachability, whether a production retry actually consumed the fresh ordinal, or whether the named commit coordinators satisfied the composition claim. | Add those attacks explicitly; reconcile every dependent status; rerun evidence from the corrected final tree. | **PROVED AFTER CORRECTION.** The final completeness attack now names all three omitted dishonest states. F8.9-C3, F8.11-C1, F8.12-FIX, F8.12-COMP, and F8.12-VERIFY were reconciled to their corrected evidence. Both full workspaces, root `worth-proof`, boundary enforcement, generated-context validation, the dirty 400-line guard, the 29-file fatal composition scan, and `git diff --check` pass from the corrected tree. |

## Finish slice F1 closure ledger (2026-08-07)

This is the controlling ledger for F8.1-F8.4. Older registry and binding rows
below remain audit history; where their feature assumptions or evidence names
conflict with this section, this section wins.

| ID | Exact closure claim | Evidence required | Current result |
|---|---|---|---|
| **F8.1-C1** | No supported feature, host facade, execution facade, recovery handle, or public identity helper exposes the recovery registry, registry slots, terminal selection, enumeration, or force termination. | Manifest and facade residue scan; public compile attack under default and all features; production Bank tests compile without a test feature. | **PROVED.** `test-support` is absent from all three manifests; registry vocabulary and slot mutation are crate-private; handle registry/slot access and enumeration are execution-owner `cfg(test)` observations only. The default and all-feature aftermath courtroom both reject the facade import and both handle routes (E0432 and two E0599 diagnostics). |
| **F8.1-C2** | Removing host registry controls does not weaken the runtime-owned one-terminal law or turn leak evidence into public test architecture. | Execution-owner concurrent terminal test, four-fate registry test, remint denials after public terminal transitions, and Bank authority/lifecycle journeys using only public consequences. | **PROVED.** The 598-test execution owner suite retains concurrent terminal and four-fate coverage. Seven exact-handle Bank tests and five recovery Bank tests pass through public consequences only; terminal expiry proves permanent claim by exact public remint denial. |
| **F8.2-C1** | Each operation declares zero or one external-effect contract and zero or one aftermath contract, with distinct typed denials before generic duplicate detection and before canonical schema identity. | Positive absence/one twins; identical and differing duplicate attacks for both families; builder-order mutation probe. | **PROVED.** Eight focused declaration tests cover absence/one, identical duplicates, and distinct external-effect and aftermath contracts through the builder. The full 143-test declaration suite passes. Removing the pre-identity builder validator changes the exact-duplicate result to `DuplicateMember`, so the ordering oracle is fault-sensitive. |
| **F8.2-C2** | Installation and reinstallation never choose the first or last contract from an ambiguous internal member set. Declaration is primary authority; defensive installation failure remains explicit. | Raw-member ambiguity tests for both families; installation source trace; mutation probe that removes ambiguity rejection. | **PROVED.** Both resolvers scan zero-or-one and return exact ambiguity denials on the second matching member; initial install maps those denials, and reinstallation rejects ambiguity. Five focused tests and the 185-test installation suite pass. Removing external-effect ambiguity rejection selects `EffectB` and fails its oracle. |
| **F8.3-C1** | An axes-only comparison cannot mint reusable dynamic authority. Query performs exact value comparison inside the owner continuation that consumes its result. | Public substrate surface inspection; compile-pass result typed as `()`; one-axis Query twins; comparison-bypass mutation probe. | **PROVED.** `BindingMatch` and every re-export are absent; a pinned public compile attack rejects its import, while the positive compile case requires comparison success to type as `()`. Query compares the admitted operation with the same handle binding used to mint exact-handle authority. The full `worth-proof` suite and 598-test execution suite pass; bypassing Query comparison makes the generation-drift twin admit. |
| **F8.4-C1** | The recovery axis sourced from `ApplicationSchemaBindingIdentity::generation()` is named application binding generation everywhere and has its own exact denial, distinct from Relational branch identity. | Source residue scan, one-axis positive/negative twin, exact `ApplicationBindingGenerationMismatch`, and documentation audit. | **PROVED.** Current Rust sources contain zero `branch_version_ordinal`/`BranchVersionOrdinal` matches. The eight binding-axis tests include the positive twin, all seven independent drifts, and exact `ApplicationBindingGenerationMismatch`; Relational branch denials remain distinct. |

### F1 ledger-completeness attack

The first attack asked what could remain wrong while the four headline findings
looked fixed. Four surviving defects were represented explicitly above:

1. a feature could still forward registry authority even if the default facade
   test passed;
2. identical declarations could fall through to generic duplicate detection
   while differing duplicates received a semantic denial;
3. declaration could be correct while installation still selected one member
   from malformed internal input;
4. renaming the field without renaming its denial could continue reporting
   schema-installation drift as Relational branch drift.

The final attack was repeated after the owner, public, compile, mutation,
formatting, clippy, line-cap, boundary, and context evidence was green. No
meaningful F1 defect survives the ledger: a forwarded legacy feature cannot
compile because the feature no longer exists; downstream tests cannot activate
a dependency's `cfg(test)` items; malformed internal member sets fail in the
installation resolver rather than selecting one; comparison yields no artifact
that could be replayed for another dynamic pair; and code residue contains no
old axis name. **F1 is closed.**

The broader Bank `phase8_` run is deliberately not relabelled green: 47 tests
pass and four real-rail tests fail identically in parallel and serial runs with
`PayloadRejected`/zero rail admission. That is the independently documented
F8.5 Rust-`type_name` versus stable-wire-identifier defect and remains the F2
entry failure, not an F1 regression.

## Finish slice F2 closure ledger, corrected by C3

C3 supersedes F2's fused wire-identifier destination with a version-free
protocol family plus separate exact version and consumer-owned window. These
rows now describe the corrected production path. C8 still owns final-source
rebinding; older Gate 8.2 and Q8.25 rows remain evidence history.

| ID | Exact closure claim | Evidence required | Current result |
|---|---|---|---|
| **F8.5-C1** | The typed external payload inseparably declares a canonical version-free protocol family and exact positive produced version. | Public authoring DX; portable compile-fail doctests; declaration source trace; raw-deserialization denials. | **PROVED FOR C3.** `ApplicationExternalEffectPayload::PROTOCOL` carries `BoundaryProtocolIdentity` plus `BoundaryProtocolVersion`. Invalid const declarations fail in Foundational-owned portable doctests; fused, uppercase, empty-segment, and zero values fail at the raw serde edge. |
| **F8.5-C2** | Rust payload identity and external protocol family/version are distinct contract dimensions. | Canonical identity trace; source residue; type-name substitution mutation; module-move journey. | **PROVED FOR C3.** Rust type name remains installed diagnostic data only. Two module-distinct payloads pass through real declaration, admission, installation, and outbox derivation with different diagnostic identities but identical family, version, and frozen bytes. |
| **F8.5-C3** | The co-committed outbox and dispatch request carry exact family, version, bound, effect, and bytes through one record-derived path. | Projection and fresh-restore tests; public construction attack; real Bank adapter journey. | **PROVED FOR C3.** Identity and version persist as separate fields. A valid eight-field restore baseline plus every omission/type/protocol corruption fails closed. The request remains private and the real Bank v1-to-`V2Only` journey proves the adapter preserves exact typed denial causality. |
| **F8.5-C4** | Moving the Rust payload type without changing family/version does not change wire behavior or canonical meaning. | Two module-distinct producer declarations; real install/outbox journey; fixed corpus. | **PROVED FOR C3.** The integrated journey establishes different Rust identities and equal installed/outbox family, version, and bytes from independent declarations. |
| **F8.8-C1** | The rail independently owns supported family, exact decoder selection, accepted interval, and retirement policy. | Separate production owner and startup profiles; v1/v2 producer corpora; source trace. | **PROVED FOR C3.** Bank owns independent v1/v2 encoder projections and frozen corpora. The rail imports neither producer nor Query; its `Current`, `V2Only`, and `V1Retired` profiles select Foundational windows and distinct v1/v2 decoders. |
| **F8.8-C2** | Wrong effect/family/version/bound/bytes and malformed raw protocol values deny before ledger admission. | Separate-process hostile matrix; exact typed postures; status/notice/count unchanged. | **PROVED FOR C3.** Crossed labels, corrupt same-length v2 prefix, dishonest bound, malformed bytes, future/predating/retired versions, fused/uppercase identities, and version zero fail with exact outcomes; every case leaves the rail owner state unchanged. |
| **F8.8-C3** | Healthy v1/v2 twins decode exact domain meaning and the retirement floor remains inclusive. | Separate-process corpus twins; exact domain oracle; threshold positive. | **PROVED FOR C3.** Both corpora produce estate `8101`, notice `8102`, and subject `8103`; v2 remains admitted at `retire_before(v2)`, catching an accidental inclusive retirement comparison. |
| **F8.8-C4** | Unsupported posture and the existing transport fault taxonomy survive adapter, Query, and publication boundaries without guessing completion. | Query exact-value mapping; real Bank-v1/rail denial/publication journey; existing fault and retry suites. | **PROVED FOR C3.** The focused Query owner test preserves exact version/posture. A real Bank v1 dispatch to a `V2Only` process publishes `{ produced: 1, PredatesWindow }` with no rail admission, making adapter collapse and publication posture swaps fail. Existing fault/safe-retry lanes remain in the standing verification set. |

### F2 ledger-completeness attack candidates

The controlling review must attempt to preserve each of these defects while
making the headline happy path pass:

1. declaration carries a stable identifier but canonical identity omits it;
2. installation carries both identities but the outbox still persists the Rust
   diagnostic name;
3. Query derives an honest request internally while the public request remains
   caller-constructible from replacement fields;
4. the Bank adapter hard-codes protocol metadata or payload bytes rather than
   forwarding the committed record;
5. the rail accepts any caller-claimed bound that happens to contain the bytes;
6. a rejected request enters status, notice, or admission-count state before
   rejection; or
7. the hostile oracle imports the production decoder's protocol constants and
   therefore repeats the same mistake instead of detecting it; or
8. the outbox constructor accepts the installed family, effect, wire ID, and
   bound as separately mixable arguments even if the current caller is honest.

This list is an audit seed, not an assertion of completeness. The final F2
review must ask again what meaningful protocol or evidence defect could survive
all current rows and add any missing guarantee before closure.

### F2 closure audit (2026-08-08)

The completeness attack found candidate 8 during review. Production now derives
the outbox record from one `InstalledExternalEffectContract`; an internal caller
cannot splice metadata from different installed declarations. Candidates 1, 2,
and 5 were exercised as temporary source mutations and each failed its dedicated
oracle. Candidates 3, 4, 6, and 7 close by compiler fence, adapter source trace,
three owner-state assertions after every rejection, and dependency/private-
constant separation respectively. Every mutation was restored before the final
run.

The QA-tests audit found distinct setup/action/oracle/teardown boundaries: the
rail is a separately spawned process whose handle terminates it on drop; hostile
cases have exact rejection oracles and inspect status, decoded meaning, and the
global admission count; the healthy twin uses independently encoded domain
values. The former in-memory “persistence” test was renamed and documented as
create-intent projection evidence so it does not steal F8.6's fresh-owner
committed-read claim.

Final-source evidence is green: 146 declaration, 187 installation, and 599
execution library tests; both application-schema and aftermath trybuild
harnesses without overwrite; eight rail exit proofs; five Bank external-effect
journeys; one mutation-free journey; and 51 Bank `phase8_` journeys. Both Query
and Bank workspaces pass `cargo test --workspace --all-features`, formatting,
and `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
The dirty Rust 400-line guard, boundary checker, and agent-context checker pass.
The composition scan reports no F2-local candidate except the pre-existing
exhaustive application-schema member serializer, which retains one named
enum-to-canonical-form responsibility. **F2 is closed; F3 is now admissible.**

## Finish slice F3 closure ledger

This is the controlling ledger for F8.6 and F8.9. Every row starts **OPEN**.
F3 may close only when the Relational owner read, dispatch source, causal event
types, public projections, retry behavior, canonical cost, and adversarial
evidence agree. A receipt-carried record value is not proof that a row
committed. Production may use only the privately sealed record-plus-`RecordRef`
binding derived from the authoritative `CommitResult`, followed by an exact
retained-commit owner projection.

| ID | Exact closure claim | Evidence required | Current result |
|---|---|---|---|
| **F8.6-C1** | A declared external effect is read from the exact retained Relational commit through its commit-carried `RecordRef`; no current-head or correlation-index search may recreate row identity. Missing/deleted, wrong-kind, and malformed fields deny rather than falling back to a record copy. Relational's entity projection exposes only live records, so Query does not claim or duplicate a separate lifecycle predicate. The single Relational owner operation admits its own sealed projection binding; callers have no separate projection-binding axis to corrupt. | Provider source trace; real Bank ordinary-front-door readback; missing/deleted, wrong-kind, and malformed owner tests; current-head and receipt-copy mutation probes. | **PROVED FOR C4.** Relational resolves the exact retained `CommitReference` without reconstruction. Query directly probes the sealed entity `RecordRef`, maps owner `EntityKindMismatch` to typed `WrongRecordKind`, projects all eight fields, and reports one exact snapshot, one record probe, eight fields, and zero index entries. A deleted record is non-visible and returns `Missing`; the former outbox correlation index and commit-time maintenance are deleted. |
| **F8.6-C2** | The committed outbox binding is sealed from the authoritative commit artifact and carries the exact record value plus `RecordRef`; the observation additionally proves the exact runtime, branch, and whole `CommitReference`. | Commit-artifact selector twins; foreign-runtime/wrong-commit/wrong-record tests; later update/delete twins; exact owner source trace. | **PROVED FOR C4.** Relational's mutation owner carries its exact/symbol-aware create-reference-to-`RecordRef` correspondence into a pipeline-only `CommitResultSeal`. Every `CommitResult` axis is private and the sole result constructor consumes that unforgeable seal, so siblings cannot detach, replace, or cross-splice the mapping, outcome, publication, validation, or execution evidence. Query binds the pending outbox create reference and record before commit and resolves only that exact reference after commit; it does not decompose lookup axes or rescan patches/changed-record lists. A real two-create commit with distinct persisted meanings proves exact reference-to-record affinity, while an unrelated identical row cannot create ambiguity or substitute its identity. The same sealed pair moves through commit evidence, receipt, recovery handle, and owner read. Later valid substitutions of every field and later deletion cannot change target-commit truth; substituting another committed record identity yields `RecordMismatch`. |
| **F8.6-C3** | Production first dispatch and safe re-dispatch consume only the exact owner observation. A caller-carried record cannot replace persisted family, version, effect, bound, payload, correlation, outcome identity, or record identity. | Dispatch call-site trace; eight-field Bank oracle and corruption matrix; record-ref substitution; compiler owner fence. | **PROVED FOR C4.** Both dispatch paths consume a sealed owner observation. Recovery retains the commit-sealed pair rather than a parallel record copy. Performed redispatch consumes a private-field owner seal whose constructor requires a module-private mint capability; `deny(private_interfaces)` makes widening the seal constructor a compiler failure. Initial owner-read, attempt-admission, canonical-derivation, and runtime-time failures are typed; the real Bank clock-failure journey proves the initial receipt-to-publication path preserves the exact denial. |
| **F8.9-C1** | Provider commit, co-committed application emission, dispatch attempt, and dispatch observation have four distinct canonical identities. The root binds Query runtime, exact record identity, commit, branch, version, and correlation; every successor binds its exact predecessor. | Exact ladder assertions; cross-runtime equal-ordinal and different-record twins; predecessor mutation probes. | **PROVED FOR C4.** The ten-entry provider basis includes Query runtime and entity/relation partition, slot, and generation with exact commit axes. Equal local ordinals in distinct Query runtimes and equal record values at distinct `RecordRef`s produce different roots, while successors remain stage-typed. |
| **F8.9-C2** | Causal progression is stage-typed and sealed. Public posture is read-only; no caller or sibling may supply a free digest, kind, predecessor, observation, clock, or ordinal. | Production signatures; public compiler attacks; internal authority-shape review. | **PROVED FOR C4.** A move-only runtime-minted dispatch admission owns the observation, Query runtime, ordinal, and clock. Causal construction additionally requires an unforgeable capability whose only value is private to the progression owner; public posture and identity types expose observation only. |
| **F8.9-C3** | Acknowledgement and completion observe one exact attempt; failures never impersonate completion. Every safe retry has a fresh attempt while retaining the same provider/emission predecessor and rail correlation, and the rail executes one request meaning once. | Full fault matrix; real initial-to-retry causal comparison; concurrent duplicate and meaning-drift rail twins. | **PROVED FOR C4.** Real Bank lost-response/disappear journeys compare the initial receipt dispatch with admitted redispatch as `FreshAttemptForSameEmission`. The rail atomically reserves correlation plus immutable payload meaning, rejects same-key drift, and counts independent physical consequences; sixteen concurrent duplicates produce one consequence. Fault control now travels over a separate test-only control port and is absent from the production dispatch frame. |
| **F8.9-C4** | Identity derivation, runtime attempt admission, transport orchestration, exact committed owner observation, and outcome classification have separate named responsibilities. Canonical work and structural read work are exact; no-effect/no-transport lanes remain zero. | Composition audit; exact causal/read counters; ordinary-lane twins; dirty line-cap. | **PROVED FOR C4.** Named modules own exact snapshot projection, commit-artifact binding, runtime attempt admission, identity derivation, progression, dispatch, and rail admission. Relational's authoritative commit is a short consuming progression whose stages and raw mechanisms are private to their transition owners. Durable append consumes a non-cloneable runtime/commit/branch-affine authority minted only by the typed commit or lineage owner, while finalization is private beneath publication execution. Relational's exact-commit projection returns operation-local owner work for canonical-version resolution, retained-snapshot lookup, projection view, direct record probe, projected record/fields, index examination, and reconstruction demand. Query only maps that evidence. The removed correlation index eliminates its former ordinary commit cost. C8 final-source rebinding remains open. |

### F3 ledger-completeness attack candidates

The controlling review must try to preserve each defect while making a simple
happy-path ladder pass:

1. the row is committed, but the proof reads the receipt copy rather than a
   fresh Relational owner view;
2. a fresh row is found, but it was created by another commit, branch, or
   runtime;
3. the owner read is honest, but production dispatch still sends the old
   in-memory record;
4. four identities exist, but two are equal or an observation points directly
   to provider commit instead of dispatch attempt;
5. constructors use distinct function names but still accept a generic
   identity, link, or reusable evidence token;
6. public callers can destructure a posture, recover its evidence value, and
   rebuild a later or replacement posture;
7. retry derives the same attempt identity and therefore collapses two rail
   attempts into one causal event;
8. a lost response or acknowledgement is relabelled completion by a later
   observation without owner evidence for that exact attempt; or
9. the implementation derives four digests while canonical work still reports
   one.

This list is an audit seed. F3 closure requires a final completeness attack
after implementation and mutation testing.

### Historical F3 completeness audit (2026-08-08; superseded by C4)

This section records the earlier closure decision that C4 subsequently
invalidated. It is historical evidence only and is not the governing current
status. The C4 row and the active F3 ledger above remain authoritative.

The historical audit stated **CLOSED** after all nine candidates were exercised.
Candidates 1 and 3 initially exposed a real gap: replacing the fresh read with
a receipt-built observation left the Bank happy paths green. The observation
sealer was therefore moved from the application-aftermath record module into
the provider owner module; repeating the same production mutation now fails to
compile with `E0624`. Candidate 2 is split across runtime, commit-id, version,
branch, and later-head tests. Candidates 4 and 7 fail the predecessor and
attempt-ordinal mutation probes. Candidates 5 and 6 are pinned by five public
compiler attacks over links, event fields/constructors, the owner observation,
and the outer completion wrapper. Candidate 8 is closed by the full fault
matrix plus lost-response safe-retry journeys, and candidate 9 by the exact
three/four-derivation counters and its mutation probe.

The QA loop also corrected two composition/authority weaknesses found after the
first implementation pass: a raw `u64` attempt input became a runtime-minted
opaque ordinal, and a public result enum that could wrap an earlier event as
`Completed` became an opaque dispatch-posture projection. Undo and redo remain
present and explicitly provisional; F3 makes no claim about accepting their
product semantics. That historical close is superseded; it does not close C4
or authorize final-source claims.

## Current C5 / finish-slice F4 correction ledger (2026-08-08)

This is the controlling retention and commit-composition ledger. It supersedes
the historical F4 closure record below. Every row remains open until production
ownership and mutation-sensitive evidence agree on the same current source.

| ID | Reopened claim or defect | Required closure evidence | Current result |
|---|---|---|---|
| **C5-LOCUS-1** | Declaration and installation erase entity/aspect identity to a bare field-name string; runtime discards the exact declared-read target. Same-named fields can install or retain under the wrong semantic locus. | Typed field-reference authoring; exact entity/aspect/field installed target; full-target runtime selection; bare-string compile failure; same-name other-entity and other-aspect install/runtime twins. | **PROVED.** `Schema` remains in the Rust type through locus, demand, recorded inverse, correction mechanism, and declared aftermath contract. Only the matching operation builder holds the private association authority that erases it into constructor-free portable meaning. The compiler court rejects both raw-string wrapping and an identical-name `SchemaB` contract at a `SchemaA` builder with the intended E0308. Installation consumes only portable owner-minted meaning, and exact entity/aspect/field runtime twins pass. |
| **C5-LOCUS-2** | Duplicate demands are accepted and multi-field selection may retain fields from different concrete records, producing a commit whose inverse evidence is unusable. | Duplicate-locus declaration denial; one declared entity-role rule; two-field same-record positive; cross-record no-write denial; exact-locus retained lookup. | **PROVED.** Declaration rejects duplicate loci and multiple entity roles. A real provider journey retains two fields from one concrete record; the cross-record twin denies before commit and leaves the Relational head unchanged. Selection is private to the commit owner and its load-bearing test traverses a real decision fact plus a real owner footprint. |
| **C5-CANON-1** | Declared schema canonical identity omits the retention byte bound and records only short field slots. | Canonical basis includes byte bound plus every locus axis; entity/aspect/field/bound single-axis identity and reinstallation twins. | **PROVED.** Declared and installed canonical bases contain every portable locus axis plus the byte bound. The real `meaning_matches` reinstallation door accepts the unchanged operation and rejects independently coherent entity, aspect, field, and bound drift; forcing it to return `true` fails the exact test. |
| **C5-WORK-1** | No-demand commits eagerly scan and allocate the complete Relational mutation footprint; the existing zero-work test observes unrelated digest counters. | Demand branch before candidate scan/footprint projection; owner-carried intents/targets/candidates/demands counters; no-demand breadth twins at exact zero; demanded breadth-sensitive twin; eager-construction mutation. | **PROVED.** No-demand exits before footprint projection or decision-fact scanning. Relational and Query carry exact intents, targets, facts, candidates, and loci work; breadth twins prove exact zero without demand and scale only on the demanded lane. Eager projection makes the zero-work proof fail. |
| **C5-FOOTPRINT-1** | Footprint unit tests directly construct private target vectors and bypass `ValidatedRelationalMutation::mutation_footprint`; whole-aspect/whole-record record affinity is incompletely attacked. | Real owner-validated production matrix for exact field, cross-axis/record, whole-aspect, whole-record, create-only, endpoint-only, and mixed breadth; representative Query no-write journeys. | **PROVED.** Ten Relational tests validate real transactions before calling the production footprint owner, covering exact field, cross-axis and cross-record attacks, whole aspect, whole record, create-only, endpoint-only, and mixed breadth. Query's owner-local selection and no-write journeys consume that opaque footprint rather than tuple evidence. |
| **C5-EVIDENCE-1** | Commit evidence exposes raw axis constructors plus a separate raw commit key, allowing sibling recombination of mutation work, pre-image, and outbox. Receipt assembly reopens the same axes through independent builders. | Commit-owner-private whole-`CommitReference` evidence mint; store derives its key; immutable owner observation; no independent receipt axis builders; compiler attack against cross-commit splice. | **PROVED.** The commit owner seals one whole-`CommitReference` artifact containing mutation/retention work, retained pre-image, and commit-derived outbox resolution; the store derives its key and checks the whole reference. No independent receipt-axis builder or cross-commit splice constructor remains. |
| **C5-EVIDENCE-2** | The first receipt consumes and removes evidence; every equivalent/idempotent receipt hardcodes retained pre-image and committed outbox to `None`. | Repeatable evidence observation with explicit lifetime; normal/equivalent/lost-response/interleaved journeys preserve exact mutation work, pre-image, outbox, and commit; stale-bundle mutation fails. | **PROVED; C4 OUTBOX CARRIAGE DEPENDENCY RESTORED.** Evidence observation is repeatable, and normal, equivalent, lost-response, and interleaved journeys preserve the same commit, work, pre-image, and outbox. A real `CommitResult` proves exact `Some`, honest `None`, and `CreatedEntityMissing`. Only the outbox owner can mint the private success seal; fresh and recovered receipt constructors accept the whole provider-plus-seal projection and cannot reinterpret denial as absence. |
| **C5-COMPOSE-1** | The apparent session coordinator hides commit, evidence assembly, snapshot/projection refresh, causality, index publication, failure injection, and result encoding in one mixed-level function. | Named typed commit-result/evidence/publication/result transitions; short linear coordinator; post-commit publication cannot be skipped by evidence failure; scrutiny and line-cap closure. | **PROVED.** Preparation, one Relational commit, exact evidence sealing, required publication, and provider result encoding are named typed transitions under the commit owner. Installation's full operation compiler is likewise private to `application_operation::installed`; aftermath consumes only its sealed read view. The dirty 400-line guard and final composition audit pass. |
| **C5-AGGREGATE-1** | Aggregate invariant projection is a 197-line mixed-phase coordinator spanning validation, cache, bounded scan, accounting, exclusivity, arithmetic, scope, and publication. | Private validated-plan/cache-result/completed-scan transitions; one accounting owner; cache-hit/miss, ambiguity, arithmetic overflow, and budget mutations fail focused proofs. | **PROVED.** Private validated-plan, cache-result, miss-scan, and completed-scan phases leave one accounting owner and publish only completed results. Seven focused tests cover cold/warm/stale cache, first/mid-scan budget, ambiguity, invalid scalar, arithmetic overflow, and denial retry without cache poisoning. |

### C5 final hostile close audit (2026-08-08)

Three independent frozen-tree reviews returned `CLOSE`: specification and
authority fidelity, mutation/theatre-test evidence, and complete dirty-set
composition/topology. The final scrutiny pass inspected 307 dirty Rust files,
classified 95 candidates with zero scan errors, and found no actionable C5
candidate; the six provider/session/schema-layout bags remain explicitly open
under C7. Hostile probes killed schema substitution, unconditional
reinstallation acceptance, production decision-fact rejection, eager footprint
projection, aggregate budget/cache/overflow drift, evidence first-entry or
stale-bundle selection, outbox denial-to-absence, and caller attempts to replace
the private receipt seal with `Option`.

Final source gates: declaration 148/148; installation 190/190 plus five
doctests; execution 638/638; Bank-domain all targets; ten Relational footprint
owner tests; all three compiler courts; strict Clippy on the affected Query and
Bank targets; formatting; dirty Rust line cap; boundary check; agent-context
check; and `git diff --check`. C8 still must bind the complete Phase 8 result to
one final source fingerprint. Undo and redo remain provisional and were not
accepted by C5.

### Historical finish-slice F4 closure ledger — superseded

The rows below record the earlier implementation and remain useful audit input.
Their `PROVED` and `CLOSED` labels are historical only; they do not override the
current C5 rows above.

## Finish slice F4 closure ledger

This is the controlling ledger for F8.7 and the commit-composition portion of
F8.11. Every row started **OPEN** and was closed only by the evidence below. F4
proves exact prior-truth retention as a general commit foundation; it does not
accept undo or redo product semantics.

| ID | Exact closure claim | Evidence required | Current result |
|---|---|---|---|
| **F8.7-C1** | A retained field is selected only when the exact existing record and exact observed aspect-field locator occur in the Relational-owner footprint of the invariant-validated mutation. A mutation of another field, another aspect carrying the same field name, another record, or endpoints alone cannot satisfy the demand. Whole-aspect and whole-record mutations have explicit semantics rather than collapsing to record membership. | Relational validated-footprint source trace; exact-field positive twin; right-record/wrong-field, same-name/other-aspect, other-record, and endpoint-only attacks; one-check removal mutation probe. | **PROVED.** Relational exposes only the opaque `ValidatedMutationFootprint`, derived from `ValidatedRelationalMutation`'s validated merged intents. Its four owner tests pin exact field, selected patch fields, whole-aspect/whole-record, and endpoint-only semantics. Query's eleven retention tests pin record/aspect/field affinity. Removing locator equality made the Relational exact-field test fail; accepting every candidate made the production-path right-record/wrong-field test commit incorrectly. |
| **F8.7-C2** | Retention uses only the admitted decision read-set and the owner-validated mutation candidate before commit. Empty, missing, ambiguous, over-bound, multi-segment, create-only, and no-existing-record demands fail closed without a live reread or a committed mutation. | Owner tests for every denial; Bank Courtroom G positive journey; commit-denial no-write assertions; residue scan for live reread/fallback. | **PROVED.** Production retention accepts the concrete Relational footprint and candidates carried from `attempt.facts`; it has no live-reader or fallback input. The eleven Query attacks cover the named denials. `right_record_wrong_field_retention_denial_commits_nothing` drives authorize/project/effect/commit and proves the Relational head is unchanged. Bank's `recorded_inverse_undo_restores_prior_status_from_retained_preimage` is the exact-prior-truth positive twin only; it does not accept undo semantics. |
| **F8.7-C3** | Mutation work and retained prior truth are handed from the provider to receipt assembly by exact Relational commit identity. Singleton “last completed” slots, stale evidence after a lost response, and cross-commit substitution are absent. | Exact-commit evidence map or phase value; lost-response/interleaved-commit hostility; wrong-commit substitution test; removal mutation probe. | **PROVED.** `WorthQueryCompletedCommitEvidenceStore` is keyed by exact Relational `CommitId` and moves mutation work plus optional pre-image as one affine bundle. Wrong-commit and paired-bundle unit tests, concurrent independent commits, response-loss resolution, and the terminal provider-resource baseline pass. Replacing exact lookup with first-entry removal made the wrong-commit test fail. |
| **F8.11-C1** | Session extraction, pre-image preparation, Relational commit, post-commit publication/index work, exact-commit evidence recording, and provider-receipt encoding have separate named responsibilities. The lifecycle orchestrator is linear and small. | Destination source trace; composition advisories as fatal; dirty line-cap; no catch-all module or hidden parallel owner. | **PROVED AFTER F6A.** `provider/session_commit/` retains the Relational commit decomposition. F6A additionally gives the outer application commit and provider progression distinct move-only `commit_preparation`, `managed_commit_run`, `provider_session_admission`, `application_attempt_registration`, `invariant_progression`, `authorized_progression`, and `commit_completion` phases. The entry and progression functions are now same-level coordinators; decision-fact binding failure explicitly aborts its staged session. The exact 29-file fatal composition audit reports zero candidates/errors and the dirty Rust line cap passes. |
| **F8.11-C2** | Test failure injection remains owner-private fixture capability and cannot alter supported production authority. Ordinary no-aftermath/no-retention commits retain zero Phase-8 work and do not inherit prior retained evidence. | Facade/feature residue; public compile boundary; Courtroom J ordinary counter and sequential contamination twins; all-feature verification. | **PROVED.** The test footprint helper and mutation-work observer are `cfg(test)`-only and do not feed production receipt assembly. The exact bundle test proves an ordinary `None` pre-image cannot inherit a prior bundle. Bank Courtroom J reports zero aftermath slots while machinery is live; Query all-features check, no-deps production clippy, full library tests, boundary check, and agent-context check pass. |

### F4 ledger-completeness attack candidates

1. the operation mutates the right record but a different field;
2. the observed and changed fields share a short name but belong to different
   aspects or paths;
3. a replace/delete/whole-aspect operation is treated as either no fields or
   every field without an explicit rule;
4. retention fails, but the Relational mutation still commits;
5. exact prior truth is read live after admission rather than carried from the
   decision read-set;
6. a lost response leaves singleton completed evidence that a later receipt
   consumes;
7. mutation work is exact but retained pre-image comes from another commit, or
   vice versa;
8. the large commit function is split cosmetically while one helper still owns
   several semantic phases; or
9. ordinary commits inherit stale retention or pay reconstructive work.

This list is an audit seed. F4 remains open until a final completeness attack
has exercised every candidate.

### F4 final completeness audit (2026-08-08)

**CLOSED.** All nine candidates were exercised against the final authority
path. Candidates 1-3 are pinned by the opaque Relational footprint owner tests,
the Query exact-locator attacks, and the two locator-removal mutations.
Candidate 4 is the production-path head-preservation test. Candidate 5 is
closed structurally: production retention has only the admitted fact candidates
and concrete owner footprint as inputs. Candidates 6 and 7 are pinned by the
commit-keyed affine bundle, wrong-commit/interleaved tests, response-loss tests,
provider-resource baseline, and first-entry-removal mutation. Candidate 8 is
closed by the named session-commit decomposition and fatal composition audit.
Candidate 9 is closed by the ordinary `None` bundle twin and Bank Courtroom J.

Final verification from the closing tree:

| Check | Result |
|---|---|
| Relational validated-footprint owner tests | 4 passed |
| `cargo test -p worth-query-execution --lib -- --test-threads=8` | 609 passed |
| Focused production-path wrong-field/no-commit proof | passed |
| Bank `ordinary_mutations` | 91 passed |
| Bank exact-prior-truth and ordinary-zero-work twins | passed |
| `cargo check -p worth-relational` | passed |
| `cargo check -p worth-query-execution --all-features` | passed |
| `cargo clippy -p worth-query-execution --lib --no-deps -- -D warnings` | passed |
| Explicit F4 composition advisories (`--advisories-fatal`) | passed |
| Dirty Rust line cap | passed |
| `boundary-check` / `agent-context check` | passed / passed |

Dependency-wide clippy remains blocked by pre-existing untouched debt in
`worth-signal` and `worth-relational`; the scoped Query production target is
clean. Undo and redo remain present and **PROVISIONAL**. F4 proves retained
commit input and exact handoff, not the product correctness of either journey.
**F4 is closed; F5 is admissible.**

## Finish slice F5 closure ledger

This is the controlling ledger for F8.10 and the publication/facade remainder
of F8.11. Every row starts **OPEN**. The boundary review found two concrete
bypasses that a cosmetic `application_aftermath` wrapper would not fix:

- `WorthQueryApplicationCommitPublicationReceipt` publicly dereferences to and
  returns the execution-owned terminal, so a consumer can leave publication and
  inspect raw execution aftermath values; and
- execution owns `recovery_publication` and constructs
  `WorthQueryRecoverySupportProjection`, while the Bank noninterference test
  assembles “published” posture from installation, execution, and provisional
  undo values itself.

The accepted source boundary is therefore exact: commit publication may consume
the sealed `WorthQueryApplicationCommitReceipt`; recovery publication may
consume the disclosure-admitted `WorthQueryRecoveryInspectionView`. Publication
may derive only closed descriptive projections. It may not accept a raw commit
identity, branch, recovery slot, wire identity, installed posture, dispatch
posture, causal event, caller-selected next action, or provisional undo/redo
value.

| ID | Exact closure claim | Evidence required | Current result |
|---|---|---|---|
| **F8.10-C1** | `worth-query-publication/application_aftermath` is the sole owner of consumer-facing committed-aftermath, recovery-support, and external-effect projections. Execution produces sealed source outcomes but contains no module or function named as publication. | Destination source trace; execution/publication residue; dependency boundary check. | **PROVED.** Publication owns five named aftermath files; `recovery_publication`, `publish_recovery_support_projection`, and `WorthQueryRecoverySupportProjection` are absent from execution; boundary-check passes. |
| **F8.10-C2** | A committed-aftermath publication can be derived only from one sealed execution commit receipt carrying its exact admitted installed aftermath and external observation. A recovery-support publication can be derived only from one disclosure-admitted inspection view. Raw identities, slots, postures, receipts assembled from fields, and copied enums cannot construct either result. | Private-field/compiler attacks for raw commit identity, recovery wire identity, copied posture, caller-selected external completion, and mismatched installed contract; healthy public-consumer twins. | **PROVED.** `publish_application_aftermath(&WorthQueryApplicationCommitReceipt)` and `publish_recovery_support(&WorthQueryRecoveryInspectionView)` are the only constructors; private fields and 13 compile-fail doctests reject raw/copy/forged inputs. |
| **F8.10-C3** | Publication exposes a closed accepted posture, explicit recovery durability/support, and external-effect observation category without exposing execution handles, causal identities, outbox bytes, protected facts, or provisional undo/redo next actions. Silence or acknowledgement cannot publish completion. | Public API inventory; acknowledgement/lost-response/completion twins; protected-fact paired-world noninterference; no provisional vocabulary on accepted facade. | **PROVED.** Closed publication enums expose descriptive posture only; the six-case real-rail matrix distinguishes completion, acknowledgement, timeout, duplicate acknowledgement, lost response, and disconnect; paired protected-fact worlds publish equal complete aftermath values. |
| **F8.10-C4** | Publication is a one-way boundary. Its public receipts neither `Deref` to nor return execution terminals. Bank's public commit surface consumes publication-owned aftermath values and does not return execution-owned aftermath/dispatch types. Internal Bank recovery may retain its sealed execution receipt without making it public. | Facade and method-signature residue; public consumer test; compile failures for `terminal`, dereference, and raw execution aftermath substitution. | **PROVED.** Receipt has no public `Deref` or `terminal`; the terminal-access mutation makes the compile-fail suite fail. Bank returns publication-owned aftermath/external-effect values and retains execution only in private fields and crate-private recovery access. |
| **F8.11-C3** | Outcome projection, recovery projection, external-effect projection, disclosure shaping, generic commit inspection, and Bank adaptation have named responsibilities under the file/function advisories. No compatibility alias or catch-all facade preserves the old authority lane. | Fatal composition audit, dirty line cap, no alias/residue scan, focused owner tests. | **PROVED.** Explicit F5 composition audit scrutinized 21 Rust files with zero candidates/errors; dirty 400-line guard passes; provisional undo/redo moved to the honestly named `provisional_aftermath` facade. |
| **F8.11-C4** | Publication changes no execution legality, history, branch affinity, canonical work, or ordinary-lane cost. Relational remains the sole history/publication authority in its domain; Query publication describes only Query application aftermath. | Before/after commit identity and branch twins; ordinary zero-work courtroom; boundary check; all-feature and Bank verification. | **PROVED.** Publication only borrows sealed sources and performs closed projection. It defines no branch, parent, history, or lineage type; inherited generic branch inspection still reads the execution receipt. Ordinary zero-work, noninterference, 609 Query tests, 91 Bank tests, all-feature check, clippy, and boundary enforcement pass. |

### F5 ledger-completeness attack candidates

1. a raw commit id plus installed posture constructs the same public result;
2. a publication receipt returns or dereferences to its execution terminal;
3. a recovery wire identity or copied support enum constructs recovery support;
4. an acknowledged, missing, or lost response is published as completed;
5. protected decision facts appear in explanation, next-action, debug, or
   external-effect output;
6. Bank still returns an execution-owned aftermath or dispatch type publicly;
7. accepted publication exposes provisional undo/redo availability or causality;
8. publication invents a branch, parent, history, or lineage identity;
9. an ordinary commit pays new execution canonical work or inherits stale
   aftermath output; or
10. the new module delegates to the old execution publication helper and merely
    renames the bypass.

### F5 final completeness attack (2026-08-08)

Every seed was exercised against the final surface:

1. raw commit identity/posture construction and field assembly fail because all
   accepted result fields and constructors are private;
2. receipt terminal access and dereference fail to compile; temporarily adding
   `terminal()` makes its compile-fail contract turn red;
3. raw recovery wire identity and copied support/posture fail to compile;
4. the real rail proves acknowledgement and every unresolved fault are not
   completion; temporarily mapping unresolved to completed turns the courtroom
   red;
5. protected-fact twins differ at the source, then compare the entire closed
   aftermath publication (including `Debug`-reachable state) equal;
6. Bank's public receipt signatures contain no execution-owned aftermath or
   dispatch return type;
7. accepted `primary_graph` and publication contain no undo/redo contract;
   compiled experiments are isolated under `provisional_aftermath`;
8. publication declares no branch, parent, history, or lineage identity and
   does not decide Relational history; generic branch inspection is inherited
   verbatim from the sealed execution receipt;
9. ordinary-commit zero-work and unrelated-graph twins remain green; and
10. execution publication names/helpers and compatibility aliases are absent.

**F5 is closed; F6 is admissible.** Undo and redo remain present and explicitly
**PROVISIONAL**. This closure proves the accepted publication boundary, not the
product semantics of either correction journey.

## Finish slice F6 closure ledger (2026-08-08)

This is the controlling closure record for F8.12 and for the final status of
the accepted Phase 8 foundation. It does not promote provisional undo/redo
evidence.

| ID | Exact closure claim | Evidence required | Final result |
|---|---|---|---|
| **F8.12-DOC** | The finish plan, Phase 8 specification, Milestone 9.16, Query roadmap, worth-proof hardening plan, feature guide, and this ledger agree on accepted, provisional, and deferred scope. | Current-status audit; relative-link audit; explicit successor handoff. | **PROVED.** The accepted feature is named Application Aftermath, External Effects, And Recovery and closed on 2026-08-08. Undo/redo is a compiled experiment awaiting a future separately governed product decision. The four feature-guide relative links resolve. |
| **F8.12-AUTH** | No closure document or accepted facade moves canonical history/lineage authority from Relational to Query, Runtime Bridge, Foundational, or worth-proof DX. | Facade and production residue scans; governing-document audit. | **PROVED.** Query has no local lineage store/head; Phase 8 aftermath legality/publication imports no Runtime Bridge authority; the accepted `primary_graph` facade exports no undo/redo causality. Relational remains the canonical commit-history, parentage, branch-head, ancestry, and entity-lineage owner. Runtime Bridge may only map or transport already-admitted descriptions. |
| **F8.12-FIX** | Compiler evidence fails for the intended authority restriction after provisional facade containment. | Run the complete aftermath trybuild suite from the provisional import path; inspect snapshots. | **PROVED AFTER F6A.** The earlier Query run exposed five fixtures still importing provisional types from accepted `primary_graph`, so they failed for the wrong reason. Those fixtures now import `provisional_aftermath`. F6A adds a positive provisional-facade import and an E0432 negative twin for the accepted `primary_graph` facade; all 35 aftermath compiler cases pass. Both Bank redo examples import their real provisional types, are pinned to E0061, and pass in the 11-case rustdoc suite. |
| **F8.12-DX** | Public feature guidance names actual stable entry points and does not promise provisional correction semantics. | API/source comparison; feature-document review. | **PROVED AFTER CORRECTION.** The new feature guide documents declaration, co-commit, rail observation, recovery, exact pre-image retention, and closed publication. A stale `operation_external_effect` comment referring to the removed duplicate aftermath external posture was corrected; undo/redo remains explicitly outside the stable feature. |
| **F8.12-COMP** | F6 changes respect the file/function composition laws and the 400-line hard cap. | Dirty line-cap check; dirty advisory scan; fatal focused scan over F6 Rust files. | **PROVED AFTER F6A.** The complete dirty advisory scan scrutinizes 630 Rust files, reports 168 advisory candidates and zero scan errors; the accepted F6A commit path contributes no candidate. The one F6A-adjacent hit is the explicitly provisional Query redo admission and is deferred with that product lane. The exact 29-file F6A scan passes `--fail-on-candidates` with zero candidates/errors, and the dirty 400-line guard passes. |
| **F8.12-VERIFY** | The complete standing verification set passes from the final tree. | Both full workspaces plus root proof and constitutional commands. | **PROVED AFTER F6A.** Exact current-tree commands and outcomes are recorded below. |

### F6 completeness attack

The final audit deliberately tested whether closure could survive any of these
dishonest states:

1. provisional compile-fail cases fail only because their import disappeared;
2. documentation calls undo/redo stable because its historical tests are green;
3. publication exposes a raw execution terminal or reconstructable authority;
4. Query or Runtime Bridge silently becomes a second history/lineage owner;
5. hostile payload rejection, lost response, or acknowledgement is presented as
   external completion;
6. exact pre-image retention is claimed without exact record/aspect/field proof;
7. a broad green suite hides all-target lint or focused compiler failures;
8. a feature guide points consumers at internals or promises unsupported public
   durability;
9. source occurrence is presented as accepted-facade reachability without a
   compiler import proof;
10. retry code accepts or compares invented ordinals without proving that both
    production dispatch paths consume the runtime's fresh-attempt operation; or
11. named helpers cosmetically shorten the files while one coordinator still
    owns multiple proof or lifecycle phases.

Cases 1, 7, and 9-11 found real evidence or composition defects and were
corrected before closure. Cases 2-6 and 8 are denied by the
accepted/provisional split, sealed publication, real rail, exact-retention
courtroom, residue probes, and the feature guide.

### F6 standing verification

- `workspaces/worth-query`: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and
  `cargo test --workspace --all-features` pass. The full test run includes the
  corrected 35-case aftermath compiler suite.
- `workspaces/worth-query-bank-world`: the same formatting, clippy, and full
  workspace test commands pass, including the real external rail, browser
  identity boundary, publication/noninterference proof, and all 91 ordinary
  mutation tests.
- Repository root: `cargo test --manifest-path crates/worth-proof/Cargo.toml`
  passes, including its authority compile-fail suites.
- Repository root: `cargo run --manifest-path
  tools/boundary-check/Cargo.toml -- --root .` and `cargo run --manifest-path
  tools/agent-context/Cargo.toml -- check` pass after the governed facade
  snapshots and generated host context were refreshed.
- Repository root on Windows: `C:\Program Files\Git\bin\bash.exe
  scripts/ci/check_workspace_rust_line_caps.sh dirty` passes.
- The accepted-facade, retired-publication, Query-history, and Runtime Bridge
  authority residue probes pass; the feature guide's four relative links all
  resolve.

**F6 is closed after F6A. Runtime Phase 8's accepted foundation is finished.**
Undo and redo remain present only as a provisional experiment and receive no
support or architecture promise from this closure.

## Reopening (2026-08-06)

### What the `PROVED` column actually meant

Before this section existed, this file recorded seventy `PROVED` rows and the
sentence *"Phase 8 closes when every `R8.*` row above reads `PROVED`."* A reader
would take that as seventy verified guarantees. It was not.

The column reflected **per-gate self-reports by the implementing agent, plus one
deep independent audit of Gate 8.7 alone.** Gates 8.1–8.6 were confirmed by
re-running their named test targets and reading their evidence claims — not by
adversarially reading the production authority paths. The final "70 PROVED, zero
open" statement was produced by counting rows in this document.

That is **Q8.11's own defect** — a row proved on evidence belonging to something
else — committed by the auditor, at the scale of the whole phase, in the file
that names Q8.11.

### The ten findings

All confirmed by direct reading of production code. This is **one defect
instantiated ten times: the caller supplies what the runtime must derive or
prove.** Every authored test passes because it calls each API in the intended
order with the intended values; nothing in the types requires that.

**Current interpretation:** the rows below preserve the correction audit at the
time each slice reported. They are not the finish backlog. The governing closed,
accepted, and provisional inventory is F8.1-F8.12 in the
[finish plan](./milestone-9.16-runtime-phase-8-finish-plan.md). In particular,
Q8.21-Q8.24 contain useful implementation evidence but do not accept the undo
or redo product design. F1-F5 close the accepted defects formerly recorded by
Q8.25-Q8.27 and the exact-field remainder of Q8.26.

| ID | Finding | Reopens |
|---|---|---|
| **Q8.18** | **CLOSED by correction slice 1.** Aftermath is one compiled field of `WorthQueryCompiledApplicationOperationContracts`; installation derives its identity and pre-image coverage from the exact installed operation and declared `graph_reads`. The retired free host-authored installation lane can no longer supply raw digests, an operation string, or parallel read coverage. | R8.16–R8.19 proved. |
| **Q8.19** | **CLOSED by correction slices 3A–3B.** The exact compiled aftermath travels privately from operation admission through the commit receipt into `WorthQueryRecoveryHandleBinding`. Mint is receipt-only. Every recovery, undo, and redo admission/transition now reads historical target facts and the installed contract from that binding; no public signature accepts a recovery target, aftermath contract, or reconstructed current observation. Only the fresh admitted operation remains caller-presented, and its live semantic axes include exact installed-aftermath identity. | R8.28 and R8.30 proved; R8.31 currentness subclaim proved. |
| **Q8.20** | **CLOSED by correction slices 2 and 3B.** Effect and inspect authority each carry proof bases for both the concrete runtime owner and the exact recovery-handle identity. Colliding registry ordinals across runtimes and two distinct committed-receipt handles inside one runtime are non-substitutable; proof-use mismatch remains `FreshAuthorityDenied`, distinct from admission-time `ForeignRuntime`. | R8.31 proved. |
| **Q8.21** | **RECOVERY FOUNDATION ACCEPTED; F8.1 CLOSED; UNDO/REDO SUBCLAIMS PROVISIONAL.** `LinearResource` owns the one-handle terminal law. Because a commit receipt is cloneable historical evidence rather than a linear permit, the runtime registry atomically claims its authoritative `(provider runtime, typed branch, commit)` identity before minting; cloned and concurrent second claims deny as `RecoveryAlreadyMinted`. F1 removed the supported `test-support` feature and every public registry/slot/terminal lane. The move-only undo/redo handoff evidence remains useful regression evidence but does not accept that product design. | R8.29 and R8.29-C1 proved for the recovery foundation; undo/redo ownership claims remain provisional. |
| **Q8.22** | **PROVISIONAL IMPLEMENTATION EVIDENCE.** Commit authority privately carries the exact original governed input and its canonical identity into the receipt and handle, and the Bank implementation derives correction inputs rather than accepting replacement values. This is valuable authority work, but the undo product and its exact target semantics are not accepted by the Phase 8 finish. | R8.36–R8.39 await Milestone 9.18. |
| **Q8.23** | **PROVISIONAL IMPLEMENTATION EVIDENCE.** `WorthQueryProvedUndo` is owner-minted rather than a public raw-parts bag, and redo does not accept replacement action or idempotency fields. `Inverts<Original, Authority>` relates action kinds, however, not exact committed occurrences, and the redo product is not accepted by the Phase 8 finish. | R8.42 and R8.43 await Milestone 9.18. |
| **Q8.24** | **HISTORY OWNERSHIP ACCEPTED; UNDO/REDO SEMANTICS PROVISIONAL.** Relational is again the only owner of commit identity, ordered parents, branch head, ancestry, serialization, and publication. The Query chain, mutable head, raw nodes, and append/record APIs are deleted, and Runtime Bridge receives no legality or current-head authority. The current `undo-of` / `redo-of` fact and expected-head policy remain provisional until their owning product milestone. | R8.12 authority split is retained; R8.44–R8.46 require later semantic acceptance/publication. |
| **Q8.25** | **CLOSED by F2-F3.** The declaration owns a stable wire identifier; the co-committed row is reread through provider-sealed owner observation; the separate rail rejects all four hostile semantic cases; provider commit, emission, dispatch attempt, and external observation carry distinct predecessor-linked identities. Rust `type_name` remains diagnostic only. | Accepted R8.22–R8.23 external-effect foundation proved by the F2/F3 ledgers. |
| **Q8.26** | **RETENTION FOUNDATION CLOSED by F4; UNDO USE PROVISIONAL.** Relational now derives an opaque exact mutation footprint from the validated merged plan. Retention requires the exact demanded record/aspect/field, consumes admitted prior facts, and fails closed before commit. This accepts retained-truth infrastructure, not undo semantics. | R8.2 retention foundation accepted; R8.40–R8.41 remain provisional successor evidence. |
| **Q8.27** | **CLOSED by F5.** `worth-query-publication/application_aftermath` owns committed-aftermath, external-effect, and recovery-support projections. Its inputs are sealed commit receipts or disclosure-admitted inspection views; execution publication helpers and raw Bank assembly are removed. | Accepted publication and noninterference proved by the F5 ledger; undo/redo next actions excluded. |

Also found and already fixed: `cargo fmt --all --check` was failing across **387
files** in both workspaces. It had never been run in this phase, and was absent
from the standing verification set entirely.

### Reopened requirements

At correction-plan entry, `R8.16`–`R8.19`, `R8.22`–`R8.23`, `R8.28`–`R8.31`,
`R8.36`–`R8.48`, and their exit proofs reverted to **OPEN**. The slice closure
rows below are now the controlling status; they close only the exact subclaims
their evidence proves.

### What is genuinely delivered and must be preserved

Not a rebuild. The following foundations should survive the correction: the
declaration taxonomy and local installation validation; typed fault
classification that never guesses completion; a real separate-process TCP rail
with a transport-fault matrix; the co-committed outbox structure and
mutation-free effect coverage; runtime-clock expiry and handle cleanup; fresh
current authorization before Bank correction operations; Gate 8.7's safe-retry
re-dispatch; and file decomposition that passes the line cap. The finish plan
states the remaining authority, semantic-boundary, evidence, and composition
work required before these foundations become accepted closure.

## Gate status

| Gate | Scope | Status |
|---|---|---|
| 8.1 | Installed aftermath classification and legal next actions | **CLOSED after correction slice 1** — Q8.18 closed |
| 8.2 | External-effect causality and indeterminate posture | **ACCEPTED FOUNDATION CLOSED** — Q8.25 closed by F2-F3 |
| 8.3 | Recovery handle and resolution lifecycle | **F8.1 CLOSED FOR THE RECOVERY FOUNDATION** — the supported-feature authority lane is absent under default and all supported features; undo/redo-specific lifecycle claims remain provisional |
| 8.4 | Fresh undo, inverse operations, and compensation | **PROVISIONAL** — F4 closes exact retained-truth infrastructure; undo product semantics remain unaccepted |
| 8.5 | Fresh redo intent over Relational-owned lineage | **PROVISIONAL** — Relational history ownership is accepted; the redo product, occurrence semantics, and current-head policy await Milestone 9.18 |
| 8.6 | Bank aftermath cutover, publication, and certification | **ACCEPTED FOUNDATION CLOSED** — Q8.27 closed by F5; undo/redo portions remain provisional |
| 8.7 | Safe-retry re-dispatch (append-only corrective to 8.3 / 8.2) | **CLOSED** (turn 2) — slice 3B removed caller-selected recovery facts and made its effect authority exact-handle affine; slices 4–5 closed the shared handle lifecycle. |

### Correction slice 2 closure ledger

| ID | Exact closure claim | Evidence required | Status and evidence |
|---|---|---|---|
| Q8.20-C1 | An external consumer cannot construct effect or inspect recovery authority from serialized, foundational, or raw runtime material. | Public-facade compile-fail proof that fails on private fields and private minting for both types. | **PROVED** — `recovery_authority_constructor_is_private`; the aftermath trybuild harness passes. |
| Q8.20-C2 | Effect and inspect authority are each affine to the concrete runtime that minted them; equal registry slots in two runtimes grant no substitutability. | For each authority kind, an owner-runtime positive twin and a foreign-runtime negative twin with deliberately colliding slot 1; exact `FreshAuthorityDenied` cause and leak-free teardown. | **PROVED** — `authority_tests` covers effect and inspect independently and both registry owners report no live handles after teardown. |
| Q8.20-C3 | Runtime authority identities never wrap or reuse an already-issued ordinal. Exhaustion fails closed before another authority owner exists. | Allocator boundary test at `u64::MAX - 1`, proving the last ordinal is issued once, the next mint is refused, and the counter remains terminal. | **PROVED** — `runtime_authority_identity_exhaustion_cannot_wrap_or_reuse`. |
| Q8.20-C4 | Admission and proof-use denials preserve their different authority meanings. A foreign handle or operation admission is `ForeignRuntime`; a foreign effect/inspect proof presented to an otherwise live handle is `FreshAuthorityDenied`. | Direct production-path inspection plus typed denial tests at the binding/admission and proof-use boundaries. | **PROVED** — `authority_tests::admission_time_foreignness_and_proof_use_foreignness_are_different_causes` asserts both causes and their inequality in one test, plus both Q8.20 authority twins. |
| Q8.20-C6 | Recovery authority is fresh at *use*, not merely at mint. An effect or inspect authority obtained before a handle's deadline cannot open a transition after it. | For each authority kind, mint the authority the way the runtime does, then present it against a handle the runtime's own clock reports as expired; exact `Expired` cause. | **PROVED** — `authority_tests::{effect,inspect}_authority_minted_before_expiry_denies_after_the_deadline`; both fail if the `deny_if_expired` call is removed from `ensure_for`. |

Ledger attack against C4 and C5 (audit of slice 2, turn 3): C4 previously cited
`binding_axis::foreign_runtime_drift_denies_distinctly`, a test slice 3B deleted
along with the runtime-instance axis. Nothing else asserted the recovery
`ForeignRuntime` cause, so half of C4's claim — the admission-time half — had no
evidence at all while the row read PROVED. C5 is new: `ensure_for` checked
liveness and owner identity but never re-read the deadline, and `deny_if_expired`
ran only inside `admit_recovery_*_authority`. Because the authority object is
reusable and holdable, every transition on an expired handle stayed open
indefinitely to whoever obtained authority before the deadline — the freshness
in `FreshnessScopedBasis<CurrentValidity, _>` was a mint-time stamp, not a
standing property. The registry now carries the owning runtime's clock, so
`ensure_for` re-samples it without any transition gaining a clock parameter.

Ledger attack: without C3, every earlier row could pass while the process-local
allocator eventually reused an owner identity after `u64` exhaustion. The
allocator now uses checked atomic progression and becomes terminal on
exhaustion. Public construction, cross-runtime slot collision, owner-identity
reuse, and denial conflation are closed. The ledger attack performed while
planning slice 3 found a different surviving program: authority admitted for
one handle remains usable with another handle in the same runtime. The earlier
"no remaining Q8.20 defect" conclusion was therefore false; exact-handle
affinity is an explicit slice-3B obligation beside Q8.19.

### Correction slices 3A–3B closure ledger

Rows C1-C4 establish authoritative carriage. C5 and Q8.20-C5 are required for
closure because carriage alone does not prevent transition-time substitution
or same-runtime cross-handle proof reuse.

| ID | Exact closure claim | Evidence required | Status and evidence |
|---|---|---|---|
| Q8.19-C1 | A commit receipt privately retains the installed aftermath compiled for the exact operation admission that produced it. No host or transition caller supplies that contract during receipt construction. | Production construction trace from operation admission through all new/recovered receipt paths; no public constructor. | **PROVED** — `WorthQueryApplicationCommitAuthorityBinding::from_admission` clones `admission.allowed_graph_contract().aftermath()` and every receipt construction consumes that private binding. |
| Q8.19-C2 | Recovery-handle mint accepts the committed receipt only. Supplying a second aftermath contract is unrepresentable to public consumers. | Negative public compile case plus a valid receipt-only positive twin and production Bank consumer. | **PROVED** — `recovery_mint_rejects_caller_aftermath`, `recovery_mint_uses_receipt_only`, and `BankIdentityRuntime::open_commit_recovery(receipt)`. |
| Q8.19-C3 | The handle retains the exact installed aftermath identity and operation slot, plus immutable mutation-work and retained-preimage target evidence, directly from the receipt. | Direct production source trace and a real committed Bank handle observed against the independently installed operation contract. | **PROVED** — `WorthQueryRecoveryHandleBinding::from_receipt`; Bank `mint_and_inspect_leave_recovery_inspection_at_zero` asserts identity and slot through the public binding. |
| Q8.19-C4 | Carriage adds no lookup, recompilation, basis preparation, digest derivation, or digest-text materialization to mint or inspection. | Source trace excluding lookup/recompile plus exact public counter assertions. | **PROVED** — mint clones already-compiled evidence; the Bank counter twin remains exactly 0/0/0. |
| Q8.19-C5 | No transition accepts a caller-supplied recovery target, installed aftermath contract, or reconstructed current-observation object. | Public signature inventory and compile-fail substitution attempts for every affected transition. | **PROVED** — all recovery, undo, and redo signatures read historical facts from the handle; `recovery_transition_rejects_caller_aftermath` proves the removed contract lane cannot compile, while the Bank mechanism twins exercise the receipt-derived contract. |
| Q8.20-C5 | Effect and inspect authority admitted for one handle cannot authorize a different handle in the same runtime. | Independent effect and inspect same-runtime/two-handle adversarial twins with exact denial and leak-free teardown. | **PROVED** — execution owner tests and Bank `phase8_exact_handle_authority` independently deny both effect and inspect substitution with `FreshAuthorityDenied`; exact-handle twins succeed and both registries tear down empty. |

Ledger attack result: the previously surviving programs are now closed at the
signature and proof boundaries. Caller-supplied aftermath is unrepresentable,
and authority for handle A cannot authorize handle B even when both handles
belong to one runtime and the same installed semantic family. Slice 5 then
separately closes receipt-level mint dedup and admission ownership; exact-handle
authority was necessary but was never treated as sufficient evidence for it.

### Correction slice 4 closure ledger

| ID | Exact closure claim | Evidence required | Status and evidence |
|---|---|---|---|
| Q8.21-L1 | One live handle reaches one terminal through the existing `worth-proof` law; no `live: bool` or registry lookup acts as a second linearity authority. | Direct owner trace, duplicate-use compile denial, all terminal paths, and residue scan. | **PROVED** — `WorthQueryRecoveryHandle` privately owns `LinearResource<WorthQueryRecoveryHandleIdentity, WorthQueryRecoveryResourceTerminal, _>`; consuming transitions take the handle by value; no aftermath `live: bool` remains. The runtime registry remains only for enumeration, force termination, terminal audit, and `Drop`. |
| Q8.21-L2 | Effect and inspect authority assert current validity through an owner-sealed proof carrier, not a public struct field or generic marker bound. | Private-construction compile proof plus exact runtime/handle substitution tests. | **PROVED** — both public authority types contain an owner-minted `Artifact` with `FreshnessScopedBasis<CurrentValidity, AssumptionBasis<runtime+handle>>`; constructor compile denial and the slice-3B cross-owner/cross-handle twins remain green. |
| Q8.21-L3 | Runtime time remains the only expiry source, but current evidence cannot satisfy the expired continuation. Neither evidence type is caller-constructible. | Compile-fail current→expire, private-constructor denial, current and expired real-clock twins. | **PROVED** — `WorthQueryRecoveryExpiryEvaluation::{Current, Expired}` carries distinct scoped freshness types; `expire_recovery_handle` accepts only `WorthQueryRecoveryExpiryDecision`; both compile cases and Bank clock twins pass. |
| Q8.21-L4 | Expired evidence is affine to the exact handle evaluated. | Same-runtime two-handle hostile twin with one expired decision, exact denial, positive owner twin, and leak-free teardown. | **PROVED** — Bank `expired_evidence_for_handle_a_cannot_expire_handle_b_in_the_same_runtime` denies B with `FreshAuthorityDenied`, expires A, and leaves the registry empty. |
| Q8.21-L5 | The opaque wire projection is structurally authority-weakened, not merely rejected by a denial helper. | Direct type trace plus public readmission denial. | **PROVED** — projection bridges an owner-sealed current artifact into `BoundaryBridgedAuthorityRevalidationRequiredBasis`; no reverse/readmission constructor exists and the public typed denial remains green. |
| Q8.21-L6 | Concurrent force termination cannot race into a falsely successful consuming transition. | Owner-level forced-terminal-before-consume attack preserving the winning terminal and exact denial. | **PROVED** — registry terminalization is atomic; a lost terminal claim consumes the local linear resource as the recorded fate and returns `AlreadyTerminal` without rewriting `ForceTerminated`. |

Ledger attack result: slice 4 proves one-handle terminal progression and typed
freshness honestly, but it cannot close Q8.21. A cloned receipt can still mint
two separate, individually lawful `LinearResource` values, and borrowed
undo/redo admission can still be repeated. Slice 5 must remove both programs.

### Correction slice 5 closure ledger

| ID | Exact closure claim | Evidence required | Status and evidence |
|---|---|---|---|
| Q8.21-L7 | A cloneable or concurrently presented receipt cannot mint a second handle *while one is live*, and cannot mint a second handle *after the commit's one recovery has been exercised*. Receipt description is not mistaken for a compile-time mint permit. | Production cloned-receipt denial, atomic owner-table race, exact typed cause, one surviving live slot, and a post-terminal re-mint denial. | **PROVED (restated at slice-5 re-audit)** — `register_once` atomically claims `(provider runtime, BranchId, CommitId)`. The original wording said the claim "is never released"; that is now false and was always the wrong invariant. `mark_terminal` retains the claim, so any *exercised* recovery is spent forever; only `relinquish` returns it (see Q8.21-L11). Bank `cloned_receipt_cannot_mint_a_second_recovery_handle` denies a live-handle duplicate with `RecoveryAlreadyMinted`; `a_completed_transition_spends_the_commits_one_recovery_permanently` denies the same way *after* a successful reconcile, which is the case that pins the retained half; `concurrent_receipt_claims_register_exactly_one_handle` proves exactly one race winner and leak-free teardown. |
| Q8.21-L8 | Undo admission takes ownership of the handle; a caller cannot retain a borrow and admit undo twice. | Public signature trace, compile-fail moved-value proof, denial/drop and committed positive paths. | **PROVED (evidence restated at slice-5 re-audit)** — `admit_undo(handle, authority)` moves the handle into private `WorthQueryUndoAdmission`, which moves through `WorthQueryUndoProgressionHandoff`; `undo_handle_reuse_is_unrepresentable` fails on the second use. Denied one-shot and lawful money/recorded-inverse Bank paths remain no-write or commit exactly once. The re-audit found this row's "denied one-shot" evidence spoke only about *database writes*; it never claimed anything about whether the capability survived a denial, which is why the defect below hid under a fully proved row. That question is now Q8.21-L11's, and the move-based linearity this row asserts is unchanged: `admit`/`admit_deriving` still take `self`. |
| Q8.21-L9 | Only the framework can pair a proved undo with the still-linear handle after ordinary undo commit. A caller cannot combine a proof with an unrelated handle. | Private-constructor compile denial and production committed-undo construction trace. | **PROVED** — `WorthQueryRedoRecovery` has private proof/handle fields and is sealed only from the unforgeable progression handoff plus committed receipt; `redo_recovery_constructor_is_private` rejects caller recombination. The value grants no current authority and still requires fresh effect admission. |
| Q8.21-L10 | Redo admission consumes the sealed continuation and ordinary redo progression terminalizes its handle; neither admission nor continuation is reusable. | Compile-fail moved-value proof, terminal registry observation, lawful redo, and all denial twins. | **PROVED (evidence restated at slice-5 re-audit)** — `admit_redo(recovery, authority, intent)` moves `WorthQueryRedoRecovery` into a private admission; `progress_admitted_redo` consumes the handle as `Consumed`; `redo_recovery_reuse_is_unrepresentable` fails on second use and Bank `redo_progression_terminalizes_its_one_shot_recovery_continuation` observes an empty registry. The continuation is still moved into `WorthQueryRedoRecovery::admit_deriving`, so "not reusable" is unchanged; what changed is that its denial paths now relinquish rather than drop (Q8.21-L11). |
| Q8.21-L11 | A **denied** transition returns the commit to exactly the recoverability it had a moment earlier: the mint claim is released and the receipt can open a fresh handle. A transition that actually **ran** — consumed, disposed, expired, force-terminated — spends the commit's one recovery forever. | The retry must be *performed* through the production assembly after a real denial, not asserted from a terminal label; plus a removal check that the retained half still holds after a successful transition. | **PROVED** — `WorthQueryRecoveryResourceTerminal::Relinquished` is a distinct fate and the only one that calls `WorthQueryRecoveryHandleRegistry::relinquish`, which removes the slot's entry from `claims_by_slot` and its claim from `claimed_commits`. Every by-value transition routes its preparatory checks through `WorthQueryRecoveryHandle::admit`/`admit_deriving` (or the same combinator on `WorthQueryUndoAdmission` / `WorthQueryRedoRecovery`), so the handle is relinquished on denial instead of falling out of scope into `Drop`. Bank `a_denied_transition_does_not_spend_the_commits_one_recovery` performs a real `FreshAuthorityDenied` reconcile against handle B, then re-mints B from its receipt and completes the transition; `a_completed_transition_spends_the_commits_one_recovery_permanently` proves the retained half. Verified by removal against the final source state: with `WorthQueryRecoveryHandle::relinquish` rewritten to `mark_terminal(slot, Relinquished)` — the pre-fix semantic — the first test fails and every other test in the file, including the affinity test that shipped as this row's neighbour, still passes. The Q8.21-L12 unit test correctly survives that removal, because it exercises the registry contract directly; the two layers are not redundant, and neither alone proves the guarantee. **Rung 3** (runtime-checked): the registry compares slot ownership; a caller can still supply the wrong authority, and the guarantee is that doing so costs nothing. **Reopened and re-proved at the slice-6 re-audit.** This row's claim was written unqualified — "a denied transition" — but its evidence and its mechanism both stopped at the crate boundary: `RelinquishOnDenial` is `pub(crate)`, so the whole class of denials raised *after* admission leaves Query, by the application host itself, was falsified. That is now Q8.22-C5's, and the correction moved the policy into `WorthQueryHeldRecoveryHandle::drop` so it holds for holders that have never heard of it. Read this row as covering Query-internal denials and Q8.22-C5 as covering the rest; together they make the unqualified sentence above true. |
| Q8.21-L12 | The line between the two halves of L11 is exactly one fate wide, and it holds for the two cases no transition can reach: a handle *abandoned* by `Drop` spends the recovery, and relinquishing an already-terminal slot cannot resurrect a spent commit. | Registry-level exhaustion over all five fates, both directions asserted by re-minting rather than by reading a label. | **PROVED** — `only_relinquishment_returns_a_commits_mint_claim` walks `Consumed`/`Expired`/`Disposed`/`ForceTerminated`, re-mints after each, and requires the denial; then relinquishes the same already-terminal slot and requires `false` back. The release direction re-mints successfully and asserts the retry gets a *fresh* slot. `Drop` is covered through `Disposed` because `Drop` calls `mark_terminal(Disposed)` — abandoning a handle is an exercise of the one recovery, which is why `Drop` was deliberately **not** changed to relinquish: if it had been, mint/drop/mint would loop forever and L7 would be worthless. `relinquish` also records no `terminated` entry, since a relinquished slot is unreachable by `consume`, `Drop`, or a retry, and retained entries would be caller-driven residue — a denial loop is the only path that produces unboundedly many slots for one commit. |

Ledger attack result: receipt cloning still exists by design because receipts
are historical/publication evidence. The surviving dynamic question is exactly
one owner-table claim, tested under a race; every continuation after successful
mint is compiler-linear. At slice 5 closure, the current undo target and the
eventual undo/redo causal edge remained separate open questions; slices 6 and 8
own them respectively.

Slice-5 re-audit (2026-08-07) — ledger-completeness defect and its correction,
retained as audit history:

The question that broke the ledger was "what meaningful defect could exist while
Q8.21-L7 through L10 all pass?" Answer: **a denied transition permanently
destroyed a commit's recovery.** Consuming transitions take the handle by value,
so every `?` on a denial path dropped it; `Drop` recorded `Disposed`; and
`register_once`'s claim was never released. One attempt with stale or wrong
authority spent a recovery that had not been exercised at all. Confirmed
empirically through the Bank assembly before any fix:

```
PROBE: denial = FreshAuthorityDenied
PROBE: B slot still live after a *denied* transition? false
PROBE: re-mint DENIED RecoveryAlreadyMinted — recovery permanently lost
```

No row covered it. L8's "denied one-shot ... remain no-write" is about database
writes. Worse, the shipped test `effect_authority_for_handle_a_cannot_transition
_handle_b_in_the_same_runtime` *performs* exactly this substitution and then
asserts `assert_no_live_handles()` — it passed **because** handle B had been
silently destroyed. That is the fixture-honesty lesson from this slice: a
teardown assertion that only asks "is the registry empty" cannot distinguish
"nothing leaked" from "everything was destroyed".

Correction, by root cause rather than by guard: the fate a handle records on
leaving is now a semantic distinction rather than a default. `Disposed` is a
decision; `Relinquished` is a non-event. Only the non-event returns the claim.
Affected rows reopened and restated: L7 (its "never releases the claim" text was
made false by the fix and was the wrong invariant to begin with), L8 and L10
(evidence restated; their linearity claims are unchanged because the combinators
still take `self`). Signatures did not change, so the E0382 move-value
compile-fail evidence behind L8 and L10 remains valid without re-blessing.

Deliberately *not* changed: `resolve_recovery_handle`'s
`UnresolvedExternalPosture` branch still consumes. Authority, handle, and
admitted read were all correct there — resolve genuinely ran and simply has no
resolution to report, so retrying would ask the same question of the same
unresolved posture. That is a completed transition, not a denial.

### Correction slice 6 closure ledger

| ID | Exact closure claim | Evidence required | Status and evidence |
|---|---|---|---|
| Q8.22-C1 | Undo cannot proceed from an absent or semantically unbound original input; recovered admission must represent the exact original governed meaning. | Private receipt/handle carriage, complete application-owned canonical identity, typed fail-closed denial, command-dimension identity twins, and a fresh-authority mismatch twin for the retained identity axis. | **PROVED** — `WorthQueryRetainedGovernedInput` is cloned only from the original operation admission and carries its canonical identity; `OriginalGovernedInputRequired` denies missing/unbound carriage. Freeze and death-notification identity tests vary every command dimension independently. Slice-7 ledger attack reopened this row, found that fresh recovery authority did not compare the retained governed-input identity, added it as a `worth-proof::Binding` axis, and proved `[0x66; 32]` versus `[0x67; 32]` denies distinctly as `GovernedInputMismatch`. |
| Q8.22-C2 | A recorded pre-image names the exact observed Relational record, and that record participated in the original commit. | Retention-source trace, exact-target unit assertion, and undo-admission touched-set membership check. | **PROVED (claim narrowed at slice-6 re-audit)** — `retain_attempt_preimage` reads `entity_id` and `kind` off observed decision facts, so provenance is honest and no caller supplies either; every retained field carries its `RecordRef`; `WorthQueryRetainedPreImage::target_record` returns `None` unless all fields share one record; `require_exact_preimage_target` rejects mixed or untouched targets before progression. The earlier wording said "record **and kind**", which overstated what is enforced: `entity_kind` is carried but never compared, and `WorthQueryTouchedRecordIdentity` holds only a `RecordRef` — there is nothing to compare it against. Plumbing a kind through the touched set was rejected rather than implemented, because `RecordRef::Entity(EntityId)` already identifies the record exactly; a kind comparison would be a redundant check dressed as an extra guarantee. The claim is narrowed to what the code actually enforces. |
| Q8.22-C3 | Callers cannot choose the undo mechanism lane or supply an action, target account, journal, amount, destination, or institution. | Public signature inventory plus compile-fail wrong-lane, extra-action, and extra-journal calls. | **PROVED** — `BankRecordedInverseUndoAdmission` and `BankCompensationUndoAdmission` are distinct move-only private-field continuations; three Bank doctests fail to compile and public progression has no semantic-input parameter. |
| Q8.22-C4 | Recorded inverse and compensation derive their domain targets from owner truth and still re-enter ordinary mutation progression. | Exact-record resolution, original-input/idempotency derivation trace, lawful positive twins, conflict/no-write twin, and independent money oracle. | **PROVED** — recorded inverse resolves the typed account and checks `matches_record` against the retained pre-image target; compensation derives the original journal and current institution before owning its continuation. Seven focused undo tests, the complete Bank ordinary suite, and all Bank compile-fail doctests exercise the resulting lanes. |
| Q8.22-C5 | An in-flight correction that is denied **after admission leaves Query** — anywhere in the application host's own progression — leaves the commit exactly as recoverable as it was, and the returned capability is *usable*, not merely a freed registry slot. Symmetrically, a correction that actually **committed** does not hand its capability back, and a handle relinquished by the redo continuation cannot undo an already-undone commit. | The denial must be performed through the production host entry point (not a Query-internal path), and closure must re-mint **and** re-admit **and** finish a real undo. The committed-undo direction must attempt a full second undo, not read a label. Fault-sensitivity by removal against the final source state. | **PROVED (added at slice-6 re-audit; the defect it names was live)** — slice 5 routed *Query-internal* denials through `RelinquishOnDenial`, but that trait is `pub(crate)`. `progress_admitted_undo` hands `WorthQueryUndoProgressionHandoff` — which owns the linear handle — across the crate boundary, and every `?` in `bank-server/src/estate_progression/undo.rs` (plus `finish_undo_progression`'s non-committed arm) dropped it. `Drop` recorded `Disposed`, so `register_once`'s `claimed_commits` entry was never released and one Bank-side conflict destroyed the commit's recovery permanently, for an undo that never ran. Confirmed empirically before any fix through the Bank assembly: `PROBE: denial = Undo(Conflicted)` then `PROBE: re-mint DENIED RecoveryAlreadyMinted — recovery permanently lost to a Bank-side denial`. Corrected at the root rather than by routing that one path: the `Disposed` default is right for a **bare** handle (Q8.21-L12 — otherwise mint/drop/mint loops forever and Q8.21-L7 is worthless) but wrong for a handle held inside a **preparation** that performed nothing. `WorthQueryHeldRecoveryHandle` carries the handle in undo admission, the redo continuation, and redo admission, and relinquishes from its own `Drop`. That is **rung 1 for the host**: there is no API left to get wrong, and the host never needs to hear of the policy. Bank `bank_side_undo_denial_leaves_the_commit_recoverable` denies through `progress_undo_commit_recovery` and then re-mints *and re-admits*; `aliasing_the_original_binding_denies_and_leaves_the_undo_still_performable` carries a denied attempt all the way to a committed undo; `a_relinquished_handle_after_a_committed_undo_cannot_undo_twice` drops the redo continuation, re-mints, and drives a second undo that is refused with the account unchanged. All three fail with `WorthQueryHeldRecoveryHandle::drop` neutered, and every pre-existing test in both files still passes under that removal — which is exactly the fixture-honesty gap this row exists to close. Deliberate asymmetry: `WorthQueryRedoRecovery::from_completed_undo` takes the handle out *before* answering and **consumes** it on seal failure, because the ordinary undo committed and the recovery is therefore exercised whether or not redo evidence could be sealed on top of it. |
| Q8.22-C6 | `progress_undo_recorded_inverse` still accepts a caller-authored `WorthQueryApplicationIdempotencyBinding`; a caller cannot use it to alias an undo onto the commit it is undoing and be answered `AlreadyCommitted` for an undo that never ran. | The substitution must be *performed* with the original commit's own binding, read from `handle.binding().idempotency()`, and the resulting state and evidence asserted. | **PROVED (attack examined and refuted at slice-6 re-audit; recorded so it is not re-derived)** — the caller's `(key_identity, intent_identity)` are inert salt. `WorthQueryApplicationIdempotencyBinding::new` is public, but `resolve_retained_idempotency` then binds `operation`, `operation_scope`, `preconditions`, `governed_input`, and `governed_proposal` from the admission, so a cross-commit alias differs on five axes it cannot reach. Bank `aliasing_the_original_binding_denies_and_leaves_the_undo_still_performable` performs the alias and gets `Denied { kind: IdempotencyIntentDrift }` with the account still `Frozen` and no proved-undo minted. **Rung 3** (runtime-checked), and deliberately asymmetric with redo, which derives *both* halves from the intent digest and the retained governed identity. Rung 1 was not taken here because the Bank owns retry identity for its own operator-facing keys; what the caller supplies buys nothing, and the row states that rather than pretending the parameter is gone. |

Ledger attack result (restated at the slice-6 re-audit): the earlier attack
paragraph asked only what a caller could *substitute*, and every row here was
about semantic inputs. It missed the class where a caller supplies nothing at
all and the defect is what the runtime does with a capability on the way out —
which is how Q8.22-C5's defect survived four fully proved rows. Type erasure
still remains only at the non-generic receipt boundary, where no caller supplies
replacement bytes, and Query still fails closed before Bank downcast if
canonical binding is absent. The only rung-3 correction-target step is the
genuinely dynamic resolution of the retained `RecordRef` against current owner
graph truth; mechanism choice and every public semantic-input substitution are
rung 1 or 2. Runtime Bridge remains only the installed inverse correspondence
owner and gains no input, history, head, or legality authority.

**What a caller can still supply on the slice-6 surface**: the authority object,
the target identity (the receipt or the handle), and — on
`progress_undo_recorded_inverse` and `progress_undo_commit_recovery` — a retry
identity that is inert salt (Q8.22-C6). Nothing else. The mechanism lane, the
action, the account, the journal, the amount, the destination, the institution,
the pre-image, and the governed input are all derived (Q8.22-C1, C3, C4).

**Structural backstop for Q8.22-C5's committed-undo direction**: a second undo
of an already-undone commit cannot land even if the domain check that fires
first in the Bank journey were absent.
`WorthQueryPendingAftermathCausality::expected_head` is
`ExpectedBranchHead::Commit(original)`; `invariant_execution.rs` puts it into
`TransactionOptions.expected_branch_head`;
`worth-relational` `validated_mutation.rs` rejects a mismatch at validate as
`StaleValidationBasis`, and `authority/commit/pipeline/execution.rs` rechecks the
same basis at commit. A committed undo advances the branch head past the
original commit, so the precondition can no longer hold. The Bank test asserts
the observable refusal; this paragraph records the mechanism beneath it rather
than claiming the test proves it.

### Correction slice 7 closure ledger

| ID | Exact closure claim | Evidence required | Status and evidence |
|---|---|---|---|
| Q8.23-C1 | A caller cannot construct proved-undo evidence from receipt fields or raw values; the proof describes completed undo but grants no current redo authority. | Private proof authority, owner-only mint path, and external-consumer compile-fail construction. | **PROVED** — `WorthQueryProvedUndo` privately contains `Proof<WorthQueryUndoCompleted, WorthQueryUndoCompletionAuthority>` and `Inverts<…>`; `axis_probe` is crate-test-only; `proved_undo_constructor_is_private` fails in the public certification crate. Exact Relational child/parent binding is not claimed here and remains Q8.24. |
| Q8.23-C2 | Redo cannot accept a replacement action, amount, destination, or raw idempotency binding at either admission or progression. | Public signature inventory and compile-fail calls against both old substitution seams. | **PROVED** — `admit_redo_disbursement_recovery` accepts only the move-only recovery, fresh principal/request, and descriptive intent; `progress_redo_disbursement` accepts only `BankDisbursementRedoAdmission`. Both old call shapes fail in Bank doctests. |
| Q8.23-C3 | The ordinary redo uses the exact original governed input and a deterministic binding derived from owner-carried meaning. | Missing-input fail-closed twin, exact typed downcast, binding assertions, and lawful production progression. | **PROVED** — Query moves `WorthQueryRetainedGovernedInput` through admission/handoff, denies missing carriage as `ChangedOperationMeaning`, derives key identity from the redo-intent digest and intent identity from the retained governed-input identity, and Bank downcasts only the carried `EstateAction`. |
| Q8.23-C4 | Fresh application context cannot be replaced after redo admission, and unrelated owner-minted proofs/intents remain non-substitutable. | Move-only application continuation plus production-path copied/foreign/unrelated twins. | **PROVED, strengthened in slice 8** — `BankDisbursementRedoAdmission` privately owns the exact freshly admitted ordinary disbursement operation; it no longer borrows principal/request and progression cannot re-admit or replace them. Production tests cross unrelated same-runtime proof/intent pairs, foreign principals, world drift, duplicate use, and divergence. |

Ledger attack result: slice 7 closes semantic replay and proof forgery, not
history ownership. The proof is deliberately descriptive and cannot replace
the co-committed typed causal fact or Relational expected-head authority owed by
slice 8. Query still owns the old mutable lineage chain at this checkpoint, so
Q8.24 and R8.45 remain open; Runtime Bridge gains no lineage, input, legality,
or fresh-authority role.

### Correction slice 8 closure ledger

| ID | Exact closure claim | Evidence required | Status and evidence |
|---|---|---|---|
| Q8.24-C1 | Query contributes aftermath meaning but cannot choose commit identity, ordered parents, branch head, or publication order. | Private typed causal-fact construction, same-transaction persistence, and committed evidence sealed from the Relational result. | **PROVED** — sealed undo/redo handoffs alone create `WorthQueryPendingAftermathCausality`; the provider adds its record to the operation's `WorkerIntentBatch`; `WorthQueryCommittedAftermathCausality::seal` accepts only the exact Relational child whose sole parent is the carried parent; proved undo now requires that exact committed fact. |
| Q8.24-C2 | Query owns no parallel history/head and callers cannot select the descriptive redo head. | Exact deletion plus external-consumer compiler failures and residue scans. | **PROVED** — `linear_lineage.rs`, the runtime mutex/head, raw node/row types, and every append/record/count API are deleted. `WorthQueryRedoIntent::derive` is crate-private; `query_has_no_lineage_store` and `redo_intent_derivation_requires_runtime` fail in certification. Runtime derives the intent from the current Relational `CommitReference`. |
| Q8.24-C3 | An intervening commit after Query admission but before publication cannot race the redo into history. | Owner-enforced expected-head validation, post-validation recheck, and an application-level fault-window test. | **PROVED** — `TransactionOptions::expect_branch_head(ExpectedBranchHead)` is checked by Relational during validation; `ValidatedRelationalMutation` carries the branch-qualified validation basis that the owner rechecks at commit. Two Relational tests prove both boundaries, and `relational_head_advance_after_redo_admission_closes_the_commit_race` proves the Bank continuation cannot commit after the branch advances. |
| Q8.24-C4 | Recovery cannot treat a plain idempotency record, failed Query projection, or Query cache as causal truth. | Missing-causality fail-closed twin, fault after Relational commit, owner reread, and absence of a Query cache. | **PROVED** — an equivalent idempotency row without the exact causal fact is `IdempotencyIntentDrift`; committed paths become indeterminate if their required fact cannot be recovered. `causal_fact_survives_index_publication_failure_via_relational_owner_read` injects publication failure, recovers the exact fact, and rereads it from Relational owner state. No Query causality cache exists. |
| Q8.24-C5 | Runtime Bridge cannot decide ordinary undo/redo legality or currentness. | Dependency/source residue over the causal, admission, and commit surfaces. | **PROVED** — the Phase 8 causal fact, typed commit entry, and redo admission contain no Runtime Bridge import or Bridge lineage/currentness type. Existing Bridge historical-continuity vocabulary remains unrelated and is not consumed by this lane. Publication lowering remains Q8.27. |

Ledger attack result: the correction does not move the competing authority into
Runtime Bridge or hide it behind a facade. Relational owns the only history and
the atomic expected-head decision; Query owns only typed semantic causality;
Bank owns the operation-specific ordinary admission. The one dynamic rung-3
check is current Relational owner truth. Missing causal evidence fails closed,
including idempotency recovery, rather than degrading to an optional proof.

### Correction slices 9A–9B closure ledger

| ID | Exact closure claim | Evidence required | Status and evidence |
|---|---|---|---|
| Q8.25-C1 | An installed external-effect contract names one schema-declared typed operation emission and a bounded wire projection; a payload type without that projection cannot be declared for the lane. | Declaration/install source trace, compile-fail unprojectable payload, and canonical-identity drift twin. | **DEFECT → CORRECTED → PROVED (slice 9A).** The escaping posture was declared *twice*: once as an aftermath axis (`DeclaredExternalEffectPosture` with a correlation demand), once as the real schema lane (`ApplicationSchemaMember::OperationExternalEffect`). Nothing reconciled them, and the reversibility guard read the **claim**. An operation could therefore assert a reconcilable external owner while projecting onto no lane — co-committing no outbox record and dispatching nothing — or declare a lane while claiming containment. Correction is **rung 1**: the aftermath axis and its `external_effect.rs` are deleted, so the second declaration is unrepresentable; `DeclaredApplicationAftermathContract`'s constructors are `const fn` over mechanism and reconciliation only. Evidence — *source trace*: the installed external posture is derived from the schema lane alone in `install.rs`; *compile-fail unprojectable payload*: `external_effect_requires_projected_payload.rs` (E0277 on `ApplicationExternalEffectPayload`); *identity drift twin*: `every_external_effect_contract_dimension_changes_identity` plus the new `the_installed_external_posture_follows_the_operation_lane`, which asserts `identity().bytes()` differs across the lane axis. Adversarial test performing the substitution: `an_operation_that_escapes_cannot_install_as_reversible`. The aftermath fixture was itself an instance of the defect — an escaping posture with no lane behind it — and now declares real effects, emits, and `operation_external_effect` members. |
| Q8.25-C2 | The outbox co-commits the exact payload projected from the admitted matching emission. Compare-and-commit and dispatch expose no caller-supplied effect name, payload type, payload bytes, or payload bound. | Missing/wrong/duplicate emission denials, exact outbox persistence read, and public signature/compile-fail evidence. | **PROVED (slice 9A), on evidence that did not previously exist.** *Emission denials*: the four pre-existing `external_payload_tests.rs` cases. *Exact outbox persistence read*: **new** `outbox_persistence_tests.rs` — `the_outbox_persists_the_installed_contract_and_the_admitted_payload_exactly` runs the whole production chain (`batch.external_payload(&contract)` → `derive_dispatch_outbox_record` → `dispatch_outbox_create_intent`) and reads all seven persisted fields back through the layout, asserting the payload hex equals the emission's own `external_effect_bytes()` and that effect, payload type, correlation family, and byte bound come from the installed contract; `a_declared_lane_with_no_projected_payload_derives_nothing` (`MissingExternalPayload`) and `an_undeclared_lane_derives_and_persists_nothing` (R8.4 zero-cost) hold the negatives. *Public signature*: `WorthQueryDispatchOutboxRecord::new` is `pub(crate)` — readable, unauthorable — and `dispatch_outbox_record_is_not_caller_forgeable.rs` fails E0624 in the public certification crate. **Fault sensitivity:** substituting the persisted payload bytes on a green 598-test baseline failed exactly one test — the new one. Before it existed the substitution was invisible to the entire suite; the only prior outbox test passes `(None, None)` and proves nothing about the declared path. |
| Q8.25-C5 | Exactly one place decides whether an escaping operation may install as reversible, and it reads only facts the runtime derived. | Single-guard source trace plus fault sensitivity by neutering the guard. | **DEFECT → CORRECTED → PROVED (slice 9A).** Found by the fault-sensitivity probe for C1: neutering `validate_external_effect_reversibility` failed *nothing*, because a second guard inside `install.rs` re-derived the same decision inline from `(authority, mechanism)`. Two guards, the pre-flight one re-deriving what `derive_published_posture` already owns and free to drift from it — the same two-declarations-of-one-fact shape as C1, inside C1's own fix. Correction is **rung 1**: the pre-flight twin is deleted. The surviving guard compares the posture the runtime just derived against the operation's own declared lane; neither input is a claim the aftermath declaration could make about itself. Re-probed after deletion: neutering the survivor fails `an_operation_that_escapes_cannot_install_as_reversible`. |
| Q8.25-C3 | The separate Bank rail receives and decodes estate, notice, and subject meaning; correlation-only delivery cannot satisfy the protocol. | Real-process positive path, malformed/wrong payload rejection, and rail-ledger domain assertions. | **OPEN — slice 9B** |
| Q8.25-C4 | Provider commit, emitted application causality, dispatch attempt, and acknowledgement/completion carry distinct identities and exact predecessor links. | Production posture trace plus hostile tests that compare every identity and predecessor and prove no earlier posture constructs a later one. | **OPEN — slice 9B** |

Ledger attack result (slice 9A): the question that broke the ledger was "what
defect could survive while an installed contract, a declared lane, and a
published posture all validate?" Answer: **they can disagree, because two of
them were separately declared.** C1 and C5 are both instances — an aftermath
claim competing with the schema lane, then a pre-flight guard competing with the
derived posture — and neither had a row before this slice, so each is recorded
as two failures: the implementation defect and the ledger-completeness defect
that let it hide. C5 is the row that did not exist at all; it was written
because C1's fix probe was insensitive, not because a test failed.

What a caller can still supply on the slice-9A surface: nothing that decides
whether an operation escapes, what it puts on the wire, or whether it may
install as reversible. The escaping declaration is **rung 1** (the competing
aftermath axis is deleted and unnameable); the outbox record's four
wire-bearing fields are **rung 1** on the caller axis (`new` is `pub(crate)`,
proved by compile-fail); payload projectability is **rung 1** via the
`ApplicationExternalEffectPayload` bound; and the reversible-versus-escaping
conflict is **rung 3** — whether a particular installation pairs a reversible
posture with a declared lane is a fact about that installation, checked once,
by the single surviving guard.

### Correction slice 10 closure ledger

Q8.26 is the same defect class as the rest of this correction: **the caller
supplies what the runtime must derive.** The installed contract already owns a
non-optional `InstalledPreImageDemand` on every `InstalledRecordedInverse`, yet
the attempt re-acquires the demand from a public `with_preimage_demand` builder
that each Bank operation must remember to call. Retention is therefore opt-in to
the very party it exists to constrain, and forgetting is silent.

| ID | Exact closure claim | Evidence required | Status and evidence |
|---|---|---|---|
| Q8.26-C1 | The pre-image demand attaches from the admitted operation's compiled aftermath for every operation declaring `RecordedInverse`. No caller-reachable opt-in exists that could omit it. | Derivation source trace from `allowed_graph_contract().aftermath()`; a committed receipt carrying a retained pre-image for **each** declaring operation (Freeze, Revoke, ApproveEmergencyAccess); residue proof that no public demand-attachment API remains. | **PROVED** — `installed_preimage_demand` (`application_attempt/provider_binding.rs`) derives the demand from the installed contract at the single `provider_execution/entry.rs` call site. The `preimage_demand` field and the `with_preimage_demand` builder are deleted, and `attach_freeze_preimage_demand` (which carried two silent `return program` early-outs) is gone from `bank-server`. Receipts: Freeze via `recorded_inverse_undo_restores_prior_status_from_retained_preimage`, Revoke via `revocation_retains_its_declared_preimage_without_a_per_operation_opt_in`, Approve via `retention_follows_the_declared_mechanism_on_both_lifecycle_commits`. Fault sensitivity: with the derivation neutered to `None`, the Revoke test failed while **all 7** pre-existing tests passed — including three that commit revocations. That gap is the fixture-honesty defect this finding names. |
| Q8.26-C2 | A demanded pre-image that cannot be retained denies the commit. The mutation does not commit and consequential state is unchanged. | One case per typed denial (`MissingDemandedField`, `ExceedsByteBound`, `EmptyDemand`) asserting no commit and unchanged graph, plus fault sensitivity by neutering the fail-closed path and confirming the new cases fail while pre-existing ones pass. | **PROVED, with the evidence re-framed.** `retain_attempt_preimage` no longer ends in `.ok()`; it returns `Result` and is `?`-propagated at `session_lifecycle.rs:101`, *before* the atomic transaction. Denial coverage is unit-level (`undo_preimage_tests.rs`): `MissingDemandedField`, `ExceedsByteBound`, and the two variants added at C7. The *consequence* — that a pre-transaction Commit-stage failure aborts and applies nothing — is proved by the pre-existing `pretransaction_commit_failure_is_proved_aborted_and_applies_nothing`, whose injection point sits three lines below the retention call on the identical path. **`EmptyDemand` is not runtime-reachable**: `DeclaredPreImageDemand::new` rejects an empty slot list at construction (rung 2), so that arm is defensive only. The original evidence column asked for a runtime case per variant; manufacturing one for `EmptyDemand` would require a fixture that cannot exist, so it is recorded as unconstructable rather than staged. |
| Q8.26-C3 | Retention binds the exact demanded field, not a prefix of a multi-segment path. Two distinct nested fields cannot collapse onto one demanded slot. | A candidate whose `CanonicalFieldPath` carries more than one segment must not satisfy a single-segment demand via its first segment; exact-match positive twin. | **PROVED** — the projection reduced each observed path to `field_path().fields().first()`, letting a nested `Account.Status` answer a demand for `Status`. The rule is now the pure `demanded_field_slot`, tested both ways by `only_a_single_segment_path_names_a_demanded_field_slot`. |
| Q8.26-C4 | The retained value is the pre-mutation value observed at admission, never the value the operation writes. | For each declaring operation, a commit whose written value differs from the prior value, asserting the receipt retained the prior one and that undo restores prior rather than written. | **PROVED by existing evidence, oracle verified.** `recorded_inverse_undo_restores_prior_status_from_retained_preimage` asserts the retained `Status` is `"open"` while the same commit leaves the account `Frozen`, then restores to `Open`. Retention taking the written value fails that assertion. Structurally, candidates are projected only from `attempt.facts` — the admitted decision read-set — and never from the effects or the live graph. |
| Q8.26-C5 | An operation whose demand is not covered by its declared decision reads cannot install. | `covered_preimage_demand_installs_with_operation_decision_reads` and `uncovered_preimage_demand_denies_operation_installation_by_name`, re-inspected for fixture honesty rather than inherited as green. | **PROVED, re-inspected.** `validate_preimage_coverage` denies a zero byte bound, a demand against an operation declaring no reads, and any uncovered slot — each arm ordered so it can actually fire. `OperationDeclaredReadFields` has no public constructor and is built only at the operation-compile resolution site, so a caller cannot author a coverage list independent of the operation being installed (**rung 2**), with the check itself at rung 3. |
| Q8.26-C6 | Every committed receipt for a `RecordedInverse` operation carries a retained pre-image, so undo admission cannot find it absent. Receipts for non-declaring operations carry none. | Receipt assertion across all three declaring operations plus the negative twin on a `not_correctable` and a `Compensation` operation. | **PROVED** — all three declaring operations assert a retained pre-image (see C1). Negative twins: `retention_follows_the_declared_mechanism_on_both_lifecycle_commits` proves the `not_correctable` create lane `RequestEmergencyAccess` retains nothing *in the same journey* as the positive, and `compensation_operations_retain_no_preimage` proves the same for `DisburseEstate`. Asserting only the positives would pass against a runtime that retained indiscriminately. |
| Q8.26-C7 | When the decision read-set observes the demanded field slot on more than one record, retention cannot silently select one by position. | A two-entity world where both records carry the demanded slot; retention must bind the mutated record or deny — never take the first match. | **DEFECT → CORRECTED → PROVED.** Selection was `find(\|c\| c.field_slot() == slot)` — slot name alone, no record binding — so a read-set observing the demanded slot on several records (the ordinary shape when an invariant is checked across siblings before one is written) retained whichever iteration reached first, and the receipt bound a prior truth belonging to a record the commit never touched. `retain_preimage_from_observed_facts` now takes the attempt's mutated record set, derived at the call site from `attempt.batch.intents` (creates excluded — a record the operation brings into existence has no prior truth). A slot is satisfied only by an observation of a mutated record; two such observations deny as `AmbiguousDemandedField`, and an operation mutating no existing record denies as `NoMutatedRecord`. Tests: `retain_preimage_binds_the_mutated_record_not_the_first_matching_slot`, `..._denies_when_only_unmutated_records_observed_the_slot`, `..._denies_an_ambiguous_demanded_slot`, `..._denies_when_the_operation_mutates_no_existing_record`. Fault sensitivity: reverting selection to slot-name-only failed the two binding tests while the four pre-existing retention tests passed. |

Ledger attack: with C1–C6 passing, retention could still succeed while binding
the **wrong record**. `retain_preimage_from_observed_facts` selects with
`find(|c| c.field_slot() == slot)` — slot name alone, no entity binding — so a
read-set observing the demanded slot on two entities retains whichever was
enumerated first. Slice 6 bound each retained field to its exact observed record
and required that record in the commit's touched set, which makes a *wrong*
record detectable at undo admission; it does not stop retention from choosing
one. C7 exists because of this attack and is the sharpest row in the slice.

**The attack landed.** C7 was a live defect, not a hypothetical: retention chose
by iteration order, and the wrong-record case was reachable without any caller
doing anything unusual. The correction moved the decision onto truth the runtime
already owns — the attempt's own mutation intents — so the demanded slot is
satisfied only by an observation of a record this commit changes. Two such
observations now deny rather than resolve. This is the second slice-10 row whose
root cause is the same shape as Q8.26 itself: a decision that looked like a
lookup was really an authority question, and the answer was taken from whatever
was nearest rather than from the party that owns it.

What a caller can still supply on the slice-10 surface: nothing that decides
retention. After C1 the demand comes from the compiled contract, the candidates
come from the already-admitted decision read-set, and after C7 the target comes
from the attempt's own intents — so the demand-attachment parameter is **rung 1**
(deleted, unnameable) and the retention target is **rung 1** on the caller axis
(there is no parameter through which to name it). Fail-closed retention is
**rung 3** — whether the admitted read-set observed the demanded field *on a
mutated record* is a fact about a particular execution that no static type can
decide — and the slice states that rather than claiming a stronger rung. One arm
is stronger than rung 3 and is recorded as such: `EmptyDemand` is unconstructable
at declaration (**rung 2**), so its runtime arm is unreachable defence.

## Cross-cutting requirement ledger

These govern every gate and close only when the last gate that touches them
closes.

| ID | Guarantee | Status | Evidence / what remains |
|---|---|---|---|
| R8.0 | Phase 8 obeys the 9.16.1 reconciliation policy: parity evidence before retirement, exact retirement in the same slice, no long-lived dual authority. | **PROVED** | Honoured at 8.1–8.5. Gate 8.6 residue proves exact removal (not privatization): monolith `operation_aftermath` directory absent; bank-local `EstateAftermath` enum absent (`declared_aftermath_for` sole path); no bank-server Phase-8 generic rollback door (`phase8_residue::r8_50_*`). |
| R8.1 | Carriers C1-C4 are repaired, each owned by a named gate; strengthening preserves unforgeability. | **PROVED** | C3/C4 at 8.2; C1 at 8.3; C2 at 8.4. |
| R8.2 | Resolution A: the installed inverse contract declares exactly which pre-image it demands; installation rejects a demand not covered by declared reads. | **PROVED** | Installation side at 8.1 (R8.18). Consumption completed at 8.4 turn 3. |
| R8.3 | No ordinary Phase 8 path calls a `*_for_replay` retention API. | **PROVED** | Gate 8.6 mechanical scan of `application_aftermath/**` finds zero `for_replay` / `*_for_replay` hits (`phase8_residue::r8_3_*`). |
| R8.4 | Operations declaring no external effect pay exactly zero. | **PROVED** | 8.2 undeclared-effect live-then-zero; re-checked at 8.6 R8.51. |
| R8.5 | Recovery correlation binds to typed Query identity, never a provider `String`. | **PROVED** | 8.2 row K1. |
| R8.6 | `resolve_by_idempotency` executes an admitted graph read of the idempotency record. | **PROVED** | Bound admitted read; Bank `resolve_commit_recovery`. |
| R8.7 | Phase 8 consumes one host-published time source; callers and adapters cannot supply a sample or choose the evaluation moment. | **PROVED** | M1–M3. |
| R8.8 | The CDC posture is stated explicitly either way, and no second change stream over Relational exists. | **PROVED** | 8.2 E1, E3, E4. |
| R8.9 | The destination inverse contract references the installed Bridge correspondence. | **PROVED** | Gate 8.4 turn 2. |
| R8.10 | Each new identity-bearing family prepares its own ready canonical basis. | **PROVED** | Handle ordinal (R8.34); undo intent basis; redo intent `CanonicalBasisDomain::Future("worth-query.application-aftermath-redo-intent")` now binds the exact undo and owner-observed head `CommitReference` fields. The co-committed causal fact has no parallel identity family: its semantic identity is the typed role plus Relational-owned parent commit; its private key is only a storage locator. |
| R8.11 | No Signal decision, slot value, or explanation classifies an aftermath or posture. | **PROVED** | Gate 8.6 mechanical scan: zero `worth_signal` / `WorthSignal` in `application_aftermath/**` (`phase8_residue::r8_11_*`). |
| R8.12 | Recovery-handle durability publishes as an explicit posture. | **PROVED** | `StoreCapabilityRequired` on inspect. |
| R8.12a | Original identity comes only from the Relational-backed receipt; every undo/redo causal fact is privately prepared by Query and co-committed with its ordinary mutation; callers supply no role, child commit, parents, head, or publication order. | **PROVED — correction slice 8** | The handoff privately creates the pending fact, provider batches it with the ordinary mutation, and the committed type seals the exact Relational parent/child. Plain idempotency without the fact fails as drift. |
| R8.12b | Query owns no aftermath history chain or head; redo binds an owner-observed Relational head and compare-and-commit consumes it atomically. | **PROVED — correction slice 8** | Query chain/head APIs are deleted. Runtime-only intent derivation reads Relational; typed `ExpectedBranchHead` and the validated-candidate recheck close both race windows. |
| R8.12c | Runtime Bridge owns no ordinary Phase 8 aftermath legality, history, or current-head authority; it may only transport a completed admitted projection for a real cross-runtime consumer. | **PROVED — correction slice 8** | Causal/admission/commit surfaces contain no Bridge lineage authority and Query consumes none. Installed inverse correspondence under R8.9 remains unchanged; publication projection remains Q8.27. |
| R8.13 | Three phase slots — `external_dispatch`, `undo_admission`, `redo_admission` — exist with no defaulted-to-zero slot. | **PROVED** | 8.2 N1; 8.4/8.5 populate undo/redo; 8.6 R8.51 asserts all three zero on ordinary commit. |
| R8.14 | Fan-out independence: growing posting, decision-fact, and lineage counts change no §8 counter. | **PROVED** | Courtroom row 12 now grows authoritative Relational history by 1 vs 100 real commits and leaves undo-admission 1/1/0; intent unit twins retain the posting/history fan-out slope proof. No Query row counter exists. |
| R8.15 | Lane separation: reconstructive inspection and compensation are distinct lanes. | **PROVED** | Inspection 0/0/0; undo/redo populate distinct slots; ordinary commit pays 0/0/0 on all three aftermath slots while transport is live (R8.51). |
| R8.52 | Installed aftermath carries correction authority and correction mechanism as two independent axes; the published posture is derived. | **PROVED** | 8.1. |
| R8.53 | The mechanism axis is populated with exactly `RecordedInverse` and its named siblings — no open-ended extension point. | **PROVED** | 8.1. |
| R8.54 | `ProvisionalDiscard` and `NoMutation` are lifecycle transitions, unreachable from every aftermath type. | **PROVED** | 8.1. |
| R8.55 | No operation emits an escaping effect without a committed local anchor. | **PROVED** | Gate 8.2 O1 + Gate 8.6 turn 3 O2: `mutation_free_external_effect::mutation_free_external_effect_co_commits_outbox_recovers_lost_response_once` (emit-only `RetransmitDeathNotice`; outbox is the sole domain-write-free anchor). See **Q8.11**. |
| R8.56 | An operation declaring an external effect may not declare `Reversible`. | **PROVED** | 8.1. |
| R8.59 | Phase 8 does not widen PB1 (the eight-parameter `ApplicationFieldRef`). | **PROVED** | Gap-closure corrective: platform eighth parameter renamed `Currency`→`Unit` (`ApplicationFieldUnit` / `NoApplicationUnit` / `ApplicationUnitRef` / `worth_query_unit!`). Bank finance `trait Currency` / `Money<C: Currency>` / `UsdCurrency` marker name retained. `rg ApplicationCurrencyMarker workspaces/worth-query/crates/worth-query-declaration` empty. |
| R8.60 | Phase 8's own new surface repeats none of the PB patterns. | **PROVED** | Typed `WorthQueryAftermathDerivationFailure` replaces `&'static str`; no new ordinary-branch literal; no test-local `BranchId` construction in Gate 8.6 fixtures. |
| R8.61 | PB1, PB2, and PB4 are entered in the Bank World gap ledger. | **PROVED** | Entered under R8.61; **gap-closure corrective CLOSED** all three (rename + branch owner + residue). Bank front-door rows updated to CLOSED. |
| R8.63 | Phase 8 produces this ledger, created when Gate 8.1 opens, updated by each gate as part of that gate's closure. | **PROVED** | Gate 8.6 updated this ledger at closure with evidence columns moved, not status-only edits. |
| R8.64 | Every gate from 8.3 onward contributes at least one scenario to a named cross-gate integration suite, exercising its product **through** the gates beneath it. | **PROVED** | 8.3–8.5 cross-gate retained; 8.6 adds publication noninterference, fan-out courtroom, ordinary-commit cost, residue suite, and turn-3 mutation-free O2 e2e under `ordinary_mutations` — accumulated suite green (80). |
| R8.65 | Test worlds obtain installed contracts, receipts, handles, and postures through a named world-construction authority in test scope, never a constructor exported from a production crate. | **PROVED** | Production install/mint unchanged. Gate 8.6 courtroom fixtures (`freeze_account/fixture`, `disburse_estate/fixture`, cross-gate worlds) construct through Bank production paths; no production facade fixture ctor. |

## Gate 8.1 requirement ledger — CLOSED

Independent oracle written before inspection: 31 rows, 28 substantive.
Consumer evidence: 372 tests across three consumer targets.

| ID | Guarantee | Status | Closure evidence |
|---|---|---|---|
| R8.16 | Exactly one correction-authority value and, where correctable, exactly one correction-mechanism contract per installed mutation; published posture derived from the pair; no default or fallback on either axis. | **PROVED** | Complete operation inventory with no unclassified mutation. Missing, contradictory, and host-authored aftermath meaning each rejected at installation for its own reason. |
| R8.17 | Semantic inverse, compensating operation, reconciliation procedure, and terminal denial are four distinct installed contracts, not one callback with a mode field; the postcondition is a field, never a variant axis. | **PROVED** | The seven-variant `WorthQueryOperationReversalContract` cross-product was retired in the same slice after parity evidence, not wrapped. |
| R8.18 | The inverse contract declares its pre-image demand; installation rejects a demand not covered by declared reads or exceeding the declared bound. | **PROVED** | Negative case with positive twin; rejection names the uncovered read. |
| R8.19 | Classification binds to exact operation, schema, package, compatibility generation, commit posture, and result contract. | **PROVED** | Drift attack per binding axis. |
| R8.20 | One Foundational canonical basis and structured comparison; compact digest admitted only through Foundational's typed slot; derived once per installed or rebuilt meaning. | **PROVED** | Structured comparison used for drift attacks; residue denial for direct-hash and debug-string identity grammars. |
| R8.21 | The public outcome type exposes only next actions installed for its exact posture; an irreversible operation has no `undo` method — type-level absence, not runtime denial. | **PROVED** | Compile-fail evidence with positive twin. |
| R8.57 | An operation declaring an external effect cannot install as `Reversible`; rejection at installation names the escaping effect. | **PROVED** | Negative case with positive twin. |
| R8.58 | `ProvisionalDiscard` and `NoMutation` are unreachable from every aftermath type. | **PROVED** | Exhaustive match plus residue search. A coverage hole on `NoMutation` was found mid-gate and repaired. |

**Defects found and repaired inside Gate 8.1:** missing `NoMutation` coverage;
negative cases lacking positive twins; and a fixture constructor exported from
the public production facade (see **Q8.1**).

## Gate 8.2 requirement ledger — CLOSED

Independent oracle written before inspection: 34 rows. Closed over two turns;
turn 1 correctly self-reported the gate as not closed.

| ID | Guarantee | Status | Closure evidence |
|---|---|---|---|
| R8.22 | Seven distinct typed postures; no posture derivable from possession of an earlier one. | **PROVED** | Rows L1, L3: seven variants each carrying identity and `ExternalEffectCausalLink`; `ProviderCommit` rootless. Successor variants `EmittedApplicationCausality`…`ExternalCompletion` (and Compensation/Reconciliation) each carry `ExternalEffectPostureEvidence` as part of the variant shape — possession of a predecessor link alone cannot construct a successor (Q8.3 CLOSED). |
| R8.23 | Each posture carries stable exact identity and a causal link to its predecessor; one dispatch or causality event derives one identity. | **PROVED** | Row N2: `assert_single_dispatch_derivation(receipt.canonical_work())` asserted inline — exactly 1 basis prep, 1 digest derivation, 0 text materializations. Delivery, ack, and timeout classification carry identity at 0 preps, 0 derivations. |
| R8.24 | Timeout, disconnect, lost response, duplicated acknowledgement, and unknown provider outcome are classified without guessing; "unknown" is first-class with its own recovery posture. | **PROVED** | Rows L5, L6: five fault classifications, each exercised end-to-end **through production dispatch**, each failing for its own intended reason. |
| R8.25 | Dispatch intent co-commits with the mutation; a mutation-free external effect still commits its dispatch record; operations declaring no external effect pay zero. | **PROVED** | O1: NotifyDeath co-commit. **O2 (Gate 8.6 turn 3):** `mutation_free_external_effect::mutation_free_external_effect_co_commits_outbox_recovers_lost_response_once` — emit-only `RetransmitDeathNotice`, `co_committed_dispatch_outbox()` with scaffolding-only `changed_record_count()==2` and unchanged death-notice status. O3/O4: undeclared money-movement zero twin. Corrective for Gate 8.2's O2-on-O1 mislabel — see **Q8.11**. |
| R8.26 | `Indeterminate` and `PartialEffect` carry the correlation evidence the layer beneath produced; `CommitRecoveryRequired` vs `AbortRecoveryRequired` survives to the application boundary. | **PROVED** | Rows K2, K3: both now carry `WorthQueryApplicationUnresolvedCommitEvidence`. This is the C3 repair. |
| R8.27 | Foundational completion, provenance, and freshness vocabulary describe a posture only after the Query boundary is known and cannot upgrade an indeterminate effect to completed. | **PROVED** | Rows F1, F2: ordering inspection plus negative case. |

**Exit-proof rows (the boundary is real).** B1: `bank-external-rail` has a
`[[bin]]`; tests spawn it via `CARGO_BIN_EXE` and assert the rail PID differs
from the test PID. **B2 is proved by construction** — the rail's dependency
list is `serde`, `serde_json`, `tokio` and nothing else, so it *cannot* reach
the runtime's truth source. That is stronger than any test could be. B3-B8:
seven exit-proof tests covering lost response, ack-without-completion,
disappearance, duplicate ack, late completion, and a success twin. No fault
reports `Completed`.

**Ledger attack.** Six defect shapes were predicted before inspection; four
were checked and none materialized. The two that mattered: "separate process
but still shares truth" (defeated structurally by B2) and "zero-cost proved
only where no transport exists" (defeated deliberately — the undeclared-effect
test asserts live transport *before* asserting zero).

**Defect found and repaired inside Gate 8.2:** the time source was aliased
rather than renamed (row M1), leaving two names for one authority.

## Gate 8.3 requirement ledger — CLOSED

Production owner: `worth-query-execution/.../application_aftermath/recovery_*`
plus C1 on `WorthQueryApplicationCommitReceipt`. Production assembly also lives
on `BankIdentityRuntime` (`open_commit_recovery`, `admit_commit_recovery_*`,
`dispose/reconcile/compensate/resolve/inspect_commit_recovery`). Cross-gate
consumer: `bank-server` `phase8_cross_gate` / `phase8_recovery_*`.

Closed at turn 4 against this ledger (R8.63). Turn 4 closes the last
evidence-substitution defect: the admitted idempotency read is bound to the
binding it was read for, is not `Clone`, and foreign reads deny with
`ForeignIdempotencyRead` — distinct from `IdempotencyMismatch` on the handle
axis. Turn 5 does not reopen the product rows; it repairs verification
integrity (Q8.8, Q8.9) so Gate 8.3 evidence remains trustworthy under load.

**Auditor confirmation (2026-08-06).** Closure rests on checks re-run by the
auditor against an oracle written before turn 1 existed, not on the
implementer's report. Confirmed independently: bank `ordinary_mutations` **49**;
consumers **313 / 37 / 22**; `compile_certification` **14** (green for the first
time since Gate 8.1); `worth-query-execution --lib` **549 × 5 consecutive runs
on a deliberately loaded machine**; `boundary-check` and `agent-context` exit 0;
no touched file over 400 lines; `compare_and_commit.rs` decomposed into
`compare_and_commit/{mod,commit_receipt,commit_outcome}.rs` rather than deleted.

Three claims were verified by reading rather than by trusting, because each was
a place where a green result could have been manufactured:

- The re-blessed `.stderr` diff removes **only** the stale
  `WorthQueryCompensationCapability` suggestion; `E0432` and the "no such symbol
  in `facade::domain`" assertion survive intact. Nothing was laundered.
- `assert_axis_drift` runs its positive twin (`expect("positive twin admits")`)
  **before** each one-axis mutation, and every one of the eleven asserts its own
  denial kind.
- All `axis_probe` constructors are `#[cfg(test)] pub(crate)`, and
  `reset_for_integration_test` is `#[cfg(feature = "test-support")]`, enabled
  only from `bank-server`'s `[dev-dependencies]` under `resolver = "2"` — so
  non-test builds genuinely omit the symbol rather than merely hiding it.

The gate's defining evidence is
`recorded_inverse_aftermath_admits_reconcile_and_denies_compensate`: one
installed `RuntimeWithExternalOwner` + `RecordedInverse` contract driven through
the production runtime under a real rail fault, where `reconcile` admits on the
**authority** axis while `compensate` denies on the **mechanism** axis. It is
the only test in Phase 8 that fails if the two-axis model were quietly reduced
to dispatch on the derived posture name, and it exists because the oracle named
that configuration before any implementation could suggest it.

### Ledger attack against R8.28 (audit of slice 3A)

Slice 3A's own claim — "mint is receipt-only" — was true and was the wrong
question. Reading the whole mint path rather than the report showed that mint
derived every binding field from the receipt and checked nothing at all about
*who* was minting. A commit receipt is `pub` and `Clone`; a process may publish
several application runtimes; `register_once` claims commits per registry. So a
second runtime could open a live handle for a commit it never admitted, and both
runtimes would then hold a live handle for the same commit.

`provider_runtime_instance_id` cannot detect this and never could. It names the
**Relational** instance, which every Query runtime published over one source
shares — the axis that would have caught it, the runtime-instance binding axis,
was removed in slice 3B. The receipt now carries the admitting **Query** runtime
authority, taken from the admitted operation at `from_admission`, and mint
compares it against the runtime it is minting into.

The compile-fail evidence was also attacked and did not survive.
`recovery_mint_rejects_caller_aftermath` failed on `E0061` — "remove the extra
argument" — which is what any second argument produces, including `42u8`. Once
the parameter is gone, arity is the only error obtainable, so an arity case can
never be the proof; the passing twin is. It was replaced by a case that proves
what makes the single lane worth anything: the receipt on it cannot be built by
struct literal, its fields cannot be read or rewritten, and the constructor
rustc suggests in its own help text is out of reach.

### Ledger attack against R8.30 and R8.66 (audit of slice 3B)

Slice 3B deleted parameters, and its compile-fail evidence was written to match
the deletions rather than to survive them. Three cases —
`recovery_transition_rejects_caller_aftermath`,
`redispatch_requires_handle_and_authority`, and
`safe_retry_requires_performed_redispatch` — passed an extra argument, or too
few, and failed on `E0061`. Every one of them would have failed identically for
`42u8`, and none could ever do better, because once a parameter is gone arity is
the only error left to obtain. All three were retired for cases that name what
survives deletion: the binding a transition reads instead cannot be constructed
or looked inside, and the re-dispatch proof cannot be constructed at all.

`external_effect_requires_projected_payload` was worse than weak. It imported
its declaration macros from the crate root rather than from `facade`, so it
failed on `E0432` — an unresolved import — and had therefore never once reached
the `ApplicationExternalEffectPayload` bound it was written to prove. It was
green the whole time on an error unrelated to its subject. Repaired, it now
fails on `E0277` against that bound.

Two runtime gaps were also found by reading, not by running. Safe-retry's
comparison of the presented re-dispatch proof against the handle's own outbox
carried a comment claiming it "rules out swapping a proof minted against handle
A into safe-retry for handle B" — a claim with no test behind it, because every
integration path mints the proof against the handle it then retries. And of the
seven binding axes, `branch_version_ordinal` was only ever drifted together with
`branch`, which is declared first and so reports first: neutering the ordinal
comparison left the entire pre-existing suite green.

### Ledger attack against R8.29 (audit of slice 4)

Slice 4's own row said it: "`LinearResource` owns the one-terminal law; the
runtime registry owns only live enumeration, force termination, and `Drop`."
Read as an inventory of what the *registry* owns that is right. Read as an
inventory of what a *host* can reach it was wrong, and the row did not
distinguish the two. `enumerate_live`, `mark_terminal`, `force_terminate`,
`terminal_of` and `assert_no_live_handles` were all `pub`, reachable through a
`pub fn registry()` on any handle and a `pub fn recovery_handle_registry()` on
the runtime itself.

Those methods are addressed by *slot ordinal*, not by the handle — a privileged
operation taking an identity the caller chose instead of the resource it acts
on. `enumerate_live` then supplied the ordinals, including ordinals belonging to
handles the asker never held, and `mark_terminal` let the caller pick the
recorded terminal kind. Performed through the production Bank assembly with two
live handles in one runtime, the holder of handle B enumerated
`[slot 2, slot 1]`, force-terminated both, and left handle A dead in its owner's
hand with the ledger reading `ForceTerminated` — an outcome the runtime never
caused. `consume()` on A would then deny `AlreadyTerminal` and its `Drop` would
record nothing. Q8.9 had closed the *named* affordance (`reset_for_integration_test`)
without closing the ordinary methods sitting beside it.

The correction is rung 1 and rung 2, not a runtime check. `recovery_handle_registry()`
on the runtime had no caller in either workspace and was deleted outright, as
was `WorthQueryRecoveryHandle::registry()`. `registry_arc()`, `force_terminate`
and `assert_no_live_handles` are `#[cfg(any(test, feature = "test-support"))]`,
which `worth-query-certification` does not enable; `enumerate_live` carries the
same gate and is additionally `pub(crate)` — clippy confirms no production path
wants it. `mark_terminal` and `terminal_of` are `pub(crate)`, leaving
`WorthQueryRecoveryHandle::consume` and `Drop` as their only production callers,
both of which hold the handle itself and name the terminal the runtime caused.

**What a caller can still supply, and why it is nothing.** `is_live` stays
public deliberately. `WorthQueryRecoveryRegistrySlot` wraps a private `u64`,
appears in no facade, and is constructed nowhere outside `recovery_registry.rs`;
with `enumerate_live` closed, the only slot an out-of-crate caller can obtain is
its own, from `registry_slot()` on a handle it holds. So the sole question it
can ask is whether its own handle is still live — which it could answer anyway
by attempting a transition.

| ID | Guarantee | Status | Evidence / what remains |
|---|---|---|---|
| R8.62 / C1 | Receipt names installed operation, principal scope, idempotency binding; derived from admission; no public ctor. | **PROVED** | Unchanged; still derived via `from_admission`. |
| R8.29-C1 | A host cannot reach the recovery registry's slot-addressed lifecycle controls, and so cannot terminate — or choose the recorded terminal of — a handle it does not own. | Hold one handle and attempt the reach a bystander would: handle → registry, registry → live slots, slot → terminal. | **PROVED under controlling F8.1-C1/C2.** `test-support` no longer exists, the registry type is absent from the facade, and the handle exposes neither registry nor slot. The compile attack produces E0432 plus E0599 for both handle routes under default and `--all-features`. |
| R8.30-C1 | Each of the seven current-truth axes denies on its own, with a distinct cause, against a positive twin. | Drift one axis at a time from a matching baseline; assert the exact denial kind for each. | **PROVED under controlling F8.3-C1/F8.4-C1.** `binding_axis_tests` has eight cases over seven axes (`branch` carries both `BranchMismatch` and `ForeignBranchEqualOrdinal`). `application_binding_generation_drift_denies_on_its_own_axis` expects exact `ApplicationBindingGenerationMismatch`; bypassing the comparison makes that test admit. |
| R8.66-C1 | A re-dispatch proof is affine to the handle it was performed for; a handle with no co-committed outbox admits none. | Perform a real dispatch for handle A's outbox, then present the resulting proof for handle B; exact `CorrelationMismatch`, with a positive twin on A's own handle. Repeat against a handle whose binding carries no outbox. | **PROVED** — `primary_graph::application_attempt::provider_execution::external_dispatch::safe_retry_affinity_tests::{redispatch_performed_for_handle_a_cannot_safe_retry_handle_b, safe_retry_denies_when_the_handle_carries_no_co_committed_outbox}`; both fail if either arm of the outbox comparison is removed. Rung 3: both sides are runtime values. |
| R8.28 | Handle mint derives its immutable binding, including exact installed aftermath identity and operation slot, from the committed receipt; recovery-not-admitted cannot mint; and the minting runtime must be the runtime that admitted the commit. | **PROVED** | Slice 3A C1-C4: receipt-only public signature, production `from_receipt` carriage, Bank identity/slot observation, and unchanged 0/0/0 counters. C5 adds the missing provenance comparison. Compile-fail evidence is now `recovery_mint_receipt_is_not_caller_forgeable` (private-fields struct literal, E0616, E0624) plus the positive twin `recovery_mint_uses_receipt_only`; the former arity attempt was retired. Transition currentness is separately R8.30/R8.31. |
| R8.28-C5 | A commit receipt admitted by one application runtime cannot mint a recovery handle in another, even within one process over one relational source. | Publish two runtimes, commit in the first, and present that receipt to the second's public `mint_recovery_handle`; exact `ForeignRuntime` cause, and the committing runtime still mints from the same receipt. | **PROVED** — `phase8_exact_handle_authority::a_receipt_committed_by_another_runtime_cannot_mint_a_handle_here`. Removing the `ensure_receipt_belongs_to_runtime` call makes the bystander runtime return a **live** handle (`slot 1, registry_live: true`) for a commit it never admitted. |
| R8.29 | Managed-run-family registry; linear consume/expire/dispose; no Clone/Copy. | **PROVED** | Slice 4 proves one-handle terminal law with `LinearResource`, atomic terminalization, typed freshness, and four leak-free terminal paths. Slice 5 adds the permanent authoritative-commit mint claim, cloned/concurrent receipt denials, move-only undo ownership, private committed-undo→redo continuation, consuming redo admission, and terminal redo progression. C1 adds what the original row conflated: the terminal law binds the *registry*, and now also binds every host — the slot-addressed lifecycle controls that could retire another holder's handle are gone from the production surface. |
| R8.30 | Six distinct transitions keyed off the handle-carried installed contract/mechanism, with no caller-selected receipt or contract lane. | **PROVED** | All six transition families read the exact installed contract from `WorthQueryRecoveryHandleBinding`; no transition accepts receipt, aftermath, recovery-target, or reconstructed-current-observation arguments. Compile-fail substitution plus execution mechanism denials and real Bank positive twins cover the boundary. |
| R8.31 | Fresh authority before effect; inspect requires disclosure; authority is affine to its exact runtime and handle. | **PROVED** | Current operation admission remains the only caller-presented freshness input and compares the live schema, branch, version, operation, principal, and exact installed-aftermath identity. Effect and inspect proofs each carry owner-sealed current-validity artifacts over runtime-owner and exact-handle bases; cross-runtime, same-runtime cross-handle, grant-expiry, foreign-principal, disclosure, and typed clock-branch evidence is green. |
| R8.32 | Resolve via G5 taxonomy; never upgrades unresolved. | **PROVED** | `WorthQueryAdmittedIdempotencyRead` privately minted, bound to the read-for binding, and non-`Clone`. `resolve_recovery_handle` demands it and denies `ForeignIdempotencyRead` when the read is foreign (handle right, read wrong). Unresolved denies; already-completed and foreign-read twins covered. |
| R8.33 | Mint/provider inquiry/inspect 0/0/0; no public identity or registry lookup helper. | **PROVED under F8.1** | Bank `phase8_recovery_counters` observes only public binding, provider posture, and inspection work and remains exactly 0/0/0. The public `integration_identity_ordinal` and registry lookup steps are absent because they widened or depended on internal identity/lifecycle state. |
| R8.34 | Opaque wire identity unforgeable; support/posture/opaque cannot readmit. | **PROVED** | Slice 4 projects through `BoundaryBridgedAuthorityRevalidationRequiredBasis`, exposing descriptive bytes without current Query authority. No reverse constructor exists; support, posture, and opaque readmission denials remain green. |
| R8.35 | Support-truth publication; handle sole next-action authority; durability explicit. | **PROVED** | Unchanged. |
| Exit proof | Eleven scenarios + leak detection + bound-read twin. | **PROVED** | Lost-response, dispose+wire, foreign-principal (axis), grant-policy current denial, disclosure inspect, compat drift+twin, already-completed resolve, clock-advanced expiry, foreign-runtime/foreign-branch-equal-ordinal (axis), T4b mechanism×authority, four-path leak, trybuild clone/dup/inspect≠effect, foreign admitted-read denial + matching twin. |

**Q8.6 disposition (turn 3).** Orphaned `query_effect_lifecycle_authority()` retired
(no successor in the new aftermath topology — recovery uses
`WorthQueryRecoveryEffectAuthority`). The type marker remains in the identity
family map. `ordinary_facade_cannot_import_replay_capability.stderr` re-blessed
after confirming E0432 still denies for the right reason; `WorthQueryCompensationCapability`
suggestion gone. `cargo test -p worth-query-certification --test compile_certification`
is green (14 passed).

**Q8.3 disposition (gap-closure corrective).** All six successor postures
require `ExternalEffectPostureEvidence` in the variant shape. `ProviderCommit`
remains the rootless origin. R8.22 is PROVED on unrepresentability; Q8.3 is
**CLOSED**. Zero deliberate carries remain.

## Finding ledger

| ID | Impact | Finding | Status | Closure evidence |
|---|---|---|---|---|
| Q8.1 | High | Gate 8.1 exported a fixture constructor from the public production facade, minting installed contracts from fabricated digests. It existed because the specification demanded honest installed identities and named no sanctioned way for a test world to obtain one, so the implementer invented a shortcut and exported it. A production facade exporting a fixture constructor is a defect regardless of whether production code calls it. | **CLOSED** | Constructor removed at 8.1. Gate 8.6 proves named test-scope world construction through Bank production mint/admit (R8.65). |
| Q8.2 | Medium | The specification referenced a Phase 8 closure ledger in its exit condition without requiring anything to create it. Gates 8.1 and 8.2 closed against per-gate scratch ledgers with no durable phase-level record. | **CLOSED** | **R8.63** added; this file created and backfilled from both per-gate ledgers. The backfill is honest but retroactive, which is exactly the weakness R8.63 now prevents. Every gate from 8.3 onward updates this file as part of its own closure. |
| Q8.3 | Medium | R8.22's "no posture is derivable from an earlier one" is enforced by module visibility, not by the type system. No consumer can violate it, but internal code holding a predecessor's causal link could construct a successor. | **CLOSED** | Gap-closure corrective: all six successor postures require `ExternalEffectPostureEvidence` in the variant shape; `ProviderCommit` remains rootless. Mint owned by dispatch/classification. R8.22 now PROVED on unrepresentability. |
| Q8.4 | High | `bank-courtroom` contains zero tests. Gates 8.3, 8.4, and 8.5 each consume the previous gate's product, so four layers stack before the first cross-cutting adversarial proof at 8.6 — the condition in which an authority leak survives every local suite. | **CLOSED** | Gate 8.6 accumulated cross-gate + publication + fan-out + residue + O2 mutation-free e2e under `ordinary_mutations` (80 green). Bank Phase 5 later supplied the separate-process courtroom; Bank Phase 6 remains downstream. |
| Q8.5 | Medium | Two of Phase 8's entry conditions named work that no phase was scheduled to produce: Gate 8.2's external boundary and Gate 8.4's C1-C4 carrier repairs. Both were papered over during execution rather than by the specification — the boundary by instructing the implementer to build it anyway, C3/C4 by an implementer who happened to need them. | **CLOSED** | §10 added. |

| Q8.6 | High | **Gate 8.1's retirement of `domain_installation/operation_aftermath/` was not exact (R8.0).** It removed the monolith correctly in production terms but left two pieces of residue, both of which survived Gate 8.1's closure, Gate 8.2's closure, and two Gate 8.3 turns: (a) `query_effect_lifecycle_authority()` in `identity_authority/authority.rs:62` lost its only caller and is now dead code with an unused import, emitting two build warnings; (b) the certification fixture `tests/ui/replay/ordinary_facade_cannot_import_replay_capability.stderr` still expects rustc to suggest the retired `WorthQueryCompensationCapability` from `facade::domain`, so `cargo test -p worth-query-certification --test compile_certification` has been **red (13 passed, 1 failed)** since Gate 8.1. | **CLOSED** | Gate 8.3 turn 3: witness function retired (no successor in recovery topology); identity family type retained. Stderr re-blessed after confirming E0432 still holds for the right reason. `compile_certification` green (14 passed). Warning-clean build. |
| Q8.7 | Medium | **The audit method itself had a gap.** Gates 8.1 and 8.2 were closed against evidence lists naming the three Query consumer targets, the Bank suite, `boundary-check`, `agent-context`, and the line cap — but never `-p worth-query-certification --test compile_certification`, and never a warning check. Gate 8.1's exit proof explicitly demanded residue denial and compile-fail evidence, so that target belonged in its evidence list. Q8.6 is the defect this gap concealed; the gap is the reason it survived three closures. | **CLOSED** | Standing verification set enforced from Gate 8.4 onward and completed at Gate 8.6 with every named target reported (including `compile_certification` and warning-clean build). |
| Q8.8 | High | **`worth-query-execution --lib` was not reliable evidence.** Phase 7 live-lease test `cancellation_and_deadline_terminalize_all_live_resources` opens a live query under a wall-clock `Instant` deadline and `.unwrap()`s the open. Under CPU load the setup exceeds its own deadline before open completes (`Admission(DeadlineExceeded)`), so the suite is green only on lucky idle runs. Surfaced during Gate 8.3 verification; Phase 8 never touched `application_query/live/`. Intent is that cancellation and deadline expiry terminalize live resources — not that the machine is fast enough. | **CLOSED** | Gate 8.3 turn 5: live-lease deadline path still uses wall-clock `Instant` (Gate 8.3's injectable `WorthQueryRuntimeClock` is SystemTime/authorization, not yet wired into request-scope deadlines). Option 2 applied: open under a non-expiring scope, then bind an already-settled deadline only to the poll phase under test — no sleep, no widened wall-clock. After Q8.9's test lock, five consecutive `--lib` runs were all `549 passed`. Injectable Instant/request-scope clock remains the preferred successor if Phase 7 live deadlines are revisited. |
| Q8.9 | High | **`WorthQueryRecoveryHandleRegistry::reset_for_integration_test` was production-callable.** `#[doc(hidden)]` is not access control; any consumer could wipe the managed-run recovery registry and silently orphan live handle terminal records — the defect class leak tests exist to detect (sharper Gate 8.1 / R8.65 pattern). Existed because the registry is process-global and `#[cfg(test)]` does not reach cross-crate integration tests. The same process-global design also made `assert_no_live_handles` flake under parallel `--lib` runs (turn 5 run 5: `matching_admitted_idempotency_read_resolves` saw a foreign live slot). | **CLOSED** | Gate 8.3 turn 5 closed the instance (`test-support` wipe). **Gate 8.4 turn 1 closes the cause:** registry is `Arc`-owned by `WorthQueryPrimaryGraphApplicationRuntime`; handles hold the same `Arc`; `reset_for_*` and `lock_for_test` deleted. Isolation falls out of construction. |

| Q8.10 | Medium | **Gate 8.4's entry condition names work that Gate 8.4 itself must build.** The entry reads "G1 resolution implemented (R8.2). G8 typed (R8.9)," but this ledger assigns R8.2's consumption side and all of R8.9 to Gate 8.4. That is the Q8.5 defect shape — an entry condition naming unscheduled work — occurring inside a specification that added §10 specifically to forbid it. Found by applying §10.5's standing check to Gate 8.4's entry before the gate opened, which is the first time that check has been run prospectively rather than after the fact. | **CLOSED** | Gate 8.4 turn 1: entry rewritten to "This gate builds its own entry condition," naming R8.2 consumption, R8.9, and C2 as this gate's first obligations — matching Gate 8.2's §10 repair. No work was unscheduled; only the wording lied. |
| Q8.11 | High | **Gate 8.2 marked O2 (mutation-free external effect still co-commits its dispatch record) PROVED on O1 evidence.** Every live `co_committed_dispatch_outbox()` assertion rode a mutating operation (`NotifyDeath` / disbursement). R8.25/R8.55 therefore read PROVED while the clause that makes anchoring structural — an operation with no domain mutation whose outbox is its sole local anchor — had no named end-to-end proof. Surfaced by Gate 8.6 turn 2's §11 courtroom map (row 14 honest gap). Spec §9 append-only corrective; Phase 8 is the unfinished dependent. | **CLOSED** | Gate 8.6 turn 3: Bank `RetransmitDeathNotice` (emit-only, no domain writes) + Query substrate admitting empty provisional programs / empty invariant state loads for emit-only commits. E2e `mutation_free_external_effect::mutation_free_external_effect_co_commits_outbox_recovers_lost_response_once` through real `bank-external-rail`: outbox co-commits, status unchanged, scaffolding-only count 2, lost-response idempotency recovery, rail attempts exactly once. R8.25/R8.55 evidence columns renamed to this test. |

| Q8.12 | Medium | **REOPENED during correction slice 3A verification.** The formerly unnamed intermittent failure recurred on consecutive execution-suite run 2 of 5 and is now identified: `domain_computation::primary_graph::tests::application_query::pinned_basis::expired_pinned_basis_is_rejected_and_released`. Setup creates a request with a five-millisecond deadline, then unwraps `pin_current_application_query_basis`; under load the request expired before pin admission and returned `WorthQueryApplicationPinnedBasisDenial { kind: DeadlineExceeded, subject: "request" }`. This is test-harness timing, not the slice-3A receipt/handle path, but the old row explicitly required reopening on recurrence. | **CLOSED** | Preserved full failing output: run 1 passed 582; run 2 failed 581/582 at `pinned_basis.rs:214`. **Closed at the root, not by re-counting greens.** The sleep and the five-millisecond pin scope are both gone. The test now captures a settled `Instant` *before* pinning, pins under a non-expiring scope, and applies that instant via `WorthQueryApplicationPinnedBasis::with_deadline_settled_at` (`#[cfg(test)]`, lease untouched, so the release assertion still means what it says). Both expiry comparisons are `Instant::now() >= expires_at` against a monotonic clock, so a pre-pin instant is settled by construction — the test requires no elapsed time and becomes *more* certain the slower the machine is, inverting the old failure direction. Verified by removal: dropping the `with_deadline_settled_at` call fails the test at `pinned_basis.rs:255`. Five consecutive `-p worth-query-execution --lib` runs: **589 passed, 0 failed** each. **Residual, same class, not this row:** `authorization::ordinary_admission.rs:287` and `hostile_resolution.rs:144` authenticate with a 20ms credential, run several `.unwrap()` setup steps, then assert the credential is *still valid* before sleeping past it — setup that fails when the machine is slow. Inverting these needs a settable credential deadline (the authentication path has no injectable clock, same gap Q8.8 recorded for request-scope deadlines); they have not been observed failing. |

| Q8.13 | High | **The dispatch outbox is write-only, and `safe_retry` admits an action no production path performs.** `dispatch_external_effect` has exactly one call site (`provider_execution/external_dispatch.rs`, in-process, immediately after commit). Nothing re-dispatches. `safe_retry_recovery_handle` returns an admission, performs no dispatch, and is the only one of R8.30's six transitions with no Bank consumer — the other five are called from `bank-server/src/estate_progression/recovery.rs`. The reachable failure needs no crash: a transport fault on a live process yields `Unresolved`, a mintable handle, and a committed outbox row whose effect never escaped. Found by end-to-end trace of the dispatch subsystem **after** Phase 8's gates had closed, not by a failing test — no `R8.*` row required a re-dispatch, which is why every gate passed without one. Same shape as **Q8.11**. Spec §9 append-only corrective → **Gate 8.7**. | **CLOSED** | Gate 8.7 turn 2: `redispatch_admitted_external_effect` on the runtime, privately-minted `WorthQueryAdmittedExternalRedispatch` required by `safe_retry_recovery_handle`, and Bank consumer `safe_retry_commit_recovery` — all six R8.30 transitions now have a production consumer. Seven e2e scenarios through the real `bank-external-rail` (`phase8_safe_retry`), four trybuild cases. |
| Q8.15 | High | **`redispatch_admitted_external_effect` needed no recovery handle (Gate 8.7 turn 1).** It took a caller-supplied `WorthQueryDispatchOutboxRecord` plus an admission. Because `WorthQueryApplicationCommitReceipt::dispatch_outbox()` is `pub` and the record type is facade-exported, a consumer holding a receipt and a current admission could emit the external effect **with no recovery handle in existence at all**, bypassing Gate 8.3's lifecycle and leaving R8.30's linear consumption unenforced on that path. The three "denies before transport" tests passed only because the single Bank consumer happened to call `admit_recovery_effect_authority` first — call-order discipline, not a property of the signature. Found by reading the diff, not from the turn report, which described the work as complete and green. **Third instance of the caller-supplies-what-should-be-evidence class** (after Gate 8.3 turn 1's `capability_currently_grants: bool`). | **CLOSED** | Turn 2: signature is `(handle, authority, admission)`; `require_fresh_effect_authority` runs before any transport reach; outbox read from `handle.binding()`. Compile-fail `redispatch_requires_handle_and_authority`. |
| Q8.16 | High | **Exactly-once was proved only where the answer was already known (Gate 8.7 turn 1).** The two scenarios were `DisappearMidDispatch` (rail never admitted — retrying something that provably did not happen) and `Succeed` (effect completed and Query knows it). Neither is the case the outbox machinery exists for. The indeterminate case — rail admitted, response lost, Query holding `Unresolved` and unable to tell — was untested, and that is precisely where a duplicate emission would occur. Same defect class as Gate 8.4 turn 2: **a test asserting something adjacent to its claim.** | **CLOSED** | Turn 2: `phase8_safe_retry::lost_response_after_commit_safe_retry_emits_nothing_twice` under `CommitThenLoseResponse` — asserts the rail admitted (`admission_count()==1`, ledger `Completed`) while the receipt posture is `Unresolved`, then safe-retries and asserts the count is still 1. |

| Q8.17 | High | **The standing set's warning-clean row had never been run at the scope it implies, and the bank-world workspace was red under it.** `RUSTFLAGS=-Dwarnings cargo check --workspace --all-targets` exits **101** in `worth-query-bank-world` with 17 diagnostics, from two causes that both predate Gate 8.7: (a) `tests/support/mod.rs` is compiled into all **eleven** integration-test binaries, but `AuthorizationTimeController` / `runtime_with_authorization_time` are used only from `ordinary_mutations`, so the other ten report dead code; (b) `phase8_redo_support.rs` carried `pub undo: BankCommitReceipt`, written by nine call sites and read by none. Surfaced at Phase 8 final audit. **Third instance of the Q8.7 class** — the row was added *after* Q8.6 precisely to catch orphaned code, and the reported evidence was `-Dwarnings (touched crates)`, a narrower target answering a broader row. Compounding it, the auditor's own first check ended in a fixed `echo` rather than `$?`, so the failure was masked until re-run. | **CLOSED** | Cause (a) fixed structurally, no suppression: the time-control items moved to `ordinary_mutations/authorization_time.rs`, compiled only into the binary that uses them; ten import sites rewritten. Cause (b): field removed — the undo receipt stays load-bearing inside `commit_and_prove_undo` where it seals the proof, but is not carried out speculatively. Both workspaces now exit 0 with zero diagnostics. Standing set row rewritten to name the exact command **and scope**. |

## Platform-boundary defect ledger

Discovered during Phase 8 scoping; recorded here so they cannot be lost. Full
statements in §13 of the specification.

| ID | Defect | Status |
|---|---|---|
| PB1 | `ApplicationFieldRef` carries an eighth `Currency` type parameter — a domain concept in a platform-generic signature. | **CLOSED** | Renamed to `Unit` / `ApplicationFieldUnit` / `NoApplicationUnit` / `ApplicationUnitRef` / `worth_query_unit!`. Bank finance `Currency` vocabulary retained. |
| PB2 | `ApplicationCapabilityAmountDimension` / constraint `amount` name finance magnitude bounds. | **CLOSED** | Renamed to `ApplicationCapabilityMagnitudeDimension` / `magnitude` (request projection `magnitude` / `magnitude_value`). Bank money `.amount()` retained. |
| PB4 | One production `"main"` branch literal; test-local branch counterfeits. | **CLOSED** | All production and execution-src sites route through `application_branch` (`primary_relational_branch_id` / `primary_truth_branch_identity`). Residue test: no `BranchId("main")` outside `application_branch.rs`. |

## Standing verification set

Q8.7, Q8.8, and Q8.9 are three instances of one failure: **evidence that looked
green without reliably being green.** A red `compile_certification` survived
three gate closures; a load-sensitive test made two separate "all passed"
reports true only on an idle machine; a registry wipe sat on the production
surface where the leak tests could not see it.

None was an architecture defect. All three were gaps in what we *ran* and how
we read the result. From Gate 8.4 onward, no gate closes without every row
below, and a gate closed without one has an incomplete evidence list rather
than a proved one.

| Check | Command | Why it is here |
|---|---|---|
| Bank consumer suite | `cargo test -p bank-server --test ordinary_mutations` | End-to-end through production dispatch |
| Query consumer targets | `installed_operating_world`, `public_declarative_journeys`, `runtime_public_journeys` | Public boundary parity |
| **Certification compile-fail** | `cargo test -p worth-query-certification --test compile_certification` | **Added after Q8.6.** Witnesses facade contents; a retirement that misses it leaves a red test nobody runs |
| Execution unit suite, **repeated** | `cargo test -p worth-query-execution --lib`, **five runs, all reported** | **Added after Q8.8.** One green run is not evidence of determinism |
| Warning-clean build | `RUSTFLAGS=-Dwarnings cargo check --workspace --all-targets`, **run in both workspaces** | **Added after Q8.6**, scope pinned after **Q8.17**. Orphaned code from a retirement shows up here first — but only at `--all-targets`: the crate-scoped form was reported green for three gates while `worth-query-bank-world` was red |
| Structural guards | `boundary-check`, `agent-context`, dirty line-cap | Mechanical, not reviewed |
| Production-surface residue | no test affordance reachable without `test-support` | **Added after Q8.9**, restated after the slice 4 audit. Q8.9 closed the affordance that *looked* like one (`reset_for_integration_test`, `#[doc(hidden)]`) and left the ordinary `pub` methods beside it untouched, though they could do the same damage. The check is whether a host can reach the operation, not whether the operation is named like a fixture |

Two reading rules, which cost us more than the missing commands did:

1. **Report the target you ran, by name.** "trybuild pass" was true of a
   narrow new target while a broader one was red. A named target cannot be
   mistaken for its neighbour.
2. **A single green run of a timing-sensitive target proves nothing.** Report
   every run, not the best one.
3. **Name the scope, not just the command** (added after Q8.17). "`-Dwarnings`
   passed" was true of a crate-scoped check while `--workspace --all-targets`
   was red in another workspace. A row that says *warning-clean build* is not
   satisfied by a narrower invocation of the same binary.
4. **Check the exit code, not the presence of output** (same finding). A shell
   pipeline ending in a fixed `echo` reports success unconditionally; a grep
   that finds no matches looks identical to a command that never ran. Capture
   `$?`.

## Auditor's final confirmation — Phase 8 (2026-08-06)

Every gate closed against an oracle written **before** its implementation
existed, and on checks re-run by the auditor rather than read from a report.

Final independent run: bank `ordinary_mutations` **80 × 2**; consumers
**313 / 37 / 22**; `compile_certification` **14**; `worth-query-execution --lib`
**578 × 3** (and ×5 by the implementer); `boundary-check` valid;
`agent-context` exit 0; warning-clean; no touched or new file over 400 lines.

**The closing gate's substrate change was audited rather than accepted.** Turn 3
made empty provisional programs and empty invariant state loads lawful for
emit-only commits — a production change that makes a previously-invalid state
valid, which is where guarantees weaken quietly. Both sites were read:
`validate_closure_and_symbols` still runs (an empty step list validates
vacuously — there are no symbols to be out of closure), the family allowlist
still applies to whatever locators exist, and the load guard was **conditioned,
not removed**:

```rust
if loaded.is_empty() {
    if !self.expected_locators.is_empty() || counters.loaded_facts() != 0 {
        // still denies EmptyStateLoad
```

An empty load is admitted only when the plan is also empty and zero facts
loaded. A provider returning empty against a non-empty plan still denies. The
precondition moved from *always deny* to *deny unless the plan agrees* — the
correct semantics for emit-only.

### What the method produced that the tests did not

Three of the phase's most consequential findings were found by **checking the
list of things checked**, not by finding broken code:

- **Q8.6** — a red `compile_certification` target survived three gate closures
  because it was in no gate's evidence list, though Gate 8.1's exit proof
  pointed straight at it.
- **Q8.7** — the audit method's own gap, which is why Q8.6 survived. Corrected
  into the standing verification set.
- **Q8.11** — Gate 8.2 marked O2 `PROVED` on O1 evidence. Surfaced only when
  Gate 8.6 built the §11 traceability map, and it mattered: the mutation-free
  case is the clause that makes R8.25's anchoring **structural rather than a
  cost optimization**. Without it, R8.55's law was proved only where a domain
  write already existed to carry the dispatch record.

The pre-written oracle also produced calibrating **negative** results — four
ledger-attack predictions that did not materialize: the double-entry oracle is
genuinely independent of production accounting; compensation counts committed
journal rows rather than requests; the redo intent type does not carry its
lane's policy; and the closing gate moved evidence columns rather than statuses.
A ledger attack that only ever confirms suspicions is not calibrated; these are
the evidence that its confirmed rows were worth acting on.

### Defect classes, by gate

The class shifted as the phase progressed, and naming each one made the next
instance cheap to spot:

| Gates | Class | Visible in |
|---|---|---|
| 8.3, 8.4, 8.5 | A caller supplying what should have been evidence | A signature |
| 8.4 | A test asserting something *adjacent* to its claim | The test body, read against the requirement |
| 8.1, 8.2, 8.6 | Evidence that looked green without reliably being green | The list of things checked |

Turn counts fell as each class was named in a brief before the next gate opened:
8.3 took five turns, 8.4 three, 8.5 two, 8.6 three (two of them corrective
rather than constructive). The fresh-admission requirement — R8.31, R8.37,
R8.43 — cost three turns at its first appearance, one at its second, and one at
its third, where it was the only row left open.

## Gate 8.7 requirement ledger — CLOSED (turn 2)

Append-only corrective under spec §9. Opened 2026-08-06 from **Q8.13**.

| ID | Guarantee | Status | Evidence |
|---|---|---|---|
| R8.66 | `safe_retry` consumes proof of a completed re-dispatch, not permission to attempt one. | **PROVED** | `WorthQueryAdmittedExternalRedispatch`: private fields, `pub(crate) mint`, no `Clone`. Compile-fail `admitted_external_redispatch_constructor_is_private` proves **both** doors — private-fields struct-literal error and `E0624` on `mint` — not merely that the parameter exists. Arity case `safe_retry_requires_admitted_redispatch` + positive twin `safe_retry_with_admitted_redispatch`. |
| R8.67 | `dispatch_external_effect` stays the single classification site; re-dispatch adds no second effect lane. | **PROVED** | Mechanical: `transport.dispatch(` has exactly one call site (`external_effect/dispatch.rs:102`); zero `WorthQueryExternalTransportOutcome::` matches outside `dispatch.rs`/`transport.rs`. Both post-commit dispatch and re-dispatch call it. |
| R8.68 | R8.28's correlation-evidence axis binds the co-committed outbox record, not the correlation identity alone. | **PROVED** | `WorthQueryRecoveryHandleBinding::dispatch_outbox` replaces the bare correlation; `correlation()` preserved for existing callers. Axis-probe drift case `correlation_mismatch_drift_denies_distinctly` rewritten to drift the record, so every axis stays bound. |
| R8.69 | Re-dispatch runs through the same fresh-authority path as every other effect-producing transition (R8.31). | **PROVED** | **Turn-2 corrective (F1).** `redispatch_admitted_external_effect(handle, authority, admission)` calls `require_fresh_effect_authority` **first** and reads the outbox from `handle.binding()` — a caller can no longer supply one. Compile-fail `redispatch_requires_handle_and_authority`. Bank: expired / terminal / foreign-principal each deny with `transport.attempts()` and `admission_count()` unchanged. See **Q8.15**. |
| R8.70 | Exactly-once is asserted at the rail, not at the request layer. | **PROVED** | Rail is an idempotent receiver: `apply_fault_script` consults `ledger.status_of` before running the fault script. `admission_count()` counts first-time ledger admissions. Three scenarios: never-admitted (`DisappearMidDispatch` 0→1), known-completed (`Succeed` then `DuplicateAcknowledgement`, stays 1), and **genuine indeterminacy** (`lost_response_after_commit_safe_retry_emits_nothing_twice`, `CommitThenLoseResponse`: rail admitted, Query holds `Unresolved`, retry keeps count at 1). See **Q8.16**. |
| R8.71 | The in-memory durability limit is published as a typed posture; no Query crate gains a `worth-store` dependency. | **PROVED** | `WorthQueryDispatchOutboxDurabilityPosture::StoreCapabilityRequired` surfaced on the safe-retry admission and asserted end-to-end. The posture states the process-local lifetime as a fact about this runtime; it tracks no outstanding obligation. |

### Gate 8.7 turn 1 → turn 2

Turn 1's architecture was correct — layering, single classification site, and
the rail's idempotent-receiver design all held under audit, and four ledger
attacks came back clean. Four findings from reading the diff:

- **F1 (High)** — `redispatch_admitted_external_effect` took a caller-supplied
  `WorthQueryDispatchOutboxRecord` and no handle. Since
  `WorthQueryApplicationCommitReceipt::dispatch_outbox()` is `pub` and the
  record type is facade-exported, **a consumer holding a receipt and a current
  admission could emit the external effect with no recovery handle in existence
  at all** — bypassing Gate 8.3's lifecycle entirely, and leaving R8.30's linear
  consumption unenforced on that path. The three "denies before transport" tests
  passed only because the one Bank consumer happened to call in the safe order.
  This is the Gate 8.3 turn-1 defect class (**the caller supplies what should be
  evidence**) in new clothing.
- **F2 (High)** — exactly-once was proved only where the answer was already
  known: `DisappearMidDispatch` (rail never admitted) and `Succeed` (Query knows
  it completed). The case the machinery exists for — `CommitThenLoseResponse`,
  where the rail admitted and Query cannot tell — was untested.
- **F3 (Medium)** — the R8.66 compile-fail expected `E0061` arity. That proves
  the parameter exists; it would pass unchanged if the proof type had a public
  constructor. Q8.7 class: evidence adjacent to the claim.
- **F4 (Low)** — a denial assertion accepting four causes; plus a redundant
  `_private: ()` beside already-private fields.

All four closed in turn 2.

### Standing verification (Gate 8.7 turn 2, reported by name)

Run by the auditor independently of the implementer's report, on the tree as it
stands after turn 2. Every target named; every repeated run reported.

| Target | Result |
|---|---|
| `cargo test -p bank-server --test ordinary_mutations` | **ok — 88 passed** (was 87 pre-gate; +1 is `lost_response_after_commit_safe_retry_emits_nothing_twice`). All 7 `phase8_safe_retry` scenarios green. |
| `cargo test -p worth-query-certification --test compile_certification` | **ok — 14 passed** |
| `cargo test -p worth-query-certification --test application_aftermath_compile_fail` | **ok** — all four safe-retry cases confirmed to *execute*, not skip: `safe_retry_requires_admitted_redispatch`, `admitted_external_redispatch_constructor_is_private`, `redispatch_requires_handle_and_authority` (all should-fail), plus the should-pass twin |
| `cargo test -p worth-query-execution --lib` ×5 | **ok — 579 / 579 / 579 / 579 / 579** (77.5s, 70.1s, 56.1s, 69.1s, 56.1s) |
| `installed_operating_world` / `public_declarative_journeys` / `runtime_public_journeys` | **ok — 313 / 37 / 22** |
| `boundary-check --root .` | **ok** — "Road 1 Cargo topology is valid" |
| `agent-context check` | **ok** (exit 0) |
| `check_workspace_rust_line_caps.sh dirty` | **PASS** |
| `RUSTFLAGS=-Dwarnings cargo check --workspace --all-targets`, `worth-query` | **ok** (exit 0, zero diagnostics) |
| `RUSTFLAGS=-Dwarnings cargo check --workspace --all-targets`, `worth-query-bank-world` | **FAILED — exit 101, 17 diagnostics.** See **Q8.17**. Green after the corrective. |
| `cargo test --workspace --no-fail-fast`, both workspaces | **ok** |

Mechanical residue checks re-run for this gate:

| Check | Result |
|---|---|
| `transport.dispatch(` call sites | **1** (`external_effect/dispatch.rs:102`) — R8.67 |
| `WorthQueryExternalTransportOutcome::` outside `dispatch.rs`/`transport.rs` | **0** — no second classification path |
| `worth_signal` / `WorthSignal` in `application_aftermath/**` | **0** — R8.11 holds; residue test untouched |
| `#[allow(` in `application_aftermath/**` | **0** |
| drain / sweep / relay / backoff / retry-policy nouns in new code | **0** — scope held |

## Exit condition

The historical rule required every `R8.*` row to read `PROVED` and every `Q8.*`
finding to read `CLOSED`. The finish plan supersedes that rule for current
product acceptance because it deliberately reclassifies undo/redo requirements
as provisional rather than falsely proving them.

**Current accepted correction findings:** none open. Q8.25 and Q8.27 are closed;
Q8.26's retained-truth foundation is closed. Q8.21-Q8.24 preserve accepted
recovery/history ownership where stated and provisional undo/redo evidence
otherwise. They are not open Phase 8 blockers and not supported product claims.

Q8.13, Q8.15, and Q8.16 remain closed by corrective Gate 8.7. Q8.12 reopened
because its formerly unnamed timing failure recurred, and is now closed against
the root-correction obligation its row set: the race-prone setup is replaced by
expiry evidence captured before the pin, not by a widened sleep and not by
re-counting greens.

Crash-recovery drain of committed outbox rows is **not a Phase 8 requirement
and is not tracked as a finding.** An in-memory outbox dies with the process it
lived in, so there is no row for a sweep to find; R8.71 publishes that lifetime
as a typed posture on the safe-retry admission, which states the runtime's
actual guarantee rather than deferring an obligation. If a Store-backed Query
runtime later makes durable outbox rows real, that is new scope for the phase
that builds it, not an inheritance from this one.

**Historical note — why no *defect* is carried.** Three rows were, at various
points, recorded as deliberate carries with owners and deadlines:

- **Q8.9** (production-callable registry wipe) — **CLOSED at Gate 8.4 turn 1**
  (instance-scoped registry).
- **Q8.3** (posture construction authority) — was carried as `PARTIAL` with a
  Phase 9 deadline, on the reasoning that the type is unexported so no consumer
  can reach the residual. **CLOSED by the gap-closure corrective:** all six
  successor postures now require `ExternalEffectPostureEvidence` in the variant
  shape, so R8.22 is proved on unrepresentability rather than on export policy.
- **PB1, PB2, PB4** — were recorded in the Bank gap ledger with Phase 9 owners
  and a pre-facade-snapshot deadline. **CLOSED by the same corrective:** the
  platform slot is `Unit` / `ApplicationCapabilityMagnitudeDimension`, and the
  ordinary branch has one owner with a mechanical residue guard.

Each of those carries was defensible on its own terms — named, owned, dated,
and justified. Recording them was still the wrong outcome. A recorded defect
with a deadline in a later phase is documentation, not closure; §10.5's standing
check applies to a phase's own findings exactly as it applies to a gate's entry
conditions. *If the answer is "a later phase will handle it," there is a gap
here.*

The one boundary that remains is not a carry but a genuine dependency, and it
is mechanically enforced: §11 row 11's session/queue half is proved unreachable
by `recovery_handle_owns_no_session_or_queue_resource_types`, which reads
`handle.rs`, asserts the handle owns none of `SessionLease`, `ProviderSession`,
`CauseQueue`, `WorkflowGraphQueue`, or `user_node`, and asserts the
`bank-user-node` crate is absent from disk. **That test fails the moment Bank
Phase 5 supplies those types**, forcing the row to be revisited rather than
trusting anyone to remember it.

## Gate 8.4 — Fresh undo (CLOSED — turn 3)

Production owner: `worth-query-execution/.../application_aftermath/undo_*`,
C2 on `WorthQueryPrimaryMutationWorkEvidence`, instance
`WorthQueryRecoveryHandleRegistry` on the application runtime. Bank assembly:
`admit_undo_*_recovery` / `progress_undo_commit_recovery` /
`progress_undo_recorded_inverse`. Courtroom: `phase8_undo_denials`,
`phase8_undo_denials_lifecycle`, `phase8_undo_money`,
`phase8_undo_recorded_inverse`.

**Auditor confirmation (2026-08-06).** Closure rests on checks re-run by the
auditor against an oracle written before turn 1 existed. Confirmed
independently: bank `ordinary_mutations` **62 × 2**; consumers
**313 / 37 / 22**; `compile_certification` **14**; `worth-query-execution --lib`
**562 × 3**; no touched file over 400 lines; no `*_for_replay` residue.

Four claims were verified by reading rather than by result, each chosen because
it was a place a green test could have been manufactured:

- **The double-entry oracle is genuinely independent.** `independent_oracle_agrees`
  sums `AccountActivityItem` amounts with its own arithmetic and asserts
  absolute values; it never asks production accounting what to expect, so it
  cannot degenerate into production agreeing with itself.
- **"Exactly one compensating transfer" counts committed journal IDs**, not
  requests — the ledger attack's predicted failure, which did not materialize.
- **Eight distinct `WorthQueryUndoDenialKind` values, each asserted exactly
  once**, each reached through the production undo entry, each with a panic on
  any unexpected denial shape.
- **N10 holds structurally, not just observationally.**
  `deny_irreversible_undo_attempt` is a pure function over the installed
  aftermath contract — no graph access, no provider, no transaction reachable —
  so the five irreversible causes cannot open a transaction at all. That is
  stronger than the before/after snapshot the tests assert.

**Turn 2's defect and what it teaches.** Turn 2 shipped
`eight_undo_denial_kinds_are_distinguishable`, which built an array of the eight
enum variants, deduplicated it, and asserted `len() == 8` — a property the
compiler already guarantees. It could not fail for any reason connected to undo,
yet it occupied the slot where R8.39's proof belonged, so the row read as
covered. Three sibling tests proved operations *install* as irreversible (an
installation property already owned by R8.21) under names implying they proved
undo *attempts* write nothing.

This is a different defect class from Gate 8.3's. There, the defects were
**callers supplying what should have been evidence** — visible in a signature.
Here they were **tests asserting something adjacent to the claim** — visible
only by reading the body against the requirement. Both survive a reviewer who
checks that a green test exists with a matching name. The oracle catches the
second class only because its rows state *what evidence is needed*, not what the
requirement claims: "all eight causes distinguishable" would have been marked
satisfied by the dedup test; "each scenario reaching its own cause through the
production path" would not.

**Turn 3 landed.** Deleted enum-dedup theatre. Eight R8.39 scenarios through
production undo admission/progression with typed public causes and
before/after journal+activity equality; irreversible denies via
`deny_irreversible_undo_attempt` before recovery mint (recovery is
NotAdmitted for irreversible); Stale/AlreadyConsumed mapped on the undo
entry from Expired/AlreadyTerminal; Conflicted at reverse-journal progress
with unchanged graph. RecordedInverse end-to-end: freeze retains `Status`
pre-image into the receipt, admission consumes it, restore writes the
retained prior through ordinary `compare_and_commit_application`;
compensation journal lane denies the same contract
(`CorrectionNotAdmitted`).

| ID | Status | Evidence |
|---|---|---|
| R8.2 consumption | **PROVED** | demand on EffectProgram → retention at commit; admit consumes; restore uses retained `Status` bytes (`phase8_undo_recorded_inverse`) |
| R8.9 | **PROVED** | catalog resolve at install; three rejection tests (turn 2) |
| R8.36–R8.37 | **PROVED by correction slice 6** | Exact original input and canonical identity carry through receipt and handle; pre-image target is bound to the touched set; typed Bank continuations own the derived compensation or inverse target. Public progression accepts no caller-authored correction semantics. |
| R8.38 | **PROVED** | `phase8_undo_money` committed journal rows + independent oracle |
| R8.39 | **PROVED** | eight production scenarios + no-write snapshots + positive twins |
| R8.40 | **PROVED** | 1/1/0 counters; posting/lineage fan-out twin (turn 2) |
| R8.41 | **PROVED** | negative + Foundational descriptive positive (turn 2) |

**Gate 8.4 remains reopened only by Q8.26.** Preserved evidence includes
exactly one compensating transfer with originals intact (`phase8_undo_money`),
current-policy denial after world drift, inverse capability progression and the
separate RecordedInverse→compensation-lane denial
(`phase8_undo_recorded_inverse`), irreversible/stale/conflicted/terminal
denials, and the independent double-entry oracle. Slice 5 deliberately removes
the old borrowed-handle retry: ordinary mutation idempotency owns retry after
one admission; a second undo admission with the same handle cannot compile.

**Still outside Gate 8.4 (not owed here).** R8.40 receipt-backed fan-out twin
remains optional polish.

---

## Gate 8.5 — Fresh redo intent and linear lineage (historical, superseded)

The section below records what the original Gate 8.5 implementation proved at
the time. It is not current closure authority. Q8.23 originally reopened redo
meaning and is now superseded by the correction-slice-7 ledger above. Q8.24 is
closed by the correction-slice-8 ledger above: the replacement is Query-owned
semantic aftermath causality co-committed through Relational history authority,
not an instance-local Query chain and not Runtime Bridge legality authority.

Every implementation name below is retained only as historical evidence and
is absent from the corrected production surface.

Production owner: `worth-query-execution/.../application_aftermath/{redo_*,linear_lineage}`,
instance-local `linear_lineage` on `WorthQueryPrimaryGraphApplicationRuntime`.
Bank assembly: `seal_proved_undo` / `derive_redo_intent` /
`admit_redo_disbursement_recovery` / `progress_redo_disbursement` /
`record_original_lineage` / `record_undo_lineage`. Courtroom:
`phase8_redo_denials`, `phase8_redo_world_drift`, `phase8_redo_support`;
cross-gate `redo_through_undo_handle_rail_and_aftermath`.

**Centre claim (R8.45).** Invalidation-on-divergence is
`WorthQueryLinearLineageChain::evaluate_divergence` — linear-lane policy —
consumed by `admit_redo`. `WorthQueryRedoIntent` records `bound_linear_head`
descriptively and has **no** method that consults the live head. A 9.18
rebasing lane can reuse `WorthQueryRedoIntent::derive` / the intent type
unchanged. The parent-causality edge type
(`WorthQueryAftermathParentCausalityEdge`) does not encode linearity; no empty
branch placeholder was added (R8.53).

**R8.43 / A9.** Proved undo is derivation only (`WorthQueryProvedUndo`). Copied
intent is re-derived and compared; duplicate redo is read from committed
lineage rows — neither is a caller-supplied bool. **Turn 2** adds the
world-drift twin of Gate 8.4 A10: after proved undo, expire the grant, leave
the intent honest, and deny `NewlyUnauthorized` at the Bank boundary with
intent equality afterwards (`newly_unauthorized_after_grant_expiry_with_honest_intent`).
**X2** adds handle clock-expiry → `Stale` through the same disbursement path
(`stale_after_handle_expiry_with_honest_intent`). Positive twins admit without
the drift.

| ID | Status | Evidence |
|---|---|---|
| R8.42 | **PROVED** | Descriptive intent; getters only; no authority/replay; residue check excludes `worth_query_replay` from redo/lineage modules |
| R8.43 | **PROVED** | Fresh `admit_redo` + bank re-admission; **world-drift** honest-intent `NewlyUnauthorized` (A9) and `Stale` (X2) through Bank |
| R8.44 | **PROVED** | Committed lineage rows counted (original/undo/redo); one parent edge per successor |
| R8.45 | **PROVED** | Divergence policy on chain; intent unchanged under divergence; edge admits leaf addition structurally |
| R8.46 | **PROVED** | Lowering → `SingularContinuity` only; six forbidden postures each deny with positive twin |

**Exit proof.** Lawful redo (bank); stale (Bank handle clock-expiry after
proved undo + positive twin); newly unauthorized (Bank grant expiry after
proved undo + positive twin; unit `map_recovery_denial` retained); copied
intent; foreign principal; changed operation meaning; duplicate redo;
divergence (ordinary intervening + intervening redo; unit also covers
intervening undo). Each cause reached through production `admit_redo` / bank
assembly — not enum dedup.

**Standing verification (turn 2, reported by name).**

| Check | Result |
|---|---|
| `cargo test -p bank-server --test ordinary_mutations` | **72** |
| `installed_operating_world` | **313** |
| `public_declarative_journeys` | **37** |
| `runtime_public_journeys` | **22** |
| `cargo test -p worth-query-certification --test compile_certification` | **14** |
| `cargo test -p worth-query-execution --lib` × 5 | **578 × 5** |
| `RUSTFLAGS=-Dwarnings cargo check` (execution + bank-server) | clean |
| `boundary-check` / `agent-context check` | exit 0 |
| Dirty line-cap (`check_workspace_rust_line_caps.sh dirty`) | PASS |

**Finding owed at Gate 8.6 residue sweep (not a Gate 8.5 defect).**
`WorthQueryRedoIntent::derive` (and peers `undo_intent`,
`external_effect/correlation`, `dispatch`) return
`Result<Self, &'static str>` for internal digest-preparation failures, while
every consumer-visible Phase 8 denial is a typed kind
(`WorthQueryRedoDenialKind`, `WorthQueryUndoDenialKind`,
`WorthQueryRecoveryHandleDenialKind`). These string-typed internal errors
predate this gate and are inconsistent with the phase's own discipline; they
must not be rediscovered a third time — schedule typed internal error kinds
(or an explicit internal-only exception) at 8.6's residue sweep.

**Residual.** Gate 8.6 not started.

---

## Gate 8.6 — Bank aftermath cutover, publication, certification (CLOSED — turn 2)

Production evidence: Bank `ordinary_mutations` courtroom modules
`phase8_publication_noninterference`, `phase8_ordinary_commit_cost`,
`phase8_fanout_courtroom`, `phase8_residue`; HTTP
`protocol_boundary::r8_49_*`; typed
`WorthQueryAftermathDerivationFailure` on undo/redo/external-effect identity
derivation; PB1/PB2/PB4 entered in Bank front-door gap ledger.

### R8.47–R8.51

| ID | Status | Evidence |
|---|---|---|
| R8.47 | **PROVED** | Host `primary_graph` exports admit/progress/recovery/undo/redo surfaces; installation exports `InstalledAftermathNextActionContract`; mechanical facade inventory (`phase8_residue::r8_47_*`). |
| R8.48 | **PROVED** | Paired freeze worlds differing only in protected foreign-account `AccountStatus` (vacuity asserted); equal published posture, next-action discriminant, derived undo request, lineage row/edge counts; undo admits in both (`phase8_publication_noninterference`). |
| R8.49 | **PROVED** | `bank-http-adapter` source scan: no recovery/undo/redo/aftermath decision vocabulary (`protocol_boundary::r8_49_*`). |
| R8.50 | **PROVED** | **Removed** (not privatized): monolith `operation_aftermath` directory absent; bank `EstateAftermath` enum absent; no bank-server Phase-8 `rollback` door (`phase8_residue::r8_50_*`). Managed-run workflow cleanup rollback is not Phase-8 aftermath authority and was not deleted. |
| R8.51 | **PROVED** | Transport installed and live asserted first; ordinary `send_money` then asserts `external_dispatch` / `undo_admission` / `redo_admission` all 0/0/0 (`phase8_ordinary_commit_cost`). |

### Courtroom §11 — traceability map

Spec §11 names fifteen scenarios that must exist and must fail closed. Each
row below names the test(s) that satisfy it. Honest gaps are recorded rather
than stretched.

| §11 | Scenario (abbrev.) | Satisfying test(s) | Notes |
|---|---|---|---|
| 1 | Compensating reversal; both journals preserved; independent oracle | `phase8_undo_money::disburse_estate_undo_commits_compensating_debit_and_credit_journals` (`independent_oracle_agrees`) | Oracle sums activity amounts; does not ask production accounting. |
| 2 | Same reversal twice — one compensation, not two | same test (equivalent-retry arm asserts journal count stays 2) | |
| 3 | Undo after beneficiary/world conflict — denied by *current* policy | `phase8_cross_gate::undo_denies_on_current_policy_after_world_drift_with_honest_receipt` | Receipt left honest; grant clock advanced. |
| 4 | Undo of released/irreversible estate — no `undo` on outcome type | `phase8_undo_denials::released_estate_undo_denies_and_writes_nothing`; compile-fail `irreversible_has_no_undo_method` + source twin `irreversible_next_action_contract_has_no_undo_method_in_source` | Runtime denial + type-level absence (R8.21). |
| 5 | Lost response after commit — idempotency returns same result, moves no money | `phase8_cross_gate::lost_response_recovery_through_real_rail_and_aftermath`; money twin: `disburse_estate` / `freeze_account` equivalent-retry `AlreadyCommitted` paths | Lost-response path is NotifyDeath (no money moved). Money non-double-apply is ordinary idempotent retry. |
| 6 | Ack-never-complete; complete-after-timeout; ack-twice — distinct postures, none `Completed` until external authority | `external_effect_dispatch::a_succeeding_rail_completes_and_no_fault_ever_reports_completion` (ack-only / slow-completion / duplicate-ack arms); `late_reconciliation_observes_rail_completion_without_dispatching_again`; `a_duplicate_acknowledgement_never_advances_the_posture_twice`; rail exit_proof twins | Integration/e2e through real separate-process rail. |
| 7 | Redo after intervening divergent operation — invalidated; journals/lineage intact | `phase8_redo_denials::divergence_by_intervening_ordinary_operation_invalidates`; `divergence_by_intervening_redo_invalidates` | |
| 8 | Redo after principal capability expired — denied on fresh admission | `phase8_redo_world_drift::newly_unauthorized_after_grant_expiry_with_honest_intent` | Honest intent retained; positive twin admits without expiry. |
| 9 | Copied recovery handle — foreign principal / runtime / equal-ordinal branch | `recovery_progression::binding_axis_tests::{foreign_principal,foreign_runtime,foreign_branch_equal_ordinal}_drift_denies_distinctly`; Bank twin: `phase8_redo_denials::foreign_principal_redo_denies` | Three distinct denial kinds with positive twins. |
| 10 | Handle expiry mid-inspection; disposal then transition attempt | `phase8_recovery_expiry::clock_advanced_expiry_terminalizes_and_denies_fresh_admission`; `phase8_cross_gate::dispose_and_wire_readmission_deny_for_distinct_reasons`; `phase8_undo_denials_lifecycle::stale_expired_handle_undo_denies_and_writes_nothing` | |
| 11 | Crashed user node mid-recovery — no leaked handle / session / queue | `phase8_recovery_terminal_leak::{consumed_reconcile,disposed,expired_clock,force_terminated}_path_leaves_no_live_registry_handles`; `recovery_handle_owns_no_session_or_queue_resource_types` | **Handle half PROVED** (four terminal paths). **Session/queue half — evidenced boundary:** mid-recovery owner `WorthQueryRecoveryHandle` holds only identity/slot/registry/binding/work (no session or queue field). Provider session leases close before `open_commit_recovery`. User-node session/queue types do not exist (`bank-user-node` crate absent; front-door ledger: no same-process substitute). Not a deferral of a probeable in-process resource. |
| 12 | Fan-out twins 10 vs 1000 postings, 1 vs 100 lineage — §8 counters unchanged | `phase8_fanout_courtroom::courtroom_row_12_lineage_fanout_twins_leave_section_8_counters_unchanged`; unit twins `undo_intent_identity_invariant_across_posting_and_lineage_fanout`, `redo_intent::tests::fanout_does_not_change_identity` | Bank lineage 1 vs 100 + unit posting/lineage slope. |
| 13 | External effect + `Reversible` — rejected at installation (R8.57) | `published_posture::external_effect_rejects_reversible_with_named_cause` + positive twin `external_effect_allows_compensation_twin` | |
| 14 | No domain mutation + one external effect — dispatch commits; lost response resolves; retry emits once (R8.55) | `mutation_free_external_effect::mutation_free_external_effect_co_commits_outbox_recovers_lost_response_once` | Real `bank-external-rail`. Asserts `co_committed_dispatch_outbox()`, unchanged `NotificationRequested` status, scaffolding-only `changed_record_count()==2`, lost-response posture, `AlreadyCommitted` retry, rail attempts == 1. Closes Gate 8.2 O2 / **Q8.11**. |
| 15 | Two ops identical except authority axis; two identical except mechanism axis — four distinct installed identities (R8.52) | `published_posture::authority_axis_drift_changes_installed_identity`; `mechanism_axis_drift_changes_installed_identity`; Bank two-axis live twin: `phase8_recovery_mechanism::recorded_inverse_aftermath_admits_reconcile_and_denies_compensate` | |

### `&'static str` residue

Closed: `WorthQueryAftermathDerivationFailure` replaces string errors on
undo/redo intent and external-effect correlation/dispatch derivation.

### Standing verification (turn 1, reported by name)

| Check | Result |
|---|---|
| `cargo test -p bank-server --test ordinary_mutations` | **79** |
| `installed_operating_world` | **313** |
| `public_declarative_journeys` | **37** |
| `runtime_public_journeys` | **22** |
| `cargo test -p worth-query-certification --test compile_certification` | **14** |
| `cargo test -p worth-query-execution --lib` × 5 | **578 × 5** |
| `RUSTFLAGS=-Dwarnings cargo check` (execution + bank-server) | clean |
| `boundary-check` / `agent-context check` | exit 0 |
| Dirty line-cap (`check_workspace_rust_line_caps.sh dirty`) | PASS |

### Turn 2 (closing) — durable artifact repair

- Exit condition rewritten: true under Q8.3 deliberate carry; "two PARTIAL /
  earlier-than-close" contradiction removed.
- §11 courtroom traceability map added (fifteen rows → named tests; row 14 O2
  and row 11 multi-node session/queue noted honestly).
- Standing verification re-run below.

### Standing verification (turn 2, reported by name)

| Check | Result |
|---|---|
| `cargo test -p bank-server --test ordinary_mutations` | **79** |
| `cargo test -p worth-query --test installed_operating_world` | **313** |
| `cargo test -p worth-query --test public_declarative_journeys` | **37** |
| `cargo test -p worth-query --test runtime_public_journeys` | **22** |
| `cargo test -p worth-query-certification --test compile_certification` | **14** |
| `cargo test -p worth-query-execution --lib` × 5 | **578, 578, 578, 578, 578** |
| `RUSTFLAGS=-Dwarnings cargo check` (execution + bank-server) | clean |
| `boundary-check` / `agent-context check` | exit 0 |
| Dirty line-cap (`check_workspace_rust_line_caps` dirty; PowerShell equivalent — bash unavailable) | PASS |

### Turn 3 — Q8.11 / Gate 8.2 O2 corrective (Phase 8 closes)

- Finding **Q8.11** recorded and **CLOSED**: Gate 8.2 O2 was marked PROVED on
  O1 evidence; turn 2's row 14 gap was the surfacing event.
- Bank `RetransmitDeathNotice` (emit-only, no domain field writes) + Query
  empty provisional program / empty invariant state-load admission for
  emit-only commits.
- E2e through real `bank-external-rail`:
  `mutation_free_external_effect::mutation_free_external_effect_co_commits_outbox_recovers_lost_response_once`.
- R8.25 / R8.55 evidence columns and courtroom row 14 updated to name that
  test. Row 11 Bank Phase 5 gap left honest.
- Note on count: production `changed_record_count()` includes Query
  scaffolding (idempotency + outbox), so mutation-free asserts count **2**
  with unchanged domain status — not literal zero.

### Standing verification (turn 3, reported by name)

| Check | Result |
|---|---|
| `cargo test -p bank-server --test ordinary_mutations` | **80** |
| `cargo test -p worth-query --test installed_operating_world` | **313** |
| `cargo test -p worth-query --test public_declarative_journeys` | **37** |
| `cargo test -p worth-query --test runtime_public_journeys` | **22** |
| `cargo test -p worth-query-certification --test compile_certification` | **14** |
| `cargo test -p worth-query-execution --lib` × 5 | **578, 578, 578, 578, 578** |
| `RUSTFLAGS=-Dwarnings cargo check` (execution + bank-server) | clean |
| `boundary-check` / `agent-context check` | exit 0 |
| Dirty line-cap (PowerShell equivalent — bash/WSL unavailable) | PASS |

### Phase 8 closure

Every `R8.*` row above is **PROVED**. Every `Q8.*` finding is **CLOSED**.
PB1/PB2/PB4 and Q8.3 are **CLOSED** by the Phase 8 gap-closure corrective
(zero deliberate carries). §11 row 11 records an evidenced session/queue
boundary (no recovery-owned or user-node session/queue types exist to probe).

**Historical Gate 8.6 result:** Phase 8 was marked closed at this point in the
archive. The current correction ledger at the top of this document supersedes
that result and keeps C8 open until the final frozen-source reviews pass.

---

## Gap-closure corrective (post Gate 8.6) — recorded gaps closed

Closes the four gaps Phase 8 recorded rather than fixed (PB1, PB2, PB4, Q8.3)
and re-grounds §11 row 11.

### Standing verification (gap closure, reported by name)

| Check | Result |
|---|---|
| `cargo test -p bank-server --test ordinary_mutations` | **81** (80 prior + row-11 session/queue boundary test) |
| `cargo test -p worth-query --test installed_operating_world` | **313** |
| `cargo test -p worth-query --test public_declarative_journeys` | **37** |
| `cargo test -p worth-query --test runtime_public_journeys` | **22** |
| `cargo test -p worth-query-certification --test compile_certification` | **14** |
| `cargo test -p worth-query-execution --lib` × 5 | **579, 579, 579, 579, 579** (578 prior + PB4 residue) |
| `RUSTFLAGS=-Dwarnings cargo check` (execution + bank-server) | clean |
| `boundary-check` / `agent-context check` | exit 0 (facade snapshot: `worth_query_currency`→`worth_query_unit`) |
| Dirty line-cap (PowerShell/Python equivalent — bash/WSL unavailable) | PASS |

## Gate 8.4 — Fresh undo (CLOSED — turn 3) archive note

See section above for the closed Gate 8.4 record. The prior closing note that
"Gate 8.5 redo/lineage not started" is superseded by the Gate 8.5 section.

Production owner: `worth-query-execution/.../application_aftermath/undo_*`,
C2 on `WorthQueryPrimaryMutationWorkEvidence`, instance
`WorthQueryRecoveryHandleRegistry` on the application runtime. Bank assembly:
`admit_undo_commit_recovery` / `admit_undo_disbursement_recovery` /
`progress_undo_commit_recovery`. Cross-gate: `phase8_cross_gate` undo scenarios;
money: `phase8_undo_money`.

**Turn 2 landed.** A10 denial arm asserts `CapabilityAuthorizationMissing`.
R8.2: `undo_preimage` retains demand-sliced decision facts into the receipt;
RecordedInverse undo consumes them (no live re-read). R8.9: install resolves
typed `InstalledLoweringCorrespondence` from
`AftermathLoweringCorrespondenceCatalog`; rejects unresolved / wrong-generation
/ mismatched graph participation; slot is diagnostic only. A11/R8.37:
`progress_admitted_undo` + Bank `progress_undo_commit_recovery` hand
compensation through ordinary `commit_reverse_journal` →
`compare_and_commit_application`. R8.38: compensating debit+credit journals,
originals preserved, retry once, independent activity-sum oracle. R8.39:
eight distinguishable denial kinds; irreversible cause classification
(legal/audit/approval/released/escaped); Stale/AlreadyConsumed/Conflicted
mapped from recovery/commit conflict; install-time no-write twin. R8.40:
posting/lineage fan-out twin (10/1 vs 1000/100) yields identical intent
digest and 1/1/0 counters. R8.41: Foundational descriptive positive.

**Turn 2 still-owed table superseded by turn 3 section above.**

**Standing verification (turn 2).** See `_tmp/gate-8-4-turn-2-status.md`.

---

## Gate 8.4 — Fresh undo (turn 1 archive)

Production owner: `worth-query-execution/.../application_aftermath/undo_*`,
C2 on `WorthQueryPrimaryMutationWorkEvidence`, instance
`WorthQueryRecoveryHandleRegistry` on the application runtime. Bank assembly:
`BankIdentityRuntime::admit_undo_commit_recovery`. Cross-gate:
`phase8_cross_gate` undo scenarios.

**Turn 1 landed.** Entry wording / Q8.10 closed. C2 names commit-derived
touched records and undo admission consumes them. Q8.9 cause closed
(instance registry; reset/lock retired). Fresh undo admission derives
request from authority × mechanism axes (not posture name), requires fresh
effect authority, mints one undo intent identity at 1/1/0, populates
`undo_admission`. Cross-gate: undo through handle+rail+aftermath; world-drift
denial with honest receipt. Foundational-only undo denied (R8.41 negative).

**Turn 1 still-owed table superseded by turn 2 section above.**

**Standing verification (turn 1, reported by name).**

| Check | Result |
|---|---|
| `cargo test -p bank-server --test ordinary_mutations` | **51 passed** |
| `installed_operating_world` | **313 passed** |
| `public_declarative_journeys` | **37 passed** |
| `runtime_public_journeys` | **22 passed** |
| `cargo test -p worth-query-certification --test compile_certification` | **14 passed** |
| `cargo test -p worth-query-execution --lib` × 5 | **554 / 554 / 554 / 554 / 554** |
| `RUSTFLAGS=-Dwarnings cargo check` (execution, host, bank-server) | **clean** |
| `boundary-check` | exit 0 |
| `agent-context check` | exit 0 |
| dirty line-cap | **PASS** |
| Production-surface residue | no `reset_for_*` / `lock_for_test`; empty `test-support` feature remains |
