# Aspect Contracts, Values, And Authoritative State

This folder documents the Milestone 1 aspect surface in
`worth-foundational`.

For nested persisted values, `WorthValue` derives canonical Serde serialization,
portable `ValueShape` and a checked seed together. `CanonicalValue` is Query-free;
opaque numerical/source owners retain their actual construction contract.
Ordinary bindings use `canonical,` without guessed ceilings; `ValueDecodeBounds`
retains optional explicit domain policies and carries no runtime resource or
publication authority. See the [application guide](../../../../docs/build-an-application.md#canonical-nested-values-and-derived-readiness)
and its executable nested-value/compiler examples. Deep retained memory
description is a separate traversal; a Serde-skipped field cannot disappear from
resident-memory accounting.

Use these docs when you need to answer questions like:

- How do I define the meaning of an aspect before any value becomes
  authoritative?
- How do I model struct-shaped aspects, field paths, and masks without falling
  back to loose JSON rules?
- How do scalar, reference, and opaque aspect shapes differ?
- How do I validate values, admit authoritative state, and apply patches
  without hiding authority boundaries?
- How do identities, locators, and compatibility lowering fit into the same
  milestone?
- What exactly is frozen by the Milestone 1 readiness artifact?

Read the docs in this order if you are new to the surface:

1. [Aspect Keys, Values, And Scalar Contracts](./aspect-keys-values-and-scalar-contracts.md)
2. [Aspect Shapes And Value Carriers](./aspect-shapes-and-value-carriers.md)
3. [Struct Contracts, Fields, And Field Paths](./struct-contracts-fields-and-field-paths.md)
4. [Projection, Mutation, And Diagnostic Masks](./projection-mutation-and-diagnostic-masks.md)
5. [Validation And Authoritative State Admission](./validation-and-authoritative-state-admission.md)
6. [Authoritative Patches And Apply Flow](./authoritative-patches-and-apply-flow.md)
7. [Identities, Locators, And Blind-Consumer Addressing](./identities-locators-and-blind-consumer-addressing.md)
8. [Compatibility Lowering And JSON Bridges](./compatibility-lowering-and-json-bridges.md)
9. [Digest Preparation And Canonical Basis](./digest-preparation-and-canonical-basis.md)
10. [Grouped Public Lanes And Common-Path Usage](./grouped-public-lanes-and-common-path-usage.md)
11. [Milestone 1 Production Readiness](./milestone-1-production-readiness.md)

The order matters.

- Start with contracts and values so aspect meaning is explicit before any data
  enters authority.
- Read the concrete carrier page before validation if you need the exact Rust
  types rather than just the contract families.
- Treat scalar, reference, and opaque shapes as separate contract meanings, not
  as one generic typed-value lane.
- Learn struct fields and masks before you think about state or patches, because
  field targeting is part of the contract surface.
- Admit authoritative state before you talk about patch application.
- Treat compatibility lowering as an explicit bridge, not as another native
  constructor.
- Read digest preparation after native meaning is clear, because it freezes
  canonical ordering for later equality and digest lanes.
- Use readiness when you need the machine-checkable closure contract for the
  whole milestone.

The hardened public surface for this milestone is part of what shipped:

- `worth_foundational::aspects()`
- `worth_foundational::compatibility()`
- the named `aspect_common_path` readiness surface
- the named `compatibility_common_path` readiness surface

These docs are capability-first on purpose. They are not milestone notes,
closeout notes, or test tours. If a Milestone 1 capability shipped, it should
have one stable home in this folder.
