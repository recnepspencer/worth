# Milestone 5.6 Build Checklist

> **Purpose:** implementation-time enforcement checklist for
> [milestone-5.6.md](./milestone-5.6.md)
>
> **Use this while building.** No new 5.6 surface is complete until its
> checklist rows are satisfied in code, tests, and certification.

## Core Rule

Every new 5.6 surface must close all three layers:

1. production shape
2. compile-time boundary
3. certification proof

If one of those layers is missing, the surface is not done.

## Global Do / Don't Gate

### Do

- add one explicit capability witness per admitted capability family
- keep config sections subsystem-owned
- validate root config into a proof-bearing validated config artifact before
  capability admission
- derive support reporting from the same registry/matrix used by admission
- add compile-fail tests when a boundary matters
- add certification rows for both success and denial paths
- assert both incremented and zero-stayed-zero counters

### Don't

- do not add `capability(family)` or other runtime-selected witness routing
- do not add bool-driven shortcut surfaces
- do not add capability-shaped config bags
- do not infer support from method presence
- do not probe lower runtimes to discover support at acquisition time
- do not add new 5.6 composition-first APIs only on the legacy broad facade
- do not collapse unsupported, deferred, and invalid-config failures into one
  generic error

## Capability Family Checklist

Run this list once for every admitted capability family:

- witness type exists and is private-constructor/proof-bearing
- facade has one statically named acquisition method for the family
- acquisition consumes `ValidatedWORTHQueryConfig`, not raw root config
- support registry has one explicit family entry
- support matrix carries admitted/deferred/unsupported posture for the family
- support report exposes the family in machine-checkable form
- typed denial exists for unsupported family
- typed denial exists for deferred family
- typed denial exists for missing-owning-section case
- unit tests cover admitted acquisition
- unit tests cover typed denial paths
- compile-fail test proves external construction is forbidden
- compile-fail test proves cross-family witness misuse is forbidden
- certification row proves admitted family behavior
- certification row proves denial behavior
- counter assertions verify lookup/denial exactness
- unrelated counters are asserted zero where applicable

## Config Section Checklist

Run this list once for every owning config section:

- section represents a real subsystem owner, not a capability family
- section fields are owned only by that subsystem
- root config construction requires the section explicitly where applicable
- validated config propagation requires the section explicitly where applicable
- adding the section forces compile errors at construction/propagation boundaries
- config validation emits a typed failure when the section is missing
- config validation emits a typed failure when the section posture is
  contradictory
- unrelated capability acquisition ignores the section cleanly
- support report records section-derived posture honestly
- unit tests cover present/valid section
- unit tests cover missing section
- unit tests cover contradictory section
- certification row proves section explicitness
- counter assertions verify section-resolution behavior

## Support Metadata Checklist

Run this list whenever registry, matrix, or report logic changes:

- `WORTHQueryCapabilityRegistry` and `WORTHQuerySupportMatrix` are the only
  support authorities
- `WORTHQuerySupportReport` is derived from registry/matrix artifacts, not a
  second hand-maintained summary path
- report includes admitted families
- report includes deferred families
- report includes unsupported families
- report includes config-section posture used for the result
- report includes canonical digests or equivalent machine-checkable identity
- unit tests compare support report against real admission behavior
- certification row proves support-metadata synchronization

## Failure Topology Checklist

Run this list whenever a new denial path is introduced:

- failure class is typed, not stringly
- unsupported vs deferred vs invalid-config are distinct failure classes
- missing-owning-section is distinct from contradictory-section posture
- capability-family denial is distinct from config-validation denial
- unit tests match on failure class, not error text
- certification row verifies typed failure
- counters distinguish denial classes exactly

## Compile-Fail Checklist

Before calling a 5.6 boundary closed, confirm compile-fail coverage exists for:

- external construction of capability witnesses
- external construction of support-owned artifacts that should stay sealed
- cross-family witness misuse
- bool-driven shortcut construction
- runtime-selected `capability(family)` or equivalent dynamic routing
- direct use of internal `application/*` modules outside the facade boundary
- new 5.6 composition-first surfaces exposed only through the legacy facade

## Certification Checklist

Before calling the milestone or a major batch complete, confirm the named suite
covers:

- unified query-read capability admission
- unified live capability admission
- unified preview capability admission
- unified workflow capability admission
- unified historical capability admission
- unified configuration section explicitness
- support-metadata and executable-admission synchronization
- unsupported composed capability denial
- deferred capability denial
- invalid unified configuration denial
- legacy broad-facade shortcut forbidden

And for each relevant row:

- hostile lane exists
- parity or comparison lane exists where applicable
- equality assertions exist where parity is claimed
- inequality assertions exist where semantics must differ
- typed-failure assertions exist for rejections
- zero assertions exist for forbidden fallback or widening
- exact counter assertions exist

## Legacy Facade Guardrail

Every 5.6 implementation PR/batch should answer these questions explicitly:

- did this add any new composition-first API on the legacy broad facade?
- if yes, why is that not a 5.6 regression?
- can the same task be completed through the application facade instead?
- does the new surface make the legacy wall more operationally dominant?

If the answer trends toward "yes", stop and redesign.

## Batch Closeout Gate

A 5.6 implementation batch is not ready to merge until all of these are true:

- production artifacts exist
- proof-bearing boundaries are sealed
- compile-fail boundaries exist
- unit tests cover success and denial
- certification rows exist
- counters are asserted exactly
- support metadata matches executable admission
- no new bag-shaped or legacy-wall shortcuts were introduced

## Recommended PR Template

For each 5.6 implementation batch, include:

- capability families touched
- config sections touched
- new witness types
- new failure classes
- new compile-fail tests
- new certification rows
- exact counters asserted
- confirmation that no legacy-facade-only 5.6 API was added
