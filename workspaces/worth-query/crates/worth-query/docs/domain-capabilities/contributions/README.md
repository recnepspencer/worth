# Domain Capability Contributions

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Page Is

This page routes domain authors to the public contribution surfaces that attach
declaration-scoped meaning without replacing Query admission, support, or
execution authority.

A contribution describes domain posture associated with a declaration. Query
validates, composes, and materializes that posture through the installed domain
boundary. A contribution never authorizes an operation by itself.

## Contribution Families

Use contributions for current declaration-scoped meaning such as:

- admission advice and violations;
- support and lower-runtime traceability;
- workflow posture;
- continuity;
- explanation;
- invariant and capability posture.

When several families participate in one declaration flow, use
[Contribution-Composed Orchestration](../contribution-composed-orchestration.md).
When the declaration describes a neighborhood rather than one isolated member,
use [Grouped Contributions](../grouped-contributions.md).

## Current Entry Points

- [Contribution-Composed Orchestration](../contribution-composed-orchestration.md)
- [Grouped Contributions](../grouped-contributions.md)
- [Admission-Local Support Reports](../support/admission-local-support-reports.md)
- [Runtime-Preflight Workflow Contributions](../workflow/runtime-preflight-workflow-contributions.md)
- [Cross-Runtime Explanation Gaps](../explanation/cross-runtime-fallback-vs-store-backed-replay-gap.md)

## Quick Rules

- Admission contributions annotate legality; they do not bypass `admit_*` APIs.
- Support contributions attach traceability; they do not replace the runtime
  support matrix.
- Explanation contributions do not replace causal inspection.
- Invariant contributions do not register or execute invariants.
- Grouped contributions preserve member-local meaning instead of looping over
  unrelated single-member declarations.
- A materialized contribution remains bound to its declaration and installed
  operating world.

## Related Docs

- [Domain Capabilities](../README.md)
- [Choosing The Right Surface](../choosing/README.md)
- [Public Documentation Coverage](../public-doc-coverage.md)
