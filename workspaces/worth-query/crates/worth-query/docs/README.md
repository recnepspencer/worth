# Worth Query Docs

These docs are organized by working category, not by implementation order.

Start with the section that matches the job you are doing. Use
`domain-capabilities/` when a downstream domain wants Query to own the
artifact, orchestration, grouping, continuation, and typed-stop model.

## Table Of Contents

- [Start Here](#start-here)
  Fastest entry points by broad job.
- [Best Starting Points](#best-starting-points)
  Shortcuts for the most common Query doc searches.

## Start Here

For the current product-branch entry path, start with
[Ordinary Application Front Door](./foundations/ordinary-application-front-door.md)
and [Branches And Previews](./foundations/branches-and-previews.md). The
certification package contains executable
[ordinary](../../worth-query-certification/examples/ordinary_product_workflow.rs)
and [advanced](../../worth-query-certification/examples/advanced_product_branching.rs)
public journeys.

- [Ordinary Application Front Door](./foundations/ordinary-application-front-door.md)
  The supported declaration, host installation, request admission, execution,
  recovery, conditional-operation, and publication journey.
- [AI Agent Orientation](./AI_README.md)
  Canonical runtime, substrate, authority, facade, and support model for AI
  agents and contributors.
- [Portable Query Packages](./portable-packages.md)
  Bounded typed export of validated package meaning for inspection and later
  store-neutral carriage.
- [Domain Capabilities](./domain-capabilities/README.md)
  Typed domain entry, declaration pipeline, helpers, grouped work, remediation,
  continuation, and certification.
- `foundations/`
  Runtime posture, support, operating modes, consumer proof, policy/tenant
  narrowing, workspace, and preview or branch context.
- `authoring/`
  Query authoring, read composition, collections, ordering, aggregates, cursor
  boundaries, parallel admission, templates, and reusable shapes.
- `runtime-surfaces/`
  Live views, region-scoped live, computed reads, and retained read or
  materialize surfaces.
- `execution/`
  Writes, effects (authoring + authority-scoped execution), and intent admission.
- `capabilities/`
  Inspection, causal inspection, mutation evidence, basis lifecycle, projection
  consumption, historical/structural correspondence, lineage, and diagnostics.
- `modeling/`
  Aspects, authority lanes, and schema or modeling guidance.

## Best Starting Points

- if you are building an application through the supported Query entry crates:
  [Ordinary Application Front Door](./foundations/ordinary-application-front-door.md)
- if you are building domain workflows with Query:
  [Domain Capabilities](./domain-capabilities/README.md)
- if you are installing typed operations and executing them through one bound
  authority chain:
  [Runtime-Installed Domains And Operations](./domain-capabilities/runtime-installed-domains.md)
- if you are authoring conditional or triggered operation nodes:
  [Conditional Installed Operations](./domain-capabilities/conditional-installed-operations.md)
- if you need ordinary workflow re-execution or cert-only semantic replay:
  [Installed Operation Re-Execution And Replay](./domain-capabilities/installed-operation-reexecution-and-replay.md)
- if a typed stop needs explanation and descriptive next-step guidance:
  [Typed Stops And Remediation Guidance](./domain-capabilities/typed-stops-and-remediation-guidance.md)
- if you need trace-bound identity evolution, persistent naming, or sparse
  graph promotion:
  [Installed Operation Lineage And Promotion](./domain-capabilities/installed-operation-lineage-and-promotion.md)
- if an installed projection needs declaration-indexed native access, managed
  lifecycle, shared live execution, or exact capability-bound invalidation:
  [Bound Projection Lifecycle, Sharing, And Consumer Invalidation](./domain-capabilities/bound-projection-sharing-and-invalidation.md)
- if you need exact structural cost evidence or a derived Foundational
  counter-backed receipt:
  [Consumption Cost Evidence](./domain-capabilities/consumption-cost-evidence.md)
- if you need the shortest chooser path inside domain work:
  [Choosing The Right Surface](./domain-capabilities/choosing/README.md)
- if you want task-first guides for common multi-surface Query jobs:
  [Workflow Guides](./domain-capabilities/workflow/README.md)
- if you want short copy-oriented examples for common Query tasks:
  [Recipes](./domain-capabilities/recipes/README.md)
- if you need runtime posture and support context first:
  [foundations/support-matrix-and-admission.md](./foundations/support-matrix-and-admission.md)
- if you need downstream consumer proof, support pins, audits, generic
  consumer-residue-audit coverage, or test workspaces:
  [foundations/consumer-kit.md](./foundations/consumer-kit.md)
- if you need store-backed vs runtime-backed honesty:
  [foundations/query-operating-modes.md](./foundations/query-operating-modes.md)
- if you need policy, tenant, or relationship-proof narrowing:
  [foundations/policy-tenant-and-relationship-proof-narrowing.md](./foundations/policy-tenant-and-relationship-proof-narrowing.md)
- if you need installed product authorization, emergency elevation, or the
  command-versus-governed-bound distinction:
  [capabilities/application-authorization-and-emergency-elevation.md](./capabilities/application-authorization-and-emergency-elevation.md)
- if a mutation performed but publication settlement failed, or it declares
  aftermath, an external effect, idempotent dispatch, or receipt-bound recovery:
  [execution/application-aftermath-and-recovery.md](./execution/application-aftermath-and-recovery.md)
- if you need exact branch observations, repeatable bases, branch-local
  publication, or typed merge settlement repair:
  [foundations/branches-and-previews.md](./foundations/branches-and-previews.md)
- if you are executing a generic effect or batch and must distinguish denial
  from performed-but-unsettled work:
  [execution/authority-scoped-effect-execution.md](./execution/authority-scoped-effect-execution.md)
- if you need basis capability lifecycle rather than raw identifiers:
  [capabilities/basis-capability-lifecycle.md](./capabilities/basis-capability-lifecycle.md)
- if you need cross-runtime “why” (not `workspace.inspections()?.inspect` alone):
  [capabilities/cross-runtime-causal-inspection.md](./capabilities/cross-runtime-causal-inspection.md)
- if you need the installed obligation, graph-read plan, session, owner
  execution, terminal, and publication chain:
  [domain-capabilities/canonical-graph-obligation-progression.md](./domain-capabilities/canonical-graph-obligation-progression.md)
- if you need graph read access planning, admitted access postures, required
  index/materialization capability, or no-N+1 receipt proof:
  [authoring/graph-read-access-planning.md](./authoring/graph-read-access-planning.md)
- if you need lower-runtime routing (not direct crate imports):
  [domain-capabilities/lower-runtime-capability-routing.md](./domain-capabilities/lower-runtime-capability-routing.md)
- if a committed truth change should update only the affected live projection,
  collection role, or consumer:
  [Granular Live Invalidation](./runtime-surfaces/granular-live-invalidation.md)
- if you need contribution lane map:
  [domain-capabilities/contributions/README.md](./domain-capabilities/contributions/README.md)
- if you need aspect and authority semantics first:
  [modeling/aspects-and-authority-lanes.md](./modeling/aspects-and-authority-lanes.md)
- if you need exact scalar/struct authoring, predicate, and projection value
  semantics:
  [capabilities/native-aspect-values.md](./capabilities/native-aspect-values.md)

## Foundations (feature docs)

- [Ordinary Application Front Door](./foundations/ordinary-application-front-door.md)
- [Workspace Overview](./foundations/workspace-overview.md)
- [Operational Identity Authority](./foundations/operational-identity-authority.md)
- [Branches And Previews](./foundations/branches-and-previews.md)
- [Consumer Kit](./foundations/consumer-kit.md)
- [Query operating modes](./foundations/query-operating-modes.md)
- [Policy, tenant, and relationship-proof narrowing](./foundations/policy-tenant-and-relationship-proof-narrowing.md)

## Authoring (feature docs)

- [Collections, ordering, aggregates, and cursors](./authoring/collections-cursors-ordering-and-aggregations.md)
- [Graph Composition Authoring](./authoring/graph-composition-authoring.md)
- [Graph Read Access Planning](./authoring/graph-read-access-planning.md)
- [Graph Touch Obligation Authority](./authoring/graph-touch-obligation-authority.md)

## Execution (feature docs)

- [Intent Admission](./execution/intent-admission.md)
- [Writes And Intent Boundaries](./execution/writes-and-intents.md)
- [Projection consumption and downstream authority](./capabilities/projection-consumption.md)
- [Authority-scoped effect execution](./execution/authority-scoped-effect-execution.md)
- [Application aftermath, external effects, and recovery](./execution/application-aftermath-and-recovery.md)

## Runtime surfaces (feature docs)

- [Granular live invalidation](./runtime-surfaces/granular-live-invalidation.md)
- [Region-scoped live invalidation and stream contracts](./runtime-surfaces/region-scoped-live-invalidation-and-stream-contracts.md)

## Capabilities (feature docs)

- [Declarative Query Experience](./capabilities/declarative-query-experience.md)
- [Application authorization and emergency elevation](./capabilities/application-authorization-and-emergency-elevation.md)
- [Basis capability lifecycle](./capabilities/basis-capability-lifecycle.md)
- [Subscription selection and diagnostics](./capabilities/subscription-selection-and-diagnostics.md)
- [Historical diff and basis](./capabilities/historical-diff-and-basis.md)
- [Cross-runtime causal inspection](./capabilities/cross-runtime-causal-inspection.md)
- [Authoritative mutation evidence](./capabilities/authoritative-mutation-evidence.md)
- [Structural correspondence and historical materialization](./capabilities/structural-correspondence-and-historical-materialization.md)
- [Native aspect values](./capabilities/native-aspect-values.md)

## Domain capabilities

- [Canonical graph obligation progression](./domain-capabilities/canonical-graph-obligation-progression.md)
- [Runtime-installed domains and operations](./domain-capabilities/runtime-installed-domains.md)
- [Conditional installed operations](./domain-capabilities/conditional-installed-operations.md)
- [Installed operation re-execution and replay](./domain-capabilities/installed-operation-reexecution-and-replay.md)
- [Typed stops and remediation guidance](./domain-capabilities/typed-stops-and-remediation-guidance.md)
- [Installed operation lineage and promotion](./domain-capabilities/installed-operation-lineage-and-promotion.md)
- [Consumption cost evidence](./domain-capabilities/consumption-cost-evidence.md)
- [Lower-runtime capability routing](./domain-capabilities/lower-runtime-capability-routing.md)
- [Contributions hub](./domain-capabilities/contributions/README.md)
- Choosers: [live vs subscription](./domain-capabilities/choosing/live-view-vs-subscription.md), [inspection vs cross-runtime explanation](./domain-capabilities/choosing/inspection-vs-cross-runtime-explanation.md), [projection vs inspection](./domain-capabilities/choosing/projection-consumption-vs-inspection.md)
