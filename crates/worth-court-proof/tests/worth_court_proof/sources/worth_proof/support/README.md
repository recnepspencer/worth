# WORTH Proof Test Support Topology

This support tree is organized by proof responsibility, not convenience.

Ownership split:

- `compile_fail/`
  - compile-fail case vocabulary
  - compile-fail bundle construction
  - compile-fail family execution helpers
- `compile_pass/`
  - compile-pass case vocabulary
  - compile-pass bundle construction
  - compile-pass trusted progression execution helpers
- `type_shapes/`
  - type-shape report vocabulary
  - codegen-honesty report vocabulary
  - debt inventory vocabulary
- `proof_shapes/`
  - proof-shape digest vocabulary
  - basis digest vocabulary
- `dx/`
  - DX-local compile-fail and compile-pass bundle definition
  - DX-local proof/failure/transition digest derivation
  - DX-local hot-path codegen-honesty capture
  - DX-local documentation default-path audit
  - DX-local residual-debt derivation
- `milestone1/`
  - Milestone 1 evidence derivation
  - Milestone 1 compile-fail bundle definition
  - Milestone 1 type-shape derivation
  - Milestone 1 proof-shape and basis digest derivation
  - Milestone 1 codegen/debt note derivation
- `milestone2/`
  - Milestone 2 compile-fail bundle definition
  - Milestone 2 proof-shape derivation for sealed and witness-bearing surfaces
  - Milestone 2 failure digest derivation for hostile minting and witness lanes
  - Milestone 2 representative codegen-honesty capture
  - Milestone 2 residual-debt report derivation
- `milestone3/`
  - Milestone 3 compile-fail bundle definition
  - Milestone 3 compile-pass control, same-basis readmission, and shifted-basis readmission bundle definition
  - Milestone 3 basis/failure/transition digest derivation for freshness, downgrade, basis drift, and readmission surfaces
  - Milestone 3 residual-debt report derivation for representative substrate-only closure debt
- `milestone4/`
  - Milestone 4 compile-fail ordering-misuse bundle definition
  - Milestone 4 compile-pass trusted progression, checked composition, freshness/failure, and admitted-equivalence bundle definition
  - Milestone 4 transition/failure digest derivation for the canonical transition contract, checked readiness carriers, outcome vocabulary, and divergence tags
  - Milestone 4 representative codegen-honesty capture for transition carriers, checked readiness surfaces, and static contract surfaces
  - Milestone 4 residual-debt report derivation for representative transition-substrate closure limits
- `milestone5/`
  - Milestone 5 compile-fail lowered-versus-ready boundary bundle definition under `tests/ui/milestone5/compile_fail/`
  - Milestone 5 compile-pass lowered-to-ready-to-executed, checked-readiness, same-basis runtime-readmission, and shifted-basis runtime-readmission progression bundle definition under `tests/ui/milestone5/compile_pass/`
  - Milestone 5 basis/failure/transition digest derivation for lowered, bridged-lowered, readmitted-lowered, ready, shifted-basis ready, and executed recipe surfaces
  - Milestone 5 representative codegen-honesty capture for lowered, ready, executed, readiness-context, and lowered-readmission carriers
  - Milestone 5 residual-debt report derivation for representative execution-boundary closure limits

Rules:

- generic compile-fail mechanics belong under `compile_fail/`
- generic compile-pass mechanics belong under `compile_pass/`
- generic report types belong under `type_shapes/`
- generic digest vocabularies for proof-bearing surfaces belong under `proof_shapes/`
- milestone-local evidence derivation belongs under the milestone module
- no domain-specific migration pressure belongs here yet because Milestone 1
  does not ship migration suites
