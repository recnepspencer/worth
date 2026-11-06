# Wave 3 seam packets

Base: `42ae530f5b` (wave 2 integrated and measured: primary graph component 52, 924 references against the host order; domain component 11, 234; stray impls 48 and 14). Every packet edits without compiling; the integrator merges them in the order below and runs one combined compile-and-fix pass for the wave.
The canonical layering governs every packet: host-supertrait order first, draft stack for ties (vocabulary and runtime_host at the bottom). Counts come from `after-wave-2/*/host-order-seams.md`.

| id | kind | model | title | seams | depends on | owner calls |
|---|---|---|---|---|---|---|
| W3-01 | proposal-03 | opus | Proposal 03 part 1: the host parameter, aliases and the facade | product_operation -> application_installation (12 refs); product_operation -> application_runtime (10); application_installation -> application_runtime (7) | — | — |
| W3-02 | proposal-03 | sonnet | Proposal 03 part 2: blocks below product_operation move up into it | application_output_demand -> product_operation (13); entity_resolution -> product_operation (5); stray impls on WorthQuerySelectedProductOperation in the four modules below it (21 of 29) | W3-01 (the host parameter on the selected operation) | — |
| W3-03 | proposal-03 | opus | Proposal 03 part 3: blocks above product_operation become extension traits; the 9 public methods | product_operation -> application_query (78 refs in 27 items); stray impls on WorthQuerySelectedProductOperation in application_query and conditional_operation (8 of 29) | W3-01 | — |
| W3-04 | contract | opus | Domain seams into the primary graph (contracts.md §2) | application_aftermath -> primary_graph (25 refs in 11 items); provider_session -> primary_graph (19 in 11; the snapshot lease, inspection target and source-facts allocation go with W3-05); execution_runtime -> primary_graph (5 in 4); product_publication -> primary_graph (4 in 2) | — | — |
| W3-05 | merge | opus | Merges §2 step 4: snapshot lease, inspection target and source-facts allocation below authorization | provider_session -> primary_graph (lease, inspection target, allocation); authorization -> primary_graph (158 refs in 63 items; the parts these three carry) | — | 1 |
| W3-06 | merge | opus | Merges §2 step 5: attempt-driving progression steps move up into the primary graph | authorization -> primary_graph (158 refs in 63 items: 117 were in operation_progression, 32 elevation_progression, 11 delegation_progression at the design's count) | W3-05 (merge after it; disjoint files) | — |
| W3-07 | item-move | opus | The older stray inherent impls (29) | stray_impls: WorthQueryExecutionRuntime 7, WorthQueryPrimaryGraphBootstrap 5, WorthQueryPrimaryGraphIntegrationHandle 4, WorthQueryExecutionInstallationAuthority 4, InstalledTransportPendingReason 2, WorthQueryExecutionObservation 2, and one each on WorthQueryAdmittedHistoryProgramOutput, DecodedApplicationCheckpoint, WorthQueryPrimaryGraph, WorthQueryApplicationInvariantProjectionAuthority, WorthQueryAdvancementDenial | — | — |
| W3-08 | contract | opus | The retention envelope and its preconditions (design, then implement) | output_lineage -> application_contribution (14 refs in 6 items); invariant_projection -> application_contribution (12 in 7); application_attempt -> application_contribution (7 in 3); application_output_demand -> application_contribution (7 in 5, retention part) | — | 1 |
| W3-09 | contract | opus | The attempt record remainder: commit receipt placement, Uncommitted, the observed fact, CompletedOriginalRead (design, then implement where unblocked) | provider -> application_attempt (45 refs in 14 items); invariant_projection -> application_attempt (33 in 12); application_output_demand -> application_attempt (27 in 5); output_lineage -> application_attempt (23 in 2); application_checkpoint -> application_attempt (14 in 1) | W3-05 for the observed fact and CompletedOriginalRead (implement those only after it merges; design now) | 1 |
| W3-10 | contract | opus | Output-demand evaluation leftovers: PendingProducerPublication, CurrentProof, OwnerPlan (design, then implement) | application_output_demand -> application_contribution (non-retention part); application_checkpoint -> application_output_demand (4 in 3) | — | 2 |
| W3-11 | contract | opus | The resolver contract for authorization (design, then implement) | authorization -> primary_graph (resolution part) | — | — |
| W3-12 | design | opus | Design pass: host-order seams the designs never covered (design only) | output_lineage -> application_output_demand (36 refs in 5); handler -> application_entry (30 in 4); application_output_demand -> application_query (30 in 7); output_lineage -> invariant_projection (28 in 4); provider -> application_query (20 in 5); invariant_projection -> application_entry (19 in 1); provider -> output_lineage (17 in 10); application_discovery -> application_attempt (15 in 9); invariant_projection -> application_query (13 in 7); provider -> application_checkpoint (11 in 3); application_contribution -> conditional_operation (11 in 5); provider -> application_output_demand (11 in 3); the smaller seams in after-wave-2/primary_graph/host-order-seams.md; domain: provider_session -> managed_run (10), domain_evidence_binding -> provider_session (2), operation_binding -> provider_session (2), artifact_owner -> operation_binding (1), execution_runtime -> product_publication, managed_run, invalidation_equivalence_control (1 each) | — | — |

Merge order: W3-01, W3-02, W3-03, W3-05, W3-04, W3-06, W3-11, W3-07, W3-09, W3-08, W3-10, W3-12.

## Integrator steps

- Relocate W2-02's court test application_contribution/producer/demand/bridge_denial/cause_contracts so its path mirrors application_output_demand/denial/bridge_conditional.
- Regenerate both layering baselines (upward and stray_impls) after the wave's compile pass.

## Owner calls open this wave

- ComputationPrior's home (every packet leaves its three holders alone).
- PendingProducerPublication: whether a checkpoint host route is added (W3-10).
- access_strategy's managed-influence types (W3-09, before the observed fact moves).
- A new domain sibling's name and place, if W3-05 needs one.

## Briefs

### W3-01

```text
W3-01 — Proposal 03 part 1: the host parameter, aliases and the facade
Model: opus
Design: proposal-03-plan.md §1-2 and §7 steps 2, 3 and 9, with the rulings: a grep gate so every impl header on the context types carries the host parameter; WorthQueryProgramApplicationRuntime takes a plain type parameter with no dyn default, recorded as an exception
Seams: product_operation -> application_installation (12 refs); product_operation -> application_runtime (10); application_installation -> application_runtime (7)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/context.rs and the files declaring the 14 public context types (proposal-03-plan.md §1)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/program/program_runtime.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_runtime/public_contexts.rs (new: the aliases)
- workspaces/worth-query/crates/worth-query-execution/src/facade.rs and facade/*.rs (alias re-exports)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/navigation/{output_completion,output_profile,output_source}.rs and application_output_demand/original_outputs.rs (the 4 stray impls on WorthQueryProgramApplicationRuntime)
- scripts/query_split/context_host_gate.py (new: the grep gate, with a unit test in test_query_split.py)
Steps:
1. Add the Host parameter (dyn default) to the 14 types, one type family at a time (entry/selected/history/correction; branches/heads; document history/closing/navigation; installation's three plus the program runtime). Every existing impl block in each type's own module gets the parameter in the same edit; constructors in the creation chain store `self`.
2. Aliases in application_runtime/public_contexts.rs; the consumer facade re-exports the aliases, so consumer names stay.
3. The 4 stray impls on WorthQueryProgramApplicationRuntime go home to its module, or become extension traits owned where they sit when they need something above it.
4. The grep gate: every impl header on a context type names the host parameter; add it as a script with a unit test, and record each generic (the 14 types and every generic shell) in plans/query-split/stage2/seams/packet-notes/W3-01.md with its reason, as ruling (a) requires.
Done when:
- The grep gate passes on the tree; the facade names are unchanged.
- Stray impls on WorthQueryProgramApplicationRuntime: 0.
- Host order, primary graph (python3 scripts/query_split/vocabulary_simulation.py workspaces/worth-query/crates/worth-query-execution/src/lib.rs domain_computation::primary_graph --impl-limit 600 --host-order plans/query-split/stage2/ownership/rerun/stack-primary_graph-draft.json --exclude-designed plans/query-split/stage2/seams/contracts.md plans/query-split/stage2/seams/merges.md plans/query-split/stage2/seams/proposal-03-plan.md plans/query-split/stage2/ownership/vocabulary/held-back.md --out <scratch>): product_operation -> application_installation and -> application_runtime shrink; no new seam.
- Line caps (dirty) pass.
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-02

```text
W3-02 — Proposal 03 part 2: blocks below product_operation move up into it
Model: sonnet
Design: proposal-03-plan.md §3.2 and §7 step 4 (the canonical host order puts application_output_demand, application_discovery, entity_resolution and resolution below product_operation)
Seams: application_output_demand -> product_operation (13); entity_resolution -> product_operation (5); stray impls on WorthQuerySelectedProductOperation in the four modules below it (21 of 29)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_discovery/workflow/{definition,enumeration,history_page,read,subscription}.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/{completed_outputs,completed_read,completed_set,currentness}.rs, currentness/{consumed,settlements,court_doors_for_test}.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/entity_resolution/admitted_scope.rs, entity_resolution/product.rs and product/{admitted_mutation,issued_scope,retained_scope}.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/resolution/selected_product.rs and selected_product/{admitted,admitted/native,admitted/court_doors_for_test}.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/<one module per source module> (new destinations)
Steps:
1. Each inherent block on WorthQuerySelectedProductOperation in these files moves up into product_operation (one destination module per source module), as an inherent impl in the type's own tree, with the Host parameter in its header (the W3-01 grep gate).
2. Where the moved block calls functions of the module it left, it calls them with the parts it needs (snapshot lease, product, application basis), not the context; those functions stay where they are.
3. Doors move with their blocks; court doors regenerated.
Done when:
- No stray impl on WorthQuerySelectedProductOperation remains in application_discovery, application_output_demand, entity_resolution or resolution.
- The W3-01 grep gate passes.
- Host order, primary graph: application_output_demand -> product_operation and entity_resolution -> product_operation shrink; no new seam.
- Line caps (dirty) pass.
Depends on: W3-01 (the host parameter on the selected operation)
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-03

```text
W3-03 — Proposal 03 part 3: blocks above product_operation become extension traits; the 9 public methods
Model: opus
Design: proposal-03-plan.md §3.2, §5-6 and §7 steps 5-8, with the ruling: the 9 public methods above product_operation become extension traits through the consumer facade, with import updates by script
Seams: product_operation -> application_query (78 refs in 27 items); stray impls on WorthQuerySelectedProductOperation in application_query and conditional_operation (8 of 29)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/live/lease/open.rs, observed_source/retained_dependencies.rs and retained_dependencies/{comparison,coverage}.rs, observed_source/workflow_capture.rs, readiness.rs, retained_read.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/conditional_operation/clock_observation/product_admission.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_runtime/host/identity.rs (the application_runtime() bridge)
- scripts/query_split/extension_trait_imports.py (reuse W2-09's script for consumer imports)
Steps:
1. Each block above product_operation becomes an extension trait owned by its module, blanket-implemented over the context with the host parameter; callers import it.
2. The 9 public methods become extension traits re-exported through the consumer facade; consumer imports added by script (extension_trait_imports.py), recorded in packet-notes/W3-03.md.
3. Erase step at the remaining sites (by value at facade hand-offs, by generic shell at reference sites); remove the context bridges of proposal-03-plan.md §5 and, when its count is 0, application_runtime() and its implementation.
Done when:
- No stray impl on WorthQuerySelectedProductOperation remains in application_query or conditional_operation.
- Host order, primary graph: product_operation -> application_query shrinks (target 0 for the context-type items); no new seam.
- application_runtime() is gone, or packet-notes/W3-03.md lists each remaining bridge with its blocker.
- Line caps (dirty) pass.
Depends on: W3-01
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-04

```text
W3-04 — Domain seams into the primary graph (contracts.md §2)
Model: opus
Design: contracts.md §2: provider_session -> primary_graph (payload pass-through removed, closure-capture change in graph_obligation, PrimaryReadSetCapture port), application_aftermath -> primary_graph (idempotency binding to application_outcome_identity with the OperationScopeIdempotency inversion; CorrectionCausality port), execution_runtime -> primary_graph (ProductSourceInvalidation port, root wiring), product_publication -> primary_graph
Seams: application_aftermath -> primary_graph (25 refs in 11 items); provider_session -> primary_graph (19 in 11; the snapshot lease, inspection target and source-facts allocation go with W3-05); execution_runtime -> primary_graph (5 in 4); product_publication -> primary_graph (4 in 2)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/provider_session/** files naming primary_graph items (graph_obligation and the read-set capture)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/application_aftermath/** files naming primary_graph items (idempotency, correction causality, recovery handle)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/execution_runtime/{runtime_root,product_world/**} wiring to output_lineage invalidation and root
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/product_publication/{owner_close,discovery,unpublished}.rs
Steps:
1. Follow contracts.md §2 for each seam; ports are dyn, owned by the lower module; record each generic kept and why in packet-notes/W3-04.md.
2. Do not move the snapshot lease, the inspection target or the source-facts allocation (W3-05 lowers them).
Done when:
- Host order, domain level (python3 scripts/query_split/vocabulary_simulation.py workspaces/worth-query/crates/worth-query-execution/src/lib.rs domain_computation --impl-limit 600 --host-order plans/query-split/stage2/ownership/rerun/stack-domain_computation-draft.json --exclude-designed plans/query-split/stage2/seams/contracts.md plans/query-split/stage2/seams/merges.md plans/query-split/stage2/seams/proposal-03-plan.md plans/query-split/stage2/ownership/vocabulary/held-back.md --out <scratch>): application_aftermath, execution_runtime and product_publication -> primary_graph gone or each remaining item named with its blocker; provider_session -> primary_graph keeps only W3-05's three items.
- No new seam at either level; line caps pass.
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-05

```text
W3-05 — Merges §2 step 4: snapshot lease, inspection target and source-facts allocation below authorization
Model: opus
Design: merges.md §2, Recommended structure 1 (third bullet) and Migration 4: each follows its blocker (the lease follows the layout, now in graph_layout; the inspection target takes an InspectionOriginalBytes contract owned by the lowered module, implemented in causal inspection)
Seams: provider_session -> primary_graph (lease, inspection target, allocation); authorization -> primary_graph (158 refs in 63 items; the parts these three carry)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/snapshot_lease.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/causal_inspection/application_inspection/target.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/source_facts/allocation.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/graph_layout/ or a new domain sibling named for what it holds (the lowered destination)
Steps:
1. Lower each of the three to the domain level below provider_session and authorization, whole, when its impls name nothing above; the inspection target through the InspectionOriginalBytes contract.
2. Write packet-notes/W3-05.md first if a destination other than graph_layout is needed (its name and place in the domain stack are an owner call).
Done when:
- Host order, domain level: provider_session -> primary_graph and authorization -> primary_graph no longer list the three items.
- No new seam; line caps pass.
Owner calls (do not decide these):
- If a new domain sibling is needed, its name and place in the domain stack.
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-06

```text
W3-06 — Merges §2 step 5: attempt-driving progression steps move up into the primary graph
Model: opus
Design: merges.md §2, Recommended structure 2 and Migration 5: elevation-approval replay, delegation replay and the attempt parts of provider commit authorization move out of authorization beside what they drive; authorization keeps the decisions and their typed refusals
Seams: authorization -> primary_graph (158 refs in 63 items: 117 were in operation_progression, 32 elevation_progression, 11 delegation_progression at the design's count)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/operation_progression/** (the attempt-driving steps only)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/elevation_progression/**
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/delegation_progression/**
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/elevation_approval_replay/ and a sibling for delegation replay (destinations)
Steps:
1. One progression at a time (design order); each step that drives the attempt moves up whole; authorization keeps functions taking the observed data and returning WorthQueryOperationAuthorizationDenial, its kind and refusal kind, unchanged.
2. The two runtime bridges the design names go with the moves.
Done when:
- Host order, domain level: authorization -> primary_graph shrinks by the moved steps' references; no new seam.
- Typed refusals unchanged (packet-notes/W3-06.md lists each moved step and the decision it calls).
- Line caps pass.
Depends on: W3-05 (merge after it; disjoint files)
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-07

```text
W3-07 — The older stray inherent impls (29)
Model: opus
Design: Owner ruling on stray impls: an impl moves home if it needs nothing above its type; otherwise its behaviour becomes an extension trait owned by the module that needs it, or a free function. Check each against the host-trait boundary: behaviour on the runtime belongs in application_runtime as an extension trait.
Seams: stray_impls: WorthQueryExecutionRuntime 7, WorthQueryPrimaryGraphBootstrap 5, WorthQueryPrimaryGraphIntegrationHandle 4, WorthQueryExecutionInstallationAuthority 4, InstalledTransportPendingReason 2, WorthQueryExecutionObservation 2, and one each on WorthQueryAdmittedHistoryProgramOutput, DecodedApplicationCheckpoint, WorthQueryPrimaryGraph, WorthQueryApplicationInvariantProjectionAuthority, WorthQueryAdvancementDenial
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/{convergence_epoch/admission,execution_resource_attempt,managed_run/admission,managed_run/direct_admission,operation_binding/authority/runtime_binding}.rs (+ their court doors)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/{application_runtime/publication_entry,handler/registry,invariant_projection/retention,typed_bootstrap,index_currency,index_refresh,merge_unique_values,application_invariant_preparation,bootstrap/checkpoint,bootstrap/preparation,entity_resolution,ordinary_read/projection}.rs (+ their court doors)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/external_dispatch/denial.rs, application_attempt/compare_and_commit/commit_outcome/denial/advancement.rs, product_operation/program_adoption/custody/recovery/readmission.rs, application_installation/program/demand/history.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/request_execution/advancement/observation.rs
Steps:
1. For each listed impl: move home, or extension trait owned where it sits, or free function; record the choice and the reason per impl in packet-notes/W3-07.md.
2. Doors on these impls follow the behaviour (door traits where the behaviour stays above the type).
Done when:
- The layering check's stray_impls lists lose all 29 (the integrator regenerates the baselines).
- No new seam; line caps pass.
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-08

```text
W3-08 — The retention envelope and its preconditions (design, then implement)
Model: opus
Design: contracts.md Family A, step 3 and the W2-04 leftovers: the envelope (RetainedComputation, SealedComputationRun/Retention, CompletedComputation*, ComputationDeposit, carried custody) is blocked until the attempt's sealed-facts data (SealedComputationFacts with its index and layouts, FactContent, CheckedReadLayout, NativeCallCarriage) and invariant_projection's call-shape vocabulary (CallShape, ComputationCallSequence, ComputationCallCharge) are lowered below output_lineage
Seams: output_lineage -> application_contribution (14 refs in 6 items); invariant_projection -> application_contribution (12 in 7); application_attempt -> application_contribution (7 in 3); application_output_demand -> application_contribution (7 in 5, retention part)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/read_set/computation_routing/** (the sealed-facts data only; computation_routing.rs itself stays)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/invariant_projection/** files declaring CallShape, ComputationCallSequence, ComputationCallCharge (not operation_reader/decision_reads.rs or computation_calls.rs)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/partitioned_computation/incremental/{retained,sealing,carriage,deposit}.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/computation_retention/** (destination)
Steps:
1. Design first (packet-notes/W3-08.md): the precondition lowering, then the envelope move, with drop order in retain_computation kept byte for byte and the thread_local test handoffs moving with the type whose door uses them.
2. Implement: lower the preconditions, then move the envelope into computation_retention.
Done when:
- Host order, primary graph: the four seams list no retention item other than ComputationPrior.
- No new seam; line caps pass.
Owner calls (do not decide these):
- ComputationPrior's home: leave required_context.rs, decision_reads.rs and computation_calls.rs as they are.
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-09

```text
W3-09 — The attempt record remainder: commit receipt placement, Uncommitted, the observed fact, CompletedOriginalRead (design, then implement where unblocked)
Model: opus
Design: contracts.md Family B (host-order direction) and the W2-08/W2-09 leftovers. WorthQueryApplicationObservedFact needs family C's move of ObservedSourcePublication, the source-facts allocation (W3-05) and a ruling on access_strategy's managed-influence types; CompletedOriginalRead needs FactPayload placement and the snapshot lease (W3-05)
Seams: provider -> application_attempt (45 refs in 14 items); invariant_projection -> application_attempt (33 in 12); application_output_demand -> application_attempt (27 in 5); output_lineage -> application_attempt (23 in 2); application_checkpoint -> application_attempt (14 in 1)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/commit_receipt/** and compare_and_commit/commit_outcome/** (receipt placement, Uncommitted)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/fact/** (observed fact: design only this wave)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/read_set/original_read.rs and application_checkpoint/facts/payload.rs (CompletedOriginalRead, FactPayload: design only this wave)
Steps:
1. Design all four in packet-notes/W3-09.md (data down, behaviour as attempt-owned traits, doors with the behaviour).
2. Implement the commit receipt placement and Uncommitted this wave; the observed fact and CompletedOriginalRead stay designed until W3-05 merges.
Done when:
- packet-notes/W3-09.md committed first; the receipt and Uncommitted no longer listed in the five seams.
- No new seam; line caps pass.
Depends on: W3-05 for the observed fact and CompletedOriginalRead (implement those only after it merges; design now)
Owner calls (do not decide these):
- access_strategy's managed-influence types (needed before the observed fact can move).
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-10

```text
W3-10 — Output-demand evaluation leftovers: PendingProducerPublication, CurrentProof, OwnerPlan (design, then implement)
Model: opus
Design: contracts.md Family E and the W2-03 leftovers: PendingProducerPublication's impls take &dyn ApplicationContributionHost and no checkpoint host route exists; CurrentProof and OwnerPlan are named by no design
Seams: application_output_demand -> application_contribution (non-retention part); application_checkpoint -> application_output_demand (4 in 3)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/** files declaring PendingProducerPublication, CurrentProof, OwnerPlan
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/registry/checkpoint.rs and the files naming CurrentProof and OwnerPlan (not required_context.rs)
Steps:
1. Design (packet-notes/W3-10.md): owner and contract for each of the three; for PendingProducerPublication, the host route its impls need.
2. Implement CurrentProof and OwnerPlan; implement PendingProducerPublication only if the design needs no new host route, otherwise stop at the design.
Done when:
- packet-notes/W3-10.md committed first; CurrentProof and OwnerPlan no longer listed upward.
- No new seam; line caps pass.
Owner calls (do not decide these):
- PendingProducerPublication: whether a checkpoint host route is added (the design states the choice).
- ComputationPrior's home: leave required_context.rs as it is.
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-11

```text
W3-11 — The resolver contract for authorization (design, then implement)
Model: opus
Design: W2-06 leftover: the resolver use in authorization's request_resolution.rs and exact_observation.rs needs a contract owned below authorization, implemented in the primary graph (the admission_subject pattern of W2-06, option 3)
Seams: authorization -> primary_graph (resolution part)
Primary files (yours alone this wave):
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/**/request_resolution.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/**/exact_observation.rs
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/admission_subject/** (the contract module W2-06 made, extended)
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/resolution/** implementing files (not resolution/selected_product*, which W3-02 owns)
Steps:
1. Design (packet-notes/W3-11.md): the object-safe contract, its owner, the seals it keeps; then implement with `<Self as Trait>::method` forwards.
Done when:
- packet-notes/W3-11.md committed first; authorization -> primary_graph no longer lists the resolver items.
- No new seam; line caps pass.
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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

### W3-12

```text
W3-12 — Design pass: host-order seams the designs never covered (design only)
Model: opus
Design: The seam designs assumed the greedy order; under the canonical host order these seams have no design yet
Seams: output_lineage -> application_output_demand (36 refs in 5); handler -> application_entry (30 in 4); application_output_demand -> application_query (30 in 7); output_lineage -> invariant_projection (28 in 4); provider -> application_query (20 in 5); invariant_projection -> application_entry (19 in 1); provider -> output_lineage (17 in 10); application_discovery -> application_attempt (15 in 9); invariant_projection -> application_query (13 in 7); provider -> application_checkpoint (11 in 3); application_contribution -> conditional_operation (11 in 5); provider -> application_output_demand (11 in 3); the smaller seams in after-wave-2/primary_graph/host-order-seams.md; domain: provider_session -> managed_run (10), domain_evidence_binding -> provider_session (2), operation_binding -> provider_session (2), artifact_owner -> operation_binding (1), execution_runtime -> product_publication, managed_run, invalidation_equivalence_control (1 each)
Primary files (yours alone this wave):
- plans/query-split/stage2/seams/host-order-designs.md (new)
Steps:
1. For each seam: what crosses (verified in source at 42ae530f5b), the owner under the canonical layering, and the move or contract (lower vocabulary, port, authority transfer); the design names the files a wave-4 packet would own.
2. Edit no source.
Done when:
- host-order-designs.md covers every seam in after-wave-2/*/host-order-seams.md not owned by another wave-3 packet, each with a mechanical/careful mark and an estimate.
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
- No inherent impl outside the module tree that defines its self type (the layering check refuses new ones; stray_impls in scripts/query_split/*_layering.json is frozen and only shrinks). A behaviour that needs something above its type becomes an extension trait owned where it sits, or a free function.
- A door that calls behaviour owned above the type it is on becomes a door trait beside that behaviour (as WorkflowActorFieldsObservationForTest); a door signature never names a private trait or type.
- Forward host or extension methods by naming the implementing trait (`<Self as Trait<_>>::method`), never `Self::method(self)`: the integrator treats every unconditional_recursion warning as an error.
- ComputationPrior's home is an open owner call: leave its three holders as they are (application_output_demand/registry/required_context.rs, invariant_projection/operation_reader/decision_reads.rs, invariant_projection/operation_reader/computation_calls.rs).
- A packet marked "design, then implement" commits its design note under plans/query-split/stage2/seams/packet-notes/<id>.md before any source edit, then implements it; a packet marked "design only" edits no source.

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
