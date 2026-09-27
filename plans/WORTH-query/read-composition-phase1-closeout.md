# Read Composition Phase 1 Closeout

This document freezes the `worth-query` graph-composition-for-reads Phase 1
kernel boundary and the public answer to â€œis Worth allowed to begin Phase 2
adoption yet?â€

## Stable Surface

Public read-composition entry points:

- `compose_read`
- `compose_read_with_invariant_pack`
- `define_read_family`
- `define_read_family_with_invariant_pack`
- `execute_read_family`
- `execute_read_family_in_basis_context`

Canonical graph artifacts:

- `read_graph`
- `read_result`
- `read_receipt`
- `typed_read_denial`

Scope classes:

- `local_neighborhood`
- `anchored_expansion`
- `explicit_broad_search`

Graph families:

- `detail`
- `collection`

Execution engines:

- `query_runtime_current`
- `query_runtime_branch`
- `query_runtime_historical`
- `query_runtime_preview_derived`

Fallback classes:

- `none`
- `snapshot_indexed_debt`
- `whole_view_debt`

Built-in operators:

- `direct_edge`
- `successor_walk`
- `shared_endpoint`
- `shared_attachment`
- `bounded_ancestor`
- `bounded_descendant`
- `anchored_frontier`
- `frontier_search`

Relationship-proof postures:

- `not_required`
- `descriptor_admitted_synthetic_runtime`

Reusable family admission modes:

- `kernel_only`
- `domain_invariant_admitted`

Extension hooks:

- `domain_read_family_lowering`
- `domain_invariant_pack`
- `domain_decoder`
- `domain_result_certification`

Boundary guards:

- `operator_owned_builders_hide_traverse`
- `scope_class_relabeling_denies_typed`
- `built_in_operator_shape_denies_typed`
- `relationship_proof_admission_denies_typed`
- `domain_invariant_pack_denies_before_execution`

Denial lanes:

- `invalid_root`
- `built_in_operator_denied`
- `relationship_proof_admission_denied`
- `scope_shape_denied`
- `authoring_denied`
- `canonicalization_denied`
- `validation_denied`
- `planning_denied`
- `basis_resolution_denied`
- `basis_preflight_denied`
- `execution_denied`
- `domain_invariant_denied`

## Safe To Build Now

- compose_read, compose_read_with_invariant_pack, define_read_family, define_read_family_with_invariant_pack, execute_read_family, and execute_read_family_in_basis_context form one public read-composition product instead of separate helper stories
- the canonical read artifact is ReadGraph and every admitted execution returns a ReadReceipt with scope class, graph family, breadth, fallback posture, and relationship-proof admission identity
- the Phase 1 runtime taxonomy now freezes query_runtime_current, query_runtime_historical, and fallback classes as public read-kernel posture
- scope classes are kernel-owned and freeze local_neighborhood, anchored_expansion, and explicit_broad_search at the shared boundary instead of letting callers relabel the same lowered read
- operator-owned graph lanes now cover direct_edge, successor_walk, shared_endpoint, shared_attachment, bounded_ancestor, bounded_descendant, anchored_frontier, and frontier_search
- traversal-bearing reads now admit descriptor-backed synthetic runtime relationship proof before execution instead of reporting only a receipt heuristic
- invariant packs can narrow an admitted read graph before execution and deny through a typed domain-invariant lane with an attached rejected-graph summary
- reusable ReadFamily artifacts are part of kernel completeness and keep admission history in their digest so invariant-admitted families do not collapse into plain reusable reads
- operator-owned builders keep traversal ownership mechanical because the exported operator-builder boundary hides traverse and is compile-fail certified
- later domain adoption must extend through the frozen read-composition hooks for domain_read_family_lowering, domain_invariant_pack, domain_decoder, and domain_result_certification instead of rebuilding a second local read stack

## Must Not Assume Yet

- do not assume this Phase 1 kernel artifact by itself certifies Worth topology migration; that closure lives in the Worth topology-domain closeout surfaces
- do not assume this generic gate certifies future non-topology Worth domains; later trim, carrier, NURBS, fillet, and branch-history vocabularies still need domain-owned adoption on top of this kernel
- do not assume all future domain families already exist; later topology, trim, carrier, NURBS, fillet, and branch-history vocabularies still need domain-owned adoption on top of this kernel

## Migration Guidance

- for new domain adoption, start by moving one bounded read family onto compose_read plus a domain-owned decoded view before widening the family set
- bind each new domain family through the frozen lowering, invariant-pack, decoder, and certification hook boundaries instead of inventing local extension seams
- prefer an operator-owned read lane whenever the domain shape matches one of the admitted built-in operators instead of open-coding traverse in the Worth facade
- for topology snapshot read-only posture, use the admitted historical basis-aware read-family path instead of carrying stale fallback debt wording
- for later Worth domains, do not resume domain-specific widening until that domain has an aggregate closeout proof naming any remaining fallback consumers as debt rows

## Phase Gate

- `phase_one_kernel_complete`
  - `satisfied`
  - the generic read kernel is frozen with stable operators, typed denials, reusable read families, typed extension hooks, and descriptor-backed relationship-proof admission
- `phase_two_worth_adoption_ready`
  - `satisfied`
  - Worth may begin domain adoption through the frozen lowering, invariant-pack, decoder, and certification hooks, starting with loop_cycle_neighborhood
- `phase_three_aggregate_proof_complete`
  - `satisfied`
  - Worth topology now exposes aggregate query-native-versus-fallback breadth, debt, parity, and no-N-plus-one proof through its domain closeout surfaces
- `milestone_three_resume_ready`
  - `satisfied`
  - Milestone 3 may resume through the Worth topology side-quest closeout gate because Phase 3 aggregate proof is now enforced outside the generic kernel

## Required Verification Commands

- `cargo fmt --package worth-query`
- `cargo test -p worth-query runtime::tests::read_composition --quiet`
- `cargo test -p worth-query --test phase_boundaries_compile_fail --quiet`
- `cargo test -p worth-query --quiet`
- `git diff --check`
