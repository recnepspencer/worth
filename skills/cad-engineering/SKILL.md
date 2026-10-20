---
name: cad-engineering
description: Apply WORTH CAD laws when designing, implementing, or reviewing numerical geometry, sketch solving, B-rep topology, modeling, measurement, or CAD exchange. Trace numerical ownership, exactness, tolerance, and graph authority across the affected path.
---

# CAD Engineering

Apply the CAD laws to the user's requested work. Preserve the requested scope
and mode: review, planning, implementation, or inline drafting.

## Governing context

Resolve document paths relative to the repository whose CAD behavior is being
worked on, not relative to this installed skill. Locate and read its CAD laws
and the relevant general coding guidelines.

For worth-proprietary, use:
- `docs/house/cad-laws.md` for the CAD engineering requirements.
- `docs/house/graph-kernel.md` for graph-native CAD authority and contracts.
- `docs/house/rust-ui-architecture.md` for the destination ownership tree.
- The specification governing the affected modeling or authoring family.

General WORTH coding guidelines live in the governing platform checkout's
`docs/coding-guidelines/`. In the current multi-repository workspace, the platform
checkout is `forge` and the CAD checkout is `worth-proprietary`. Use the checkout
governing the task; do not silently substitute a different worktree's specification.

Check status and supersession notices. Retained geometry docs and operator or
validator inventories do not override current authority or topology contracts.
Existing APIs are implementation evidence, not exemptions from the laws.

## Trace the consequential path

Before changing or evaluating a CAD path, identify:

- Authored inputs and their numerical origin.
- Quantity, unit, frame and parameter-domain meaning.
- The numerical owner and actual public capability used.
- Tolerance policy, input uncertainty and approximation guarantees.
- Which facts establish geometry, topology, eligibility and current publication.
- The source dependencies and resource bounds relevant to those facts.

Read the real implementation and consequential consumers. Names such as
"exact", "robust", "finite" and "certified" are claims to inspect.

## Resolve ownership at the responsible boundary

Keep shared arithmetic, decimal admission, rounding, predicates and root
machinery in worth-math; quantities in worth-measure; general geometric
algorithms in worth-geom.

Keep sketch and modeling interpretation at their domain owners. Keep graph
identity, dependency tracking and publication with the platform integration.

If a required guarantee is absent, correct the owning capability and its
affected consumers within the authorized scope. Do not patch around it with
a second parser, epsilon rule, private solver, topology store or freshness
registry.

Honor committed destination topology without creating unused placeholders.

## Review numerical honesty

Inspect relevant:

- f32/f64 fields, parameters, extraction and casts.
- Decimal parsing and display-to-source round trips.
- Hardcoded epsilon, grid, clamp and perturbation decisions.
- Loss of units, frame, uncertainty or parameter meaning.
- Residuals presented as stronger bounds.
- Approximation or tessellation promoted into authoritative geometry.
- Exact arithmetic whose intermediate work is unbounded.

Classify floating-point use by its named purpose and boundary. Do not treat a
numerical crate, finite wrapper or existing legacy API as a blanket exemption.

Preserve exact authored values. Require typed accuracy-bearing projections
where conversion error matters. Distinguish arithmetic ambiguity, uncertain
input, unsupported geometry and resource exhaustion.

## Select evidence for the actual claim

Choose independent expectations and adversarial cases relevant to the change.

Use numerical tests for numerical claims, real graph integration for publication
and currentness, native authoring for UX claims, and an independent interpreter
for exchange preservation.

Do not expand into unrelated cleanup or create evidence machinery whose main
purpose is to certify other evidence. Preserve human approval checkpoints
already required by the governing specification; do not invent new ones.

## Report

State the affected ownership boundary, concrete result or findings, supporting
evidence and material limitations.

For reviews, separate established violations from proposed improvements.
For implementation, identify any remaining incomplete capability explicitly.

Do not claim adoption, installation, execution or compliance without evidence.
