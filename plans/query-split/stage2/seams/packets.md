# Wave 2 seam packets

Base: `1dcb07d44e` (wave 1 committed and measured). Every packet edits without compiling; the integrator merges them in the order below and runs one combined compile-and-fix pass for the wave.
The canonical layering governs every packet: host-supertrait order first, draft stack for ties. Done checks read the host-order seams (`after-wave-1/*/host-order-seams.md` is the starting point).

| id | kind | model | title | seams | depends on | owner calls |
|---|---|---|---|---|---|---|
| W2-01 | item-move | sonnet | Convergence provider port to provider_session | provider_session -> convergence_epoch (2 refs, host order) | — | — |
| W2-02 | item-move | sonnet | Bridge refusal translators to output demand (family E step 1) | application_output_demand -> application_contribution (bridge_denial_kind part) | — | — |
| W2-03 | contract | opus | Producer output evaluation evidence (family E step 2) | application_output_demand -> application_contribution (evaluation part); causal_inspection -> application_contribution | W2-02 (same seam; merge after it) | — |
| W2-04 | contract | opus | Computation retention family (family A) | output_lineage -> application_contribution (23); invariant_projection -> application_contribution (16); application_attempt -> application_contribution (13); application_output_demand -> application_contribution (retention part) | — | 1 |
| W2-05 | merge | opus | Graph layout below authorization (merges §2, layout) | authorization -> primary_graph (layout part); provider_session -> primary_graph (layout part) | — | 1 |
| W2-06 | merge | opus | Application principal below authorization (merges §2, principal) | authorization -> primary_graph (principal and entity identity part) | — | 1 |
| W2-07 | merge | opus | Product operation above the attempt (merges §1) | application_attempt -> product_operation (39 refs: WorthQuerySelectedProductOperation, HistoryProducerInvocation, WorthQueryAdmittedApplicationConditionalDefinition, SharedSelectedProductOperation) | — | — |
| W2-08 | contract | opus | Attempt fact and decision-fact store vocabulary below the provider (family B, host-order direction) | provider -> application_attempt (118); invariant_projection -> application_attempt (75); output_lineage -> application_attempt (57); application_output_demand -> application_attempt (46); application_checkpoint -> application_attempt (43) | — | — |
| W2-09 | contract | opus | Attempt commit and registration vocabulary below the provider (family B, host-order direction) | provider -> application_attempt (commit and registration items); application_output_demand -> application_attempt (commit items); output_lineage -> application_attempt (commit items) | — | — |

Merge order: W2-01, W2-02, W2-05, W2-06, W2-08, W2-09, W2-04, W2-03, W2-07.

## Later waves (not briefed yet)

| work | model | scope | when |
|---|---|---|---|
| proposal-03 | opus | Generic host parameter on the public context types (proposal-03-plan.md §7; rulings: grep gate on impl headers, the 9 public methods above product_operation as extension traits through the consumer facade with imports by script, WorthQueryProgramApplicationRuntime with a plain type parameter recorded as an exception) | after W2-07 (shares step 4) |
| domain seams | opus | provider_session -> primary_graph, application_aftermath -> primary_graph, execution_runtime -> primary_graph (contracts.md §2) | after W2-05, W2-06 |
| merges §2 steps 4-5 | opus | Snapshot lease, inspection target, source-facts allocation; progression steps move up into the primary graph | after W2-05, W2-06 |
| design pass | opus | Host-order seams the designs do not cover (designs assumed the greedy order): output_lineage -> application_output_demand (45), handler -> application_entry (30), application_output_demand -> application_query (30), output_lineage -> invariant_projection (28), provider -> application_query / output_lineage / application_checkpoint, application_inbound -> application_runtime | docs only, can start now |

## Briefs

### W2-01

```text
W2-01 — Convergence provider port to provider_session
Design: merges.md §3 (Recommended structure, Migration 2-4)
Seams: provider_session -> convergence_epoch (2 refs, host order)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/convergence_epoch/domain_provider.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/convergence_epoch/domain_assessment.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/convergence_epoch/comparison.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/convergence_epoch/domain_decision.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/convergence_epoch/provider_families.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/convergence_epoch/identity_validation.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/convergence_epoch/incumbent.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/convergence_epoch/domain_work.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/provider_session/graph_provider/convergence_port/ (new)
Steps:
1. Move the eight port files to provider_session/graph_provider/convergence_port/ (module_move.py; one module per file, sibling of semantic_provider_ports).
2. Constructors the iteration owner calls (WorthQueryConvergenceAssessment::new, WorthQueryRetainedConvergenceCandidateEvidence::new, WorthQueryConvergenceDomainWorkEvidence::empty) become pub(in crate::domain_computation).
3. facade::convergence_epoch re-exports from the new location; consumer names unchanged.
4. Do not move any managed_run unit into convergence_epoch.
Done when:
- Host order, domain level (python3 scripts/query_split/vocabulary_simulation.py workspaces/worth-query/crates/worth-query-execution/src/lib.rs domain_computation --impl-limit 600 --host-order plans/query-split/stage2/ownership/rerun/stack-domain_computation-draft.json --exclude-designed plans/query-split/stage2/seams/contracts.md plans/query-split/stage2/seams/merges.md plans/query-split/stage2/seams/proposal-03-plan.md plans/query-split/stage2/ownership/vocabulary/held-back.md --out <scratch>): no `provider_session -> convergence_epoch` seam; no new seam into provider_session.
- Line caps (dirty) pass; doors regenerated.
Rules:
- Edit without compiling: no cargo. The integrator runs one combined compile-and-fix pass for the wave.
- The canonical layering governs: the host-supertrait order that compiles, the draft stack only to break ties. Cheapness of a cut never decides order or ownership; where a cheap cut and the coherent owner disagree, follow the owner.
- Units move whole: a type with all its impls and court doors. Use scripts/query_split/module_move.py (whole modules) or scripts/query_split/unit_move.py (items, `into` mode for existing modules); every path is rewritten repo-wide, no re-export shim at an old path.
- No visibility wider than pub(in <module>) or pub(crate); never widen to pub outside sealed doors. Sealed authority (`_private` fields) does not move away from its seal.
- The consumer facade keeps every name it exports (facade.rs and facade/*.rs re-export from the new path).
- Regenerate court doors with `python3 scripts/test_topology/src_court_doors.py items --root .` and delete door files no `#[path]` reaches any more.
- Files stay at 400 lines or fewer: `scripts/ci/check_workspace_rust_line_caps.sh dirty`.
- Work in your own worktree and branch; commit only your own paths, explicitly; never `git add -A` at the repository root; never push.
- Secondary edits (import and path lines in files you do not own) are expected; keep them to path changes so the integrator can merge them mechanically.
- Typed refusals keep their variants and exhaustive matches byte for byte.

Validation policy (verbatim):
New validation policy from the user, effective now: treat each check as expensive and batch your changes.

Treat checks that take 3–5 minutes as expensive operations. Batch related fixes, check at stable milestones, skip duplicate checks, and use a default budget of 2–4 blocking checkpoints for a substantial task plus a final verification. Go over that budget only when correctness risk requires it.

Concrete rules:
1. Don't run cargo after a single edit. Apply the whole batch first (script output plus every hand fix you can see), then check once.
2. After one compiler run fails, collect every error from that run and fix them all, including predictable follow-on errors, before you check again. Don't fix one error and recompile.
3. Never re-run a check on a tree that hasn't changed. Record the passing revision or diff hash for each command and reuse it.
4. Never run two cargo builds on the same target dir at once. If you need a long check while you keep editing, run it in the background against a fixed snapshot (a commit or stash), not the moving tree.
5. Between milestones, use the cheap checks: the module_coupling/ownership simulations, the tokenizer dry runs, and a library-only `cargo check -p worth-query-execution`. Run the full `cargo check --workspace --tests --keep-going`, the courts and the lib unit tests only at the end of a wave. Run targeted court tests only at milestones that touch their surface.
6. For the current pg owner-vocabulary batch: if you can, apply the domain batch too before the next check, then do one combined check-and-fix pass.

Put this same policy, word for word, in every wave-2 agent brief. Sub-agents edit without compiling, and only you run the combined compile-and-fix pass for each wave. In your next status report, say how many blocking checks you ran in this wave.
```

### W2-02

```text
W2-02 — Bridge refusal translators to output demand (family E step 1)
Design: contracts.md Family E, step 1
Seams: application_output_demand -> application_contribution (bridge_denial_kind part)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/bridge_denial.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/bridge_execution_denial.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/denial/bridge_conditional.rs (new)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/denial/bridge_execution.rs (new)
Steps:
1. Move bridge_denial.rs and bridge_execution_denial.rs whole to application_output_demand/denial/ as bridge_conditional.rs and bridge_execution.rs; rewrite callers. No trait.
Done when:
- Host order, primary graph (python3 scripts/query_split/vocabulary_simulation.py workspaces/worth-query/crates/worth-query-execution/src/lib.rs domain_computation::primary_graph --impl-limit 600 --host-order plans/query-split/stage2/ownership/rerun/stack-primary_graph-draft.json --exclude-designed plans/query-split/stage2/seams/contracts.md plans/query-split/stage2/seams/merges.md plans/query-split/stage2/seams/proposal-03-plan.md plans/query-split/stage2/ownership/vocabulary/held-back.md --out <scratch>): `bridge_denial_kind` no longer listed under application_output_demand -> application_contribution.
- Line caps (dirty) pass; doors regenerated.
Rules:
- Edit without compiling: no cargo. The integrator runs one combined compile-and-fix pass for the wave.
- The canonical layering governs: the host-supertrait order that compiles, the draft stack only to break ties. Cheapness of a cut never decides order or ownership; where a cheap cut and the coherent owner disagree, follow the owner.
- Units move whole: a type with all its impls and court doors. Use scripts/query_split/module_move.py (whole modules) or scripts/query_split/unit_move.py (items, `into` mode for existing modules); every path is rewritten repo-wide, no re-export shim at an old path.
- No visibility wider than pub(in <module>) or pub(crate); never widen to pub outside sealed doors. Sealed authority (`_private` fields) does not move away from its seal.
- The consumer facade keeps every name it exports (facade.rs and facade/*.rs re-export from the new path).
- Regenerate court doors with `python3 scripts/test_topology/src_court_doors.py items --root .` and delete door files no `#[path]` reaches any more.
- Files stay at 400 lines or fewer: `scripts/ci/check_workspace_rust_line_caps.sh dirty`.
- Work in your own worktree and branch; commit only your own paths, explicitly; never `git add -A` at the repository root; never push.
- Secondary edits (import and path lines in files you do not own) are expected; keep them to path changes so the integrator can merge them mechanically.
- Typed refusals keep their variants and exhaustive matches byte for byte.

Validation policy (verbatim):
New validation policy from the user, effective now: treat each check as expensive and batch your changes.

Treat checks that take 3–5 minutes as expensive operations. Batch related fixes, check at stable milestones, skip duplicate checks, and use a default budget of 2–4 blocking checkpoints for a substantial task plus a final verification. Go over that budget only when correctness risk requires it.

Concrete rules:
1. Don't run cargo after a single edit. Apply the whole batch first (script output plus every hand fix you can see), then check once.
2. After one compiler run fails, collect every error from that run and fix them all, including predictable follow-on errors, before you check again. Don't fix one error and recompile.
3. Never re-run a check on a tree that hasn't changed. Record the passing revision or diff hash for each command and reuse it.
4. Never run two cargo builds on the same target dir at once. If you need a long check while you keep editing, run it in the background against a fixed snapshot (a commit or stash), not the moving tree.
5. Between milestones, use the cheap checks: the module_coupling/ownership simulations, the tokenizer dry runs, and a library-only `cargo check -p worth-query-execution`. Run the full `cargo check --workspace --tests --keep-going`, the courts and the lib unit tests only at the end of a wave. Run targeted court tests only at milestones that touch their surface.
6. For the current pg owner-vocabulary batch: if you can, apply the domain batch too before the next check, then do one combined check-and-fix pass.

Put this same policy, word for word, in every wave-2 agent brief. Sub-agents edit without compiling, and only you run the combined compile-and-fix pass for each wave. In your next status report, say how many blocking checks you ran in this wave.
```

### W2-03

```text
W2-03 — Producer output evaluation evidence (family E step 2)
Design: contracts.md Family E, step 2
Seams: application_output_demand -> application_contribution (evaluation part); causal_inspection -> application_contribution
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/** files defining ProducerOutputEvaluation, ProducerOutputEvaluationState, ComparedProducerOutput, PendingProducerPublication
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/registry/{ready_backing,checkpoint}.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/settlement/observation.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/readiness_delivery.rs
Steps:
1. Lower the evaluation evidence into output demand with the `dyn` ProducerOutputFieldScope the design names; contribution implements the scope.
2. Record the decision in plans/query-split/stage2/seams/packet-notes/W2-03.md (what moved, the trait, every generic kept and why).
Done when:
- Host order, primary graph: application_output_demand -> application_contribution lists only family A retention items (or none once W2-04 lands); causal_inspection -> application_contribution absent.
- No new upward edge in `module_coupling.py report`; line caps pass.
Rules:
- Edit without compiling: no cargo. The integrator runs one combined compile-and-fix pass for the wave.
- The canonical layering governs: the host-supertrait order that compiles, the draft stack only to break ties. Cheapness of a cut never decides order or ownership; where a cheap cut and the coherent owner disagree, follow the owner.
- Units move whole: a type with all its impls and court doors. Use scripts/query_split/module_move.py (whole modules) or scripts/query_split/unit_move.py (items, `into` mode for existing modules); every path is rewritten repo-wide, no re-export shim at an old path.
- No visibility wider than pub(in <module>) or pub(crate); never widen to pub outside sealed doors. Sealed authority (`_private` fields) does not move away from its seal.
- The consumer facade keeps every name it exports (facade.rs and facade/*.rs re-export from the new path).
- Regenerate court doors with `python3 scripts/test_topology/src_court_doors.py items --root .` and delete door files no `#[path]` reaches any more.
- Files stay at 400 lines or fewer: `scripts/ci/check_workspace_rust_line_caps.sh dirty`.
- Work in your own worktree and branch; commit only your own paths, explicitly; never `git add -A` at the repository root; never push.
- Secondary edits (import and path lines in files you do not own) are expected; keep them to path changes so the integrator can merge them mechanically.
- Typed refusals keep their variants and exhaustive matches byte for byte.

Validation policy (verbatim):
New validation policy from the user, effective now: treat each check as expensive and batch your changes.

Treat checks that take 3–5 minutes as expensive operations. Batch related fixes, check at stable milestones, skip duplicate checks, and use a default budget of 2–4 blocking checkpoints for a substantial task plus a final verification. Go over that budget only when correctness risk requires it.

Concrete rules:
1. Don't run cargo after a single edit. Apply the whole batch first (script output plus every hand fix you can see), then check once.
2. After one compiler run fails, collect every error from that run and fix them all, including predictable follow-on errors, before you check again. Don't fix one error and recompile.
3. Never re-run a check on a tree that hasn't changed. Record the passing revision or diff hash for each command and reuse it.
4. Never run two cargo builds on the same target dir at once. If you need a long check while you keep editing, run it in the background against a fixed snapshot (a commit or stash), not the moving tree.
5. Between milestones, use the cheap checks: the module_coupling/ownership simulations, the tokenizer dry runs, and a library-only `cargo check -p worth-query-execution`. Run the full `cargo check --workspace --tests --keep-going`, the courts and the lib unit tests only at the end of a wave. Run targeted court tests only at milestones that touch their surface.
6. For the current pg owner-vocabulary batch: if you can, apply the domain batch too before the next check, then do one combined check-and-fix pass.

Put this same policy, word for word, in every wave-2 agent brief. Sub-agents edit without compiling, and only you run the combined compile-and-fix pass for each wave. In your next status report, say how many blocking checks you ran in this wave.
```

### W2-04

```text
W2-04 — Computation retention family (family A)
Design: contracts.md Family A (one family, one module, per the ruling: not four contracts)
Seams: output_lineage -> application_contribution (23); invariant_projection -> application_contribution (16); application_attempt -> application_contribution (13); application_output_demand -> application_contribution (retention part)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/computation_retention/ (new module)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/partitioned_computation/incremental/{prior_absence,deposit,retained,sealing,carriage,native_reads}.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/registry/semantic_edition.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/read_set/computation_routing.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/read_set/computation_routing/layout.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/output_lineage/retained_computation.rs (PriorRecordSelection)
Steps:
1. New pg::computation_retention below application_attempt; move PriorAbsence with Suppression (beside it, per the ruling), WorthQueryPartitionedComputationFullCause, ComputationInstallation, RetainedBasis, RetainedComputation, SealedComputationRun, SealedComputationRetention, CompletedComputationRetention with CompletedComputationRun, ComputationDeposit, SealedComputationFacts, CheckedCallShape.
2. Impl inversion ProducerEditionAdmission (owned by application_contribution::producer::registry); InstalledProducerEdition moves with from_parts and matches_types.
3. Call contract NativeCarriageReads (dyn, owned by application_attempt::read_set::computation_routing), implemented in incremental/native_reads.rs.
4. output_lineage returns PriorRecordSelection; contribution builds ComputationPrior.
5. Decision point step 3: move the carried-call custody with CompletedComputationRun (the design recommends it).
6. Keep drop order in retain_computation byte for byte; move thread_local test handoffs with the type whose door uses them.
7. Write plans/query-split/stage2/seams/packet-notes/W2-04.md: decisions taken, generics kept (admits_binding stays generic, as designed).
Done when:
- Host order, primary graph: the four seams list no retention item (ComputationPrior in required_context.rs excepted until the owner rules).
- No new upward edge; line caps pass; facade name WorthQueryPartitionedComputationFullCause unchanged.
Owner calls (do not decide these):
- Family A step 4: RequiredOutputDemandContext keeps ComputationPrior by value. Either its two ComputationPrior slots move into the producer context in contribution, or ComputationPrior moves down once output_lineage is reached through PriorRecordSelection. Until ruled, leave required_context.rs as is (one upward reference remains).
Rules:
- Edit without compiling: no cargo. The integrator runs one combined compile-and-fix pass for the wave.
- The canonical layering governs: the host-supertrait order that compiles, the draft stack only to break ties. Cheapness of a cut never decides order or ownership; where a cheap cut and the coherent owner disagree, follow the owner.
- Units move whole: a type with all its impls and court doors. Use scripts/query_split/module_move.py (whole modules) or scripts/query_split/unit_move.py (items, `into` mode for existing modules); every path is rewritten repo-wide, no re-export shim at an old path.
- No visibility wider than pub(in <module>) or pub(crate); never widen to pub outside sealed doors. Sealed authority (`_private` fields) does not move away from its seal.
- The consumer facade keeps every name it exports (facade.rs and facade/*.rs re-export from the new path).
- Regenerate court doors with `python3 scripts/test_topology/src_court_doors.py items --root .` and delete door files no `#[path]` reaches any more.
- Files stay at 400 lines or fewer: `scripts/ci/check_workspace_rust_line_caps.sh dirty`.
- Work in your own worktree and branch; commit only your own paths, explicitly; never `git add -A` at the repository root; never push.
- Secondary edits (import and path lines in files you do not own) are expected; keep them to path changes so the integrator can merge them mechanically.
- Typed refusals keep their variants and exhaustive matches byte for byte.

Validation policy (verbatim):
New validation policy from the user, effective now: treat each check as expensive and batch your changes.

Treat checks that take 3–5 minutes as expensive operations. Batch related fixes, check at stable milestones, skip duplicate checks, and use a default budget of 2–4 blocking checkpoints for a substantial task plus a final verification. Go over that budget only when correctness risk requires it.

Concrete rules:
1. Don't run cargo after a single edit. Apply the whole batch first (script output plus every hand fix you can see), then check once.
2. After one compiler run fails, collect every error from that run and fix them all, including predictable follow-on errors, before you check again. Don't fix one error and recompile.
3. Never re-run a check on a tree that hasn't changed. Record the passing revision or diff hash for each command and reuse it.
4. Never run two cargo builds on the same target dir at once. If you need a long check while you keep editing, run it in the background against a fixed snapshot (a commit or stash), not the moving tree.
5. Between milestones, use the cheap checks: the module_coupling/ownership simulations, the tokenizer dry runs, and a library-only `cargo check -p worth-query-execution`. Run the full `cargo check --workspace --tests --keep-going`, the courts and the lib unit tests only at the end of a wave. Run targeted court tests only at milestones that touch their surface.
6. For the current pg owner-vocabulary batch: if you can, apply the domain batch too before the next check, then do one combined check-and-fix pass.

Put this same policy, word for word, in every wave-2 agent brief. Sub-agents edit without compiling, and only you run the combined compile-and-fix pass for each wave. In your next status report, say how many blocking checks you ran in this wave.
```

### W2-05

```text
W2-05 — Graph layout below authorization (merges §2, layout)
Design: merges.md §2, Recommended structure 1 (graph_layout) and Migration 2
Seams: authorization -> primary_graph (layout part); provider_session -> primary_graph (layout part)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/schema_layout/** -> workspaces/worth-query/crates/worth-query-execution/src/domain_computation/graph_layout/
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/denial.rs (installation refusal) -> graph_layout
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/workflow/schema/** (workflow layout, lower_workflow) -> graph_layout
Steps:
1. Cut the three references that block the move first: denial -> contribution (1), schema_layout -> discovery (2).
2. module_move.py the layout family to domain_computation::graph_layout; WorthQueryPrimaryGraphInstallationDenial and its kinds move unchanged; facade re-exports point at the new path.
3. Place graph_layout below provider_session and authorization in the domain stack (between execution resources and sessions/authority/aftermath, per the design).
Done when:
- Host order, domain level (python3 scripts/query_split/vocabulary_simulation.py workspaces/worth-query/crates/worth-query-execution/src/lib.rs domain_computation --impl-limit 600 --host-order plans/query-split/stage2/ownership/rerun/stack-domain_computation-draft.json --exclude-designed plans/query-split/stage2/seams/contracts.md plans/query-split/stage2/seams/merges.md plans/query-split/stage2/seams/proposal-03-plan.md plans/query-split/stage2/ownership/vocabulary/held-back.md --out <scratch>): authorization -> primary_graph and provider_session -> primary_graph no longer list WorthQueryPrimaryGraphLayout or the installation denial.
- Primary graph level: no new upward edge; line caps pass.
Owner calls (do not decide these):
- merges.md §2 Migration 1: the module name `graph_layout` and its place in the domain stack. The packet uses the designed name unless the owner rules otherwise.
Rules:
- Edit without compiling: no cargo. The integrator runs one combined compile-and-fix pass for the wave.
- The canonical layering governs: the host-supertrait order that compiles, the draft stack only to break ties. Cheapness of a cut never decides order or ownership; where a cheap cut and the coherent owner disagree, follow the owner.
- Units move whole: a type with all its impls and court doors. Use scripts/query_split/module_move.py (whole modules) or scripts/query_split/unit_move.py (items, `into` mode for existing modules); every path is rewritten repo-wide, no re-export shim at an old path.
- No visibility wider than pub(in <module>) or pub(crate); never widen to pub outside sealed doors. Sealed authority (`_private` fields) does not move away from its seal.
- The consumer facade keeps every name it exports (facade.rs and facade/*.rs re-export from the new path).
- Regenerate court doors with `python3 scripts/test_topology/src_court_doors.py items --root .` and delete door files no `#[path]` reaches any more.
- Files stay at 400 lines or fewer: `scripts/ci/check_workspace_rust_line_caps.sh dirty`.
- Work in your own worktree and branch; commit only your own paths, explicitly; never `git add -A` at the repository root; never push.
- Secondary edits (import and path lines in files you do not own) are expected; keep them to path changes so the integrator can merge them mechanically.
- Typed refusals keep their variants and exhaustive matches byte for byte.

Validation policy (verbatim):
New validation policy from the user, effective now: treat each check as expensive and batch your changes.

Treat checks that take 3–5 minutes as expensive operations. Batch related fixes, check at stable milestones, skip duplicate checks, and use a default budget of 2–4 blocking checkpoints for a substantial task plus a final verification. Go over that budget only when correctness risk requires it.

Concrete rules:
1. Don't run cargo after a single edit. Apply the whole batch first (script output plus every hand fix you can see), then check once.
2. After one compiler run fails, collect every error from that run and fix them all, including predictable follow-on errors, before you check again. Don't fix one error and recompile.
3. Never re-run a check on a tree that hasn't changed. Record the passing revision or diff hash for each command and reuse it.
4. Never run two cargo builds on the same target dir at once. If you need a long check while you keep editing, run it in the background against a fixed snapshot (a commit or stash), not the moving tree.
5. Between milestones, use the cheap checks: the module_coupling/ownership simulations, the tokenizer dry runs, and a library-only `cargo check -p worth-query-execution`. Run the full `cargo check --workspace --tests --keep-going`, the courts and the lib unit tests only at the end of a wave. Run targeted court tests only at milestones that touch their surface.
6. For the current pg owner-vocabulary batch: if you can, apply the domain batch too before the next check, then do one combined check-and-fix pass.

Put this same policy, word for word, in every wave-2 agent brief. Sub-agents edit without compiling, and only you run the combined compile-and-fix pass for each wave. In your next status report, say how many blocking checks you ran in this wave.
```

### W2-06

```text
W2-06 — Application principal below authorization (merges §2, principal)
Design: merges.md §2, Recommended structure 1 (application_principal) and Migration 3
Seams: authorization -> primary_graph (principal and entity identity part)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/authenticated_principal/** -> workspaces/worth-query/crates/worth-query-execution/src/domain_computation/application_principal/
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/freshness (WorthQueryDurablePrincipalCurrentness, WorthQueryPrincipalFreshnessEvidence)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/entity_resolution/resolved (WorthQueryApplicationEntityIdentity, WorthQueryResolvedEntity, only if their impls name nothing above)
Steps:
1. Move authenticated_principal whole; move the two freshness evidence types and the entity identity values whole when their impls name nothing above (verify each; stop and note it if not).
2. Facade names unchanged.
Done when:
- Host order, domain level: authorization -> primary_graph no longer lists WorthQueryAuthenticatedPrincipal, the two freshness evidence types or the entity identity values that moved.
- No new upward edge at either level; line caps pass.
Owner calls (do not decide these):
- merges.md §2 Migration 1: the module name `application_principal` and its place in the domain stack.
Rules:
- Edit without compiling: no cargo. The integrator runs one combined compile-and-fix pass for the wave.
- The canonical layering governs: the host-supertrait order that compiles, the draft stack only to break ties. Cheapness of a cut never decides order or ownership; where a cheap cut and the coherent owner disagree, follow the owner.
- Units move whole: a type with all its impls and court doors. Use scripts/query_split/module_move.py (whole modules) or scripts/query_split/unit_move.py (items, `into` mode for existing modules); every path is rewritten repo-wide, no re-export shim at an old path.
- No visibility wider than pub(in <module>) or pub(crate); never widen to pub outside sealed doors. Sealed authority (`_private` fields) does not move away from its seal.
- The consumer facade keeps every name it exports (facade.rs and facade/*.rs re-export from the new path).
- Regenerate court doors with `python3 scripts/test_topology/src_court_doors.py items --root .` and delete door files no `#[path]` reaches any more.
- Files stay at 400 lines or fewer: `scripts/ci/check_workspace_rust_line_caps.sh dirty`.
- Work in your own worktree and branch; commit only your own paths, explicitly; never `git add -A` at the repository root; never push.
- Secondary edits (import and path lines in files you do not own) are expected; keep them to path changes so the integrator can merge them mechanically.
- Typed refusals keep their variants and exhaustive matches byte for byte.

Validation policy (verbatim):
New validation policy from the user, effective now: treat each check as expensive and batch your changes.

Treat checks that take 3–5 minutes as expensive operations. Batch related fixes, check at stable milestones, skip duplicate checks, and use a default budget of 2–4 blocking checkpoints for a substantial task plus a final verification. Go over that budget only when correctness risk requires it.

Concrete rules:
1. Don't run cargo after a single edit. Apply the whole batch first (script output plus every hand fix you can see), then check once.
2. After one compiler run fails, collect every error from that run and fix them all, including predictable follow-on errors, before you check again. Don't fix one error and recompile.
3. Never re-run a check on a tree that hasn't changed. Record the passing revision or diff hash for each command and reuse it.
4. Never run two cargo builds on the same target dir at once. If you need a long check while you keep editing, run it in the background against a fixed snapshot (a commit or stash), not the moving tree.
5. Between milestones, use the cheap checks: the module_coupling/ownership simulations, the tokenizer dry runs, and a library-only `cargo check -p worth-query-execution`. Run the full `cargo check --workspace --tests --keep-going`, the courts and the lib unit tests only at the end of a wave. Run targeted court tests only at milestones that touch their surface.
6. For the current pg owner-vocabulary batch: if you can, apply the domain batch too before the next check, then do one combined check-and-fix pass.

Put this same policy, word for word, in every wave-2 agent brief. Sub-agents edit without compiling, and only you run the combined compile-and-fix pass for each wave. In your next status report, say how many blocking checks you ran in this wave.
```

### W2-07

```text
W2-07 — Product operation above the attempt (merges §1)
Design: merges.md §1, Migration 2-5 (layering is already the host order)
Seams: application_attempt -> product_operation (39 refs: WorthQuerySelectedProductOperation, HistoryProducerInvocation, WorthQueryAdmittedApplicationConditionalDefinition, SharedSelectedProductOperation)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/** files holding the 8 inherent blocks on WorthQuerySelectedProductOperation (workflow preparations)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/workflow_preparation/ (new)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/conditional_definition/application_change.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/shared_query_basis.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/navigation/output_authority/producer_invocation.rs
Steps:
1. Move the 8 workflow-preparation blocks into product_operation/workflow_preparation/ (one file per program); the attempt functions take the parts they need (snapshot_lease, product, application_basis), not the context.
2. Hand moves: WorthQueryAdmittedApplicationConditionalDefinition, SelectedQueryOrigin, SharedSelectedProductOperation to application_attempt (whole units). WorthQuerySelectedProductOperation itself stays in product_operation (ruling).
3. Lower HistoryProducerInvocation into application_attempt/provider_execution/entry/ with HistoryAuthorityLifetime, or a lower-owned &dyn HistoryOutputAuthority if the lifetime cannot move.
4. Singles: the two denials move into the attempt if their impls name nothing above, each in its own file beside its users (application_attempt/denial is W2-08's primary file this wave); ObservationRoomScopedOps becomes an ApplicationAttemptHost method.
5. Write plans/query-split/stage2/seams/packet-notes/W2-07.md with what moved and which proposal-03 step 4 work this leaves.
Done when:
- Host order, primary graph: no `application_attempt -> product_operation` seam.
- No new upward edge; line caps pass.
Rules:
- Edit without compiling: no cargo. The integrator runs one combined compile-and-fix pass for the wave.
- The canonical layering governs: the host-supertrait order that compiles, the draft stack only to break ties. Cheapness of a cut never decides order or ownership; where a cheap cut and the coherent owner disagree, follow the owner.
- Units move whole: a type with all its impls and court doors. Use scripts/query_split/module_move.py (whole modules) or scripts/query_split/unit_move.py (items, `into` mode for existing modules); every path is rewritten repo-wide, no re-export shim at an old path.
- No visibility wider than pub(in <module>) or pub(crate); never widen to pub outside sealed doors. Sealed authority (`_private` fields) does not move away from its seal.
- The consumer facade keeps every name it exports (facade.rs and facade/*.rs re-export from the new path).
- Regenerate court doors with `python3 scripts/test_topology/src_court_doors.py items --root .` and delete door files no `#[path]` reaches any more.
- Files stay at 400 lines or fewer: `scripts/ci/check_workspace_rust_line_caps.sh dirty`.
- Work in your own worktree and branch; commit only your own paths, explicitly; never `git add -A` at the repository root; never push.
- Secondary edits (import and path lines in files you do not own) are expected; keep them to path changes so the integrator can merge them mechanically.
- Typed refusals keep their variants and exhaustive matches byte for byte.

Validation policy (verbatim):
New validation policy from the user, effective now: treat each check as expensive and batch your changes.

Treat checks that take 3–5 minutes as expensive operations. Batch related fixes, check at stable milestones, skip duplicate checks, and use a default budget of 2–4 blocking checkpoints for a substantial task plus a final verification. Go over that budget only when correctness risk requires it.

Concrete rules:
1. Don't run cargo after a single edit. Apply the whole batch first (script output plus every hand fix you can see), then check once.
2. After one compiler run fails, collect every error from that run and fix them all, including predictable follow-on errors, before you check again. Don't fix one error and recompile.
3. Never re-run a check on a tree that hasn't changed. Record the passing revision or diff hash for each command and reuse it.
4. Never run two cargo builds on the same target dir at once. If you need a long check while you keep editing, run it in the background against a fixed snapshot (a commit or stash), not the moving tree.
5. Between milestones, use the cheap checks: the module_coupling/ownership simulations, the tokenizer dry runs, and a library-only `cargo check -p worth-query-execution`. Run the full `cargo check --workspace --tests --keep-going`, the courts and the lib unit tests only at the end of a wave. Run targeted court tests only at milestones that touch their surface.
6. For the current pg owner-vocabulary batch: if you can, apply the domain batch too before the next check, then do one combined check-and-fix pass.

Put this same policy, word for word, in every wave-2 agent brief. Sub-agents edit without compiling, and only you run the combined compile-and-fix pass for each wave. In your next status report, say how many blocking checks you ran in this wave.
```

### W2-08

```text
W2-08 — Attempt fact and decision-fact store vocabulary below the provider (family B, host-order direction)
Design: contracts.md Family B, Direction check: under the host order the seam is provider -> attempt and the design flips: lower the attempt-record vocabulary the provider store holds. Owner stays the attempt: the vocabulary goes to vocabulary::application_attempt (or a module the attempt owns below the provider).
Seams: provider -> application_attempt (118); invariant_projection -> application_attempt (75); output_lineage -> application_attempt (57); application_output_demand -> application_attempt (46); application_checkpoint -> application_attempt (43)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/fact/**
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/retained_decision_facts/**
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/denial.rs and denial/**
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/read_set/application_projection.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/read_set/original_read.rs
Steps:
1. Items: WorthQueryApplicationObservedFact, WorthQueryApplicationFactKey, StoreDenial, StorageControl, AdmittedAdjacencyEndpoints, WorthQueryApplicationAttemptDenial (with its kind), CompletedFactProjection, CompletedOriginalRead.
2. They were not vocabulary-movable for one reason each (see after-wave-1/primary_graph/host-order-seams.md `why`): ObservedFact is an enum with more than 600 lines of impls, StoreDenial is held by the attempt denial impl, CompletedFactProjection is held by ObservedFact and StoreDenial. Split behaviour from data: the data types and their pure impls go down; impls that drive attempt behaviour stay as extension traits owned by the attempt (iteration law 7: shells over concrete code).
3. Write plans/query-split/stage2/seams/packet-notes/W2-08.md: the design (what goes down, which traits stay, why each is dyn or generic) before editing, then what was done.
Done when:
- Host order, primary graph: the five seams no longer list these items.
- No new upward edge; line caps pass; facade names unchanged.
Rules:
- Edit without compiling: no cargo. The integrator runs one combined compile-and-fix pass for the wave.
- The canonical layering governs: the host-supertrait order that compiles, the draft stack only to break ties. Cheapness of a cut never decides order or ownership; where a cheap cut and the coherent owner disagree, follow the owner.
- Units move whole: a type with all its impls and court doors. Use scripts/query_split/module_move.py (whole modules) or scripts/query_split/unit_move.py (items, `into` mode for existing modules); every path is rewritten repo-wide, no re-export shim at an old path.
- No visibility wider than pub(in <module>) or pub(crate); never widen to pub outside sealed doors. Sealed authority (`_private` fields) does not move away from its seal.
- The consumer facade keeps every name it exports (facade.rs and facade/*.rs re-export from the new path).
- Regenerate court doors with `python3 scripts/test_topology/src_court_doors.py items --root .` and delete door files no `#[path]` reaches any more.
- Files stay at 400 lines or fewer: `scripts/ci/check_workspace_rust_line_caps.sh dirty`.
- Work in your own worktree and branch; commit only your own paths, explicitly; never `git add -A` at the repository root; never push.
- Secondary edits (import and path lines in files you do not own) are expected; keep them to path changes so the integrator can merge them mechanically.
- Typed refusals keep their variants and exhaustive matches byte for byte.

Validation policy (verbatim):
New validation policy from the user, effective now: treat each check as expensive and batch your changes.

Treat checks that take 3–5 minutes as expensive operations. Batch related fixes, check at stable milestones, skip duplicate checks, and use a default budget of 2–4 blocking checkpoints for a substantial task plus a final verification. Go over that budget only when correctness risk requires it.

Concrete rules:
1. Don't run cargo after a single edit. Apply the whole batch first (script output plus every hand fix you can see), then check once.
2. After one compiler run fails, collect every error from that run and fix them all, including predictable follow-on errors, before you check again. Don't fix one error and recompile.
3. Never re-run a check on a tree that hasn't changed. Record the passing revision or diff hash for each command and reuse it.
4. Never run two cargo builds on the same target dir at once. If you need a long check while you keep editing, run it in the background against a fixed snapshot (a commit or stash), not the moving tree.
5. Between milestones, use the cheap checks: the module_coupling/ownership simulations, the tokenizer dry runs, and a library-only `cargo check -p worth-query-execution`. Run the full `cargo check --workspace --tests --keep-going`, the courts and the lib unit tests only at the end of a wave. Run targeted court tests only at milestones that touch their surface.
6. For the current pg owner-vocabulary batch: if you can, apply the domain batch too before the next check, then do one combined check-and-fix pass.

Put this same policy, word for word, in every wave-2 agent brief. Sub-agents edit without compiling, and only you run the combined compile-and-fix pass for each wave. In your next status report, say how many blocking checks you ran in this wave.
```

### W2-09

```text
W2-09 — Attempt commit and registration vocabulary below the provider (family B, host-order direction)
Design: contracts.md Family B, Direction check (flipped), with Family B step 1 re-read for the host order
Seams: provider -> application_attempt (commit and registration items); application_output_demand -> application_attempt (commit items); output_lineage -> application_attempt (commit items)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/**
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/idempotency.rs and idempotency/**
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_binding/registration/**
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/effect_program/output_correspondence.rs and output_correspondence/**
Steps:
1. Items: WorthQueryApplicationCommitReceipt, WorthQueryApplicationCommittedChanges, WorthQueryApplicationUncommitted, WorthQueryApplicationCommitDenialStage, WorthQueryCommittedProductPublication, WorthQueryApplicationIdempotencyBinding, WorthQueryPrimaryGraphApplicationAttempt, WorthQueryApplicationOutputCorrespondence.
2. provider_registration.rs is an inherent `impl WorthQueryPrimaryGraphProvider` block inside the attempt. Under the host order the provider is below the attempt, so moving the block into provider (Family B step 1 as written) would point provider at attempt types. Decide with the owner rule: the provider keeps what its store owns; the attempt calls it through ProviderHost; record the choice.
3. Sealed authority stays with its seal (WorthQueryCommittedProductPublication and the commit receipt were held back in wave 1 because their impls name their family): split data from behaviour as in W2-08.
4. Write plans/query-split/stage2/seams/packet-notes/W2-09.md (design first, then what was done).
Done when:
- Host order, primary graph: the listed seams no longer name these items.
- No new upward edge; line caps pass; facade names unchanged.
Rules:
- Edit without compiling: no cargo. The integrator runs one combined compile-and-fix pass for the wave.
- The canonical layering governs: the host-supertrait order that compiles, the draft stack only to break ties. Cheapness of a cut never decides order or ownership; where a cheap cut and the coherent owner disagree, follow the owner.
- Units move whole: a type with all its impls and court doors. Use scripts/query_split/module_move.py (whole modules) or scripts/query_split/unit_move.py (items, `into` mode for existing modules); every path is rewritten repo-wide, no re-export shim at an old path.
- No visibility wider than pub(in <module>) or pub(crate); never widen to pub outside sealed doors. Sealed authority (`_private` fields) does not move away from its seal.
- The consumer facade keeps every name it exports (facade.rs and facade/*.rs re-export from the new path).
- Regenerate court doors with `python3 scripts/test_topology/src_court_doors.py items --root .` and delete door files no `#[path]` reaches any more.
- Files stay at 400 lines or fewer: `scripts/ci/check_workspace_rust_line_caps.sh dirty`.
- Work in your own worktree and branch; commit only your own paths, explicitly; never `git add -A` at the repository root; never push.
- Secondary edits (import and path lines in files you do not own) are expected; keep them to path changes so the integrator can merge them mechanically.
- Typed refusals keep their variants and exhaustive matches byte for byte.

Validation policy (verbatim):
New validation policy from the user, effective now: treat each check as expensive and batch your changes.

Treat checks that take 3–5 minutes as expensive operations. Batch related fixes, check at stable milestones, skip duplicate checks, and use a default budget of 2–4 blocking checkpoints for a substantial task plus a final verification. Go over that budget only when correctness risk requires it.

Concrete rules:
1. Don't run cargo after a single edit. Apply the whole batch first (script output plus every hand fix you can see), then check once.
2. After one compiler run fails, collect every error from that run and fix them all, including predictable follow-on errors, before you check again. Don't fix one error and recompile.
3. Never re-run a check on a tree that hasn't changed. Record the passing revision or diff hash for each command and reuse it.
4. Never run two cargo builds on the same target dir at once. If you need a long check while you keep editing, run it in the background against a fixed snapshot (a commit or stash), not the moving tree.
5. Between milestones, use the cheap checks: the module_coupling/ownership simulations, the tokenizer dry runs, and a library-only `cargo check -p worth-query-execution`. Run the full `cargo check --workspace --tests --keep-going`, the courts and the lib unit tests only at the end of a wave. Run targeted court tests only at milestones that touch their surface.
6. For the current pg owner-vocabulary batch: if you can, apply the domain batch too before the next check, then do one combined check-and-fix pass.

Put this same policy, word for word, in every wave-2 agent brief. Sub-agents edit without compiling, and only you run the combined compile-and-fix pass for each wave. In your next status report, say how many blocking checks you ran in this wave.
```
