# Milestone 2 Closeout: Sealed Minting And Witness Authority

## Status

Closed.

Milestone 2 now has an implemented named certification surface and a machine-checkable
closeout bundle that matches the engineering spec and the crate-level test
requirements.

## Implemented Surface

- Sealed stronger proof-bearing construction:
  - `Proof<P>`
  - `CanonicalVec<T>`
  - `UniqueVec<T>`
  - `DisjointPair<T>`
- Sealed witness vocabulary:
  - `AuthorityWitness<A>`
  - `CapabilityWitness<C>`
- Representative witness-bearing recipe progression:
  - `Recipe<Unresolved, ...> -> Recipe<Resolved, ...>`
  - `Recipe<Resolved, ...> -> Recipe<Lowered, ...>`
  - `Recipe<Lowered, ...> -> Recipe<Admitted, ...>`
- Public access remains through the crate facade only.

## Certification Surface

Named suite:

- `Sealed Minting And Witness Authority Test`

Primary test:

- [sealed_minting_and_witness_authority_certification.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/sealed_minting_and_witness_authority_certification.rs)

Supporting evidence module:

- [tests/support/milestone2/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/support/milestone2/mod.rs)

Machine-checkable outputs:

- `compile_fail_bundle`
- `proof_shape_digest`
- `failure_digest`
- `codegen_honesty_report`
- `residual_debt_report`

## Hostile Coverage

The closeout suite now owns the hostile minting and authority lanes required by
the milestone:

- WORTHd stronger-form construction denial
- WORTHd witness minting denial
- witness-required progression denial without witness input
- recipe-stage skip-construction denial

Compile-fail fixtures:

- [stronger_proof_bearing_constructors_are_not_public.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/stronger_proof_bearing_constructors_are_not_public.rs)
- [observed_proofs_cannot_be_duplicated.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/observed_proofs_cannot_be_duplicated.rs)
- [witnesses_are_not_publicly_mintable.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/witnesses_are_not_publicly_mintable.rs)
- [witness_required_apis_reject_callers_without_witness.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/witness_required_apis_reject_callers_without_witness.rs)
- [recipe_stages_are_not_publicly_skippable.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/recipe_stages_are_not_publicly_skippable.rs)

## Zero-Cost Posture

Milestone 2 certifies representative size/alignment/drop honesty for:

- sealed proof forms
- authority and capability witnesses
- sealed proven collections
- resolved/lowered/admitted recipe stages

The current codegen posture is intentionally scoped to representative
size/layout/drop honesty only. No MIR or ASM baseline diff against bespoke
domain code is shipped yet; that remains explicit residual debt rather than an
implicit completeness claim.

## Residual Debt

- Public cross-crate trusted witness issuer ergonomics remain deferred. The
  milestone hardens crate-owned witness minting, but does not yet ship a
  broader domain-facing issuer facade.
- Codegen honesty remains representative rather than handwritten-baseline
  comparative.

These are explicit debt items, not hidden closure gaps.

## Verification

Verified with:

- `cargo fmt -p worth-proof`
- `cargo test -p worth-proof sealed_minting_and_witness_authority_certification -- --nocapture`
- `cargo test -p worth-proof witness_authority_boundaries_hold -- --nocapture`
- `cargo test -p worth-proof`

## What Later Milestones May Assume

Milestone 3, Milestone 4, and Milestone 5 may now assume:

- stronger proof-bearing forms are not ordinary-caller constructible
- witness-bearing authority is the canonical static admission pattern
- recipe progression is stage-typed and skip construction is compile-time denied
- the Milestone 1 carrier family remains the canonical substrate; Milestone 2
  did not replace it with a parallel wrapper system
