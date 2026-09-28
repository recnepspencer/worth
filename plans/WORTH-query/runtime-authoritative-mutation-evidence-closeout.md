# Runtime Authoritative Mutation Evidence Closeout

This closeout freezes what downstream runtimes may rely on from the
cross-runtime authority-evidence lane right now.

## Safe To Build Now

- workspace.insert/update/delete/batch receipts preserve declared-versus-resolved target evidence together with touched-aspect fallout
- existing-truth binding, same-batch symbolic target reference, same-batch symbolic aspect reference, naming mutation, and continuity mutation evidence are part of the ordinary public receipt and inspection story
- graph composition receipts and inspection now expose explicit symbolic-to-resolved mapping instead of forcing downstream domains to infer same-batch edge resolution from final rows alone
- graph composition receipts and inspection now expose explicit entity-versus-relation breadth counters instead of forcing downstream domains to reconstruct graph breadth from generic batch rows
- graph composition receipts and inspection now expose an explicit canonical lowered program ordering instead of forcing downstream domains to infer component meaning from generic batch families
- graph composition receipts and inspection now expose explicit lifecycle outcome snapshots instead of forcing downstream domains to infer create-versus-update-versus-retire meaning from step kinds alone
- graph composition now admits symbolic entity follow-up mutation and symbolic relation retirement as ordinary typed lifecycle steps instead of forcing downstream domains back onto scalar batch escape hatches
- graph composition now admits existing-target update and retirement steps inside the same canonical program instead of forcing mixed created/existing workflows back onto generic batch orchestration
- graph composition now admits existing-target retarget steps as explicit identity-preserved lifecycle lanes instead of flattening successor rewires back into generic update semantics
- graph composition now admits existing-target supersession steps as explicit lineage-preserved lifecycle lanes instead of flattening split-or-merge successor semantics into retarget or retirement folklore
- graph composition now admits bridge-verified existing-target update and retirement steps inside the same canonical program instead of forcing verified mixed-shape workflows back out into separate batch orchestration
- graph composition now admits bridge-verified existing-target retarget steps inside the same canonical program instead of making verified relation rewires fall back to generic update folklore
- graph composition now admits bridge-verified existing-target supersession steps inside the same canonical program instead of making lineage-preserved verified rewrites masquerade as plain updates
- graph composition declaration and symbolic-edge failures now deny through a typed graph-composition lane instead of collapsing into generic workspace strings
- graph composition denied paths now expose admission traces with explicit failure stages instead of forcing callers to infer where admission stopped from denial prose alone
- graph composition invariant-pack rejection now denies through a distinct domain-invariant lane instead of collapsing domain invalidity into generic graph-composition support denial
- graph composition domain-invariant denials now expose attempted-shape summaries with declared collections, declared symbols, capability families, and lifecycle families instead of forcing kernels to reconstruct rejected topology from builder folklore
- graph composition support is now machine-readable by capability class and extension-hook boundary instead of forcing downstream domains to treat one flat family list as the whole support contract
- verified graph composition lanes now expose aggregate assumption snapshot, verified precondition, and read-set-breadth summaries instead of forcing kernels to reconstruct operation preconditions from component rows one by one
- lineage-carrying graph composition lanes now expose aggregate prior-versus-successor continuity summaries instead of forcing kernels to reconstruct edge-split lineage from scattered component continuity rows
- existing-truth assertions now distinguish retained authoritative assertions from backend-verified assertions on the public receipt and inspection surface
- backend-verified existing-truth lanes now expose verified assumption-set, assumption snapshot token/digest, verified precondition digest, and read-set-breadth evidence instead of collapsing all verification meaning into one opaque assertion digest
- mixed existing-truth authority sessions now preserve aggregate mode evidence that distinguishes retained assertions, backend-verified assertions, verified updates, and verified deletes without reconstructing that story from component receipts
- existing-truth probes now expose a typed backend-verified probe lane for current authoritative values without smuggling that truth through mutation receipts
- existing-truth verified updates now expose a typed backend-verified update lane that proves current authoritative values before applying update-family mutation receipts
- existing-target relation updates on admitted families preserve authoritative relation identity instead of disguising delete-plus-recreate as update vocabulary
- existing-truth verified deletes now expose a typed backend-verified delete lane that proves current authoritative values before applying delete-family mutation receipts
- existing-truth batch receipts, scalar inspection, and probe surfaces keep retained assertions, backend-verified assertions, backend-verified probes, verified updates, and verified deletes semantically distinct under mixed authority sessions
- primary multi-command batches now commit atomically at the backend boundary instead of degrading into per-command commits, so invariant-complete closures can rely on one truth-change boundary
- batch and import-style authority sessions preserve aggregate existing-binding, symbolic-target, symbolic-aspect, naming, continuity, causality, and provenance digests
- bridge-backed verified-existing support rows are machine-readable by operation family and target-binding family instead of hiding behind one generic backend bool
- downstream domains may rely on Query receipts and inspection instead of rebuilding target-recovery, naming, or continuity explanation glue locally
- downstream domains may rely on `verify_existing(...)` only when the active backend actually supports backend verification; unsupported backends remain typed and fail-closed
- downstream domains may rely on `update_existing_verified(...)` only when the active backend actually supports backend verification; unsupported backends remain typed and fail-closed
- downstream domains may rely on `delete_existing_verified(...)` only when the active backend actually supports backend verification; unsupported backends remain typed and fail-closed

## Must Not Assume Yet

- authority-mutation evidence closes durable restart, temporal, async, or store-backed mutation semantics
- unsupported identity-binding, naming, or continuity families remain fail-closed until explicitly admitted
- unsupported existing-truth binding, assertion, verified-mutation, and probe neighbors remain typed and fail-closed rather than degrading into best-effort fallback behavior
- unsupported identity-preserving relation update families remain fail-closed until the lower runtime can preserve target identity honestly
- bridge-backed verified-existing support rows that deny on the primary posture may not be treated as production-ready just because the scaffold posture admits them
- downstream code may bypass Query receipts and inspect raw bridge/runtime provenance bags directly

## Migration Guidance

- move authoritative mutation onto workspace.insert/update/delete plus explicit submission or graph-composition lanes, and consume receipts plus inspect output as the domain explanation contract
- read bridge-backed verified-existing support rows before teaching graph-composition verified-existing lanes or probe-intent execution as ordinary production runtime flows
- read graph-composition capability rows and extension-hook rows before teaching a new mixed-shape lifecycle or domain extension as ordinary stable runtime support
- use `workspace.compose_graph(...)` or `workspace.compose_graph_with_invariant_pack(...)` when one logical mutation needs symbolic resolution, verified preconditions, lineage, or domain-invalidity evidence as part of the ordinary receipt story
- use typed existing-truth binding artifacts inside graph composition when a retained assertion, verified precondition, update, or retirement must stay identity-preserved
- use `workspace.probe_existing_intent(request).execute()` when the domain needs current authoritative aspect values as input rather than a retained assertion receipt
- use graph-composition existing-target update, retarget, supersession, and retirement lanes when an admitted relation family must preserve authoritative target identity
- use graph-composition verified-existing lanes when the backend must prove current stored truth immediately before an existing-target mutation
- delete local existing-target rebinding, naming outcome reconstruction, and continuity breadcrumb glue once equivalent Query evidence is available
- delete local graph-program rejection reconstruction once `admission_trace()` and `domain_invariant_summary()` cover the denied-path explanation contract
- treat unsupported mutation-evidence neighbors as fail-closed support gates rather than alternate runtime seams

## Bridge Carry-Forward Contract

- bridge writeback artifacts can carry target, causality, provenance, naming, and continuity evidence into one Query-facing contract
- batch/session authority bundles preserve aggregate existing-binding, symbolic-target, naming, continuity, causality, and provenance digests
- replay-safe request and receipt digests remain part of the carry-forward story for admitted authority sessions

## Certified Support Families

- existing-truth binding: `direct_entity_identity`, `direct_relation_identity`
- existing-truth assertion modes: `retained_authoritative_assertion`, `backend_verified_assertion`
- existing-truth probe modes: `backend_verified_probe`
- existing-truth verified mutation modes: `backend_verified_update`, `backend_verified_delete`
- bridge-backed verification rows:
  - `verify_existing` x `direct_entity_identity`
  - `verify_existing` x `direct_relation_identity`
  - `probe_existing` x `direct_entity_identity`
  - `probe_existing` x `direct_relation_identity`
  - `update_existing_verified` x `direct_entity_identity`
  - `update_existing_verified` x `direct_relation_identity`
  - `delete_existing_verified` x `direct_entity_identity`
  - `delete_existing_verified` x `direct_relation_identity`
- identity-preserving update families: `direct_entity_identity_update`, `direct_relation_identity_update`
- symbolic same-batch target reference: `same_batch_declared_target`
- symbolic same-batch aspect reference: `same_batch_declared_entity_identity`
- graph composition capability rows:
  - `same_batch_entity_relation_identity_edges` x `target-combination`
  - `mixed_existing_and_symbolic_entity_identity_edges` x `target-combination`
  - `same_batch_symbolic_entity_followup_mutation` x `lifecycle-step`
  - `same_batch_symbolic_relation_followup_mutation` x `lifecycle-step`
  - `same_batch_symbolic_relation_retirement` x `lifecycle-step`
  - `mixed_existing_target_followup_mutation` x `lifecycle-step`
  - `mixed_existing_target_retarget` x `lifecycle-step`
  - `mixed_existing_target_supersession` x `lifecycle-step`
  - `mixed_existing_target_retirement` x `lifecycle-step`
  - `mixed_existing_target_verified_followup_mutation` x `lifecycle-step`
  - `mixed_existing_target_verified_retarget` x `lifecycle-step`
  - `mixed_existing_target_verified_supersession` x `lifecycle-step`
  - `mixed_existing_target_verified_retirement` x `lifecycle-step`
- graph composition extension-hook rows:
  - `domain_lowering_hook` x `lowering`
  - `domain_invariant_pack_hook` x `invariant-pack`
  - `domain_interpretation_hook` x `interpretation`
- naming mutation families: `attach_new_target`, `attach_existing_target`, `rebind_target`, `remove`
- continuity mutation families: `rebind_existing_target`, `split_existing_target`

## Verification

- `cargo fmt -p worth-query`
- `cargo check -p worth-query --tests`
- `cargo test --manifest-path crates/worth-query/Cargo.toml --test phase_boundaries_compile_fail`
- `cargo test -p worth-query`
- `cargo fmt -p worth-runtime-bridge`
- `cargo check -p worth-runtime-bridge --tests`
- `cargo test -p worth-runtime-bridge`
- `cargo test --manifest-path crates/worth-runtime-bridge/Cargo.toml --test phase_boundaries_compile_fail`
- `git diff --check`
