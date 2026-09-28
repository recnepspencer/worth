# Worth Query Docs

This index lists every page under `docs/`, grouped by audience.

- **Application API.** Application code uses `worth-query-decl` to declare
  meaning and `worth-query-host` to install and run it. Certification code
  also uses `worth-query-replay`. Start with
  [Build an Application](../../../../../docs/build-an-application.md): the
  real calls for features, programs, installation, workflows, and adoption.
  Then read the
  [Ordinary Application Front Door](./foundations/ordinary-application-front-door.md).
- **Internal engine surface.** `WorthQueryWorkspace` (`worth_query::facade`)
  is the engine surface that `worth-ui-query-binding` uses. Its pages carry an
  "Internal engine surface" banner. Application code does not import it.

## Application API (worth-query-decl / worth-query-host)

Crate guides: [`worth-query-decl`](../../worth-query-decl/README.md),
[`worth-query-host`](../../worth-query-host/README.md), and
[`worth-query-replay`](../../worth-query-replay/README.md).
Executable journeys:
[ordinary product workflow](../../worth-query-certification/examples/ordinary_product_workflow.rs),
[advanced product branching](../../worth-query-certification/examples/advanced_product_branching.rs), and
[authored workflow](../../worth-query-certification/examples/authored_workflow/main.rs).

### Start here

- [Build an Application](../../../../../docs/build-an-application.md): declare features and a program, install the application graph, run it, author and run workflows, and adopt a new program on a branch. The centerpiece.
- [Programs and Adoption](./foundations/programs-and-adoption.md): program revisions, rosters, branch adoption, migration, workflow dispositions, branch sets, and retirement.
- [Workflows](./foundations/workflows.md): workflow specs, definitions, publication, instances, approvals, assessments, conditions, retries, and cancellation.
- [Ordinary Application Front Door](./foundations/ordinary-application-front-door.md): declare, install, admit, execute, commit, and recover one application request.
- [Branches and Previews](./foundations/branches-and-previews.md): application product branches and branch-local programs; its Workspace preview section is internal.
- [AI Agent Orientation](./AI_README.md): runtime, substrate, authority, facade, and support model for AI agents and contributors.

### Declaring application meaning

- [Feature Capsule Authoring](./authoring/feature-capsule-authoring.md): state one feature's installed meaning as an `ApplicationFeatureSpec`.
- [Read Composition](./authoring/read-composition.md): turn a typed application-query declaration into one installed read graph.
- [Graph Read Access Planning](./authoring/graph-read-access-planning.md): how Query proves an installed read has the access structures and bounded cost it needs.
- [Graph Touch Obligation Authority](./authoring/graph-touch-obligation-authority.md): the graph work an installed query or operation must perform.
- [Aspects And Authority Lanes](./modeling/aspects-and-authority-lanes.md): stable slices of domain meaning and which runtime may change them.
- [Portable Query Packages](./portable-packages.md): bounded, store-neutral carriage of validated Query meaning.

### Authorization

- [Application Authorization And Emergency Elevation](./capabilities/application-authorization-and-emergency-elevation.md): product permissions derived from current application-graph facts, plus audited elevation.

### Execution, aftermath, and recovery

- [Application Aftermath, External Effects, And Recovery](./execution/application-aftermath-and-recovery.md): post-commit consequences, bounded external effects, and settlement recovery.

### Installed operations and managed computation

- [Canonical Graph Obligation Progression](./domain-capabilities/canonical-graph-obligation-progression.md): how Query admits and executes the graph work an installed operation carries.
- [Conditional Installed Operations](./domain-capabilities/conditional-installed-operations.md): eligibility, trigger, comparison, and maintenance meaning on installed operations.
- [Execution Resource Admission And Managed Runs](./domain-capabilities/execution-resource-admission-and-managed-runs.md): govern large, interruptible, or stateful domain work.
- [Installed Computation Artifact Contracts](./domain-capabilities/installed-computation-artifact-contracts.md): one portable, validated meaning for domain-produced working data.
- [Managed Artifact Ownership And Native Access](./domain-capabilities/managed-artifact-ownership-and-native-access.md): carry large or provider-native data through an installed computation.
- [Provider Sessions And Decision Read-Sets](./domain-capabilities/provider-sessions-and-decision-read-sets.md): sealed provider execution contexts and the facts that influenced a decision.
- [Provisional State And Invariant Execution](./domain-capabilities/provisional-state-and-invariant-execution.md): stage changes in an overlay and check invariants before they become authoritative.

### Certification

- [Installed Operation Certification Kit](./domain-capabilities/certification/installed-operation-certification-kit.md): `worth-query-certification` contracts for proving installed-operation meaning across providers.

## Internal engine surface (Workspace)

These pages document `WorthQueryWorkspace` and the lower `worth_query::facade`
modules. They serve engine work and `worth-ui-query-binding`.

### Workspace foundations

- [Worth Query Workspace Overview](./foundations/workspace-overview.md): the Workspace engine surface and its entry points.
- [State and Readiness Surfaces](./foundations/state.md): typed posture snapshots through `workspace.state(...)`.
- [Query Operating Modes](./foundations/query-operating-modes.md): how execution, artifacts, and subscriptions are backed today.
- [Support Matrix And Admission](./foundations/support-matrix-and-admission.md): what the runtime facade supports, defers, or fails closed.
- [Worth Query Hard Prohibitions](./foundations/hard-prohibitions.md): the hard prohibition reference.
- [Consumer Kit](./foundations/consumer-kit.md): evidence reports that prove a crate consumes `worth-query` correctly.
- [Downstream Runtime Integration](./foundations/downstream-runtime-integration.md): the contract for runtimes built on `worth-query`.
- [Operational Identity Authority](./foundations/operational-identity-authority.md): printable identities versus identities that authorize runtime work.
- [Policy, Tenant, and Relationship-Proof Narrowing](./foundations/policy-tenant-and-relationship-proof-narrowing.md): narrow what a query may see or assert.

### Query authoring

- [Typed Query Expressions And Result Shapes](./authoring/query-expressions-and-result-shapes.md): typed query shapes and result shapes before canonicalization.
- [Collections, Ordering, Aggregates, And Cursors](./authoring/collections-cursors-ordering-and-aggregations.md): collection reads, ordering, count aggregates, and cursors.
- [Scopes, Templates, Saved Queries, And View Shapes](./authoring/scopes-templates-saved-queries-and-view-shapes.md): reusable scopes, templates, and saved queries.
- [Graph Composition Authoring](./authoring/graph-composition-authoring.md): ordered multi-record graph mutations in one workspace batch.
- [Schema Validation](./modeling/schema-validation.md): the legality gate for canonical query bundles.

### Runtime surfaces

- [Live Views](./runtime-surfaces/live-views.md): durable, query-shaped runtime surfaces over authoritative truth.
- [Computed](./runtime-surfaces/computed.md): retained derived runtime state.
- [Reads, Observation, and Materialization](./runtime-surfaces/reads-observe-materialize.md): data-consumption surfaces for retained handles.
- [Granular Live Invalidation](./runtime-surfaces/granular-live-invalidation.md): update a live result from the exact truth change.
- [Region-Scoped Live Invalidation and Stream Contracts](./runtime-surfaces/region-scoped-live-invalidation-and-stream-contracts.md): locality-scoped live plans and stream contracts.

### Writes, effects, and intents

- [Writes and Intent Boundaries](./execution/writes-and-intents.md): direct workspace writes and staged intent.
- [Effects](./execution/effects.md): retained delivery or staging surfaces that react to live changes.
- [Intent Admission](./execution/intent-admission.md): inspectable eligibility decisions before execution.
- [Authority-Scoped Effect Execution](./execution/authority-scoped-effect-execution.md): effects performed by the runtime that owns their target.

### Workspace capabilities

- [Declarative Query Experience](./capabilities/declarative-query-experience.md): the Workspace declarative API for reads, live resources, history, previews, and mutations.
- [Inspection](./capabilities/inspection.md): why a retained surface or receipt has its current state.
- [Cross-Runtime Causal Inspection](./capabilities/cross-runtime-causal-inspection.md): explanations across Query, Bridge, Relational, and Signal.
- [Authoritative Mutation Evidence](./capabilities/authoritative-mutation-evidence.md): what a write batch bound, executed, and retained.
- [Basis Capability Lifecycle](./capabilities/basis-capability-lifecycle.md): choose which version of truth an operation may use.
- [Historical Basis, Diff, And Comparison Queries](./capabilities/historical-diff-and-basis.md): run and compare one query against different admitted versions.
- [Existing Truth](./capabilities/existing-truth.md): work against already authoritative entities and relations.
- [Lineage And Correspondence](./capabilities/lineage-and-correspondence.md): identity correspondence and continuity questions.
- [Structural Correspondence and Historical Materialization](./capabilities/structural-correspondence-and-historical-materialization.md): identity-evolution queries and historical envelopes.
- [Native Aspect Values](./capabilities/native-aspect-values.md): author and query `worth-foundational` value vocabulary.
- [Projection Consumption And Downstream Authority](./capabilities/projection-consumption.md): move read facts into another subsystem with sealed authority.
- [Async Resources And Result State](./capabilities/async-resources-and-result-state.md): resource- or completion-driven work and its retained state.
- [Automatic Subscription Family Selection And Diagnostics](./capabilities/subscription-selection-and-diagnostics.md): how Query picks a subscription family.

### Domain capabilities (Workspace)

- [Domain Capabilities](./domain-capabilities/README.md): index of the installed-domain Workspace surfaces.
- [Runtime-Installed Domains And Operations](./domain-capabilities/runtime-installed-domains.md): declare operations once and bind them to one runtime.
- [Installed Operation Lineage And Promotion](./domain-capabilities/installed-operation-lineage-and-promotion.md): identity outcomes of an operation and graph promotion.
- [Installed Operation Re-Execution And Replay](./domain-capabilities/installed-operation-reexecution-and-replay.md): re-execution and cert-only semantic replay.
- [Typed Stops And Remediation Guidance](./domain-capabilities/typed-stops-and-remediation-guidance.md): why work stopped and descriptive next steps.
- [Consumption Cost Evidence](./domain-capabilities/consumption-cost-evidence.md): structural work Query performed for an operation.
- [Bound Projection Lifecycle, Sharing, And Consumer Invalidation](./domain-capabilities/bound-projection-sharing-and-invalidation.md): installed projection access, sharing, and invalidation.
- [Ordinary Outcomes](./domain-capabilities/ordinary-outcomes.md): the `WorthQueryOrdinaryOutcome<T>` bound/stopped vocabulary.
- [Family Helpers](./domain-capabilities/family-helpers.md): domain-native helpers over generic Query surfaces.
- [Typed Binding Pipeline](./domain-capabilities/typed-binding-pipeline.md): turn declared context into the next admissible input.
- [Continuation Pipeline](./domain-capabilities/continuation-pipeline.md): prepare and execute continuation artifacts.
- [Contribution-Composed Orchestration](./domain-capabilities/contribution-composed-orchestration.md): one declaration-entry run with attached contributions.
- [Signal Compatibility Orchestration](./domain-capabilities/signal-compatibility-orchestration.md): compose retained surfaces into one signal-facing result.
- [Orchestration Inventory](./domain-capabilities/orchestration-inventory.md): the semantic registry of orchestration surfaces.
- [Lower-Runtime Capability Routing](./domain-capabilities/lower-runtime-capability-routing.md): declared routes to Relational, Bridge, and Signal.
- [Declaration Entry Orchestration](./domain-capabilities/declaration-entry-orchestration.md): advance one declaration to its next proof-bearing artifact.
- [Installed Domain Inspection](./domain-capabilities/declaration-entry-inspection.md): explain an installed-domain runtime product.
- [Declaration Entry Readiness](./domain-capabilities/declaration-entry-readiness.md): family-level support and readiness projection.
- [Declaration Family Taxonomy](./domain-capabilities/declaration-family-taxonomy.md): classification over downstream-owned family identity.
- [Declaration Boundary Envelopes](./domain-capabilities/declaration-boundary-envelopes.md): artifacts that carry one declaration crossing forward.
- [Declaration Boundary Receipts](./domain-capabilities/declaration-boundary-receipts.md): crossing posture recorded after route planning.
- [Declaration Bridge Continuation Routing](./domain-capabilities/declaration-bridge-continuation-routing.md): envelope to bridge-continuation binding.
- [Declaration Relational Truth Routing](./domain-capabilities/declaration-relational-truth-routing.md): envelope to relational authority binding.
- [Declaration Signal Compatibility](./domain-capabilities/declaration-signal-compatibility.md): envelope to signal compatibility artifact.
- [Grouped Authoring](./domain-capabilities/grouped-authoring.md): neighborhood-shaped work where the group carries meaning.
- [Grouped Products](./domain-capabilities/grouped-products.md): per-member routes, receipts, and envelopes of a grouped declaration.
- [Grouped Contributions](./domain-capabilities/grouped-contributions.md): shared group posture with member-local contributions.
- [Grouped Support And Readiness](./domain-capabilities/grouped-support-readiness.md): whether a grouped claim is supportable.
- [Installed Domain Closeout Evidence](./domain-capabilities/platform-entry-closeout.md): composed installation, execution, and boundary evidence.
- [Domain Capability Documentation Certification](./domain-capabilities/public-doc-coverage.md): how docs, examples, and tests stay in agreement.
- [Domain Capability Contributions](./domain-capabilities/contributions/README.md): routes to declaration-scoped contribution surfaces.
- [Capability Gaps And Invariant Denials](./domain-capabilities/invariants/capability-gaps-and-invariant-denials.md): domain-authored gap and denial posture.
- [Admission-Local Support Reports](./domain-capabilities/support/admission-local-support-reports.md): support posture tied to an admitted intent plan.
- [Cross-Runtime Fallback Vs Store-Backed Replay Gap](./domain-capabilities/explanation/cross-runtime-fallback-vs-store-backed-replay-gap.md): two explanation kinds that are not substitutes.
- [Certification Surface And Closeout Bundle](./domain-capabilities/certification/certification-surface-and-closeout-bundle.md): the machine-checkable readout of the domain-capability seam.
- [Goldens, Boundaries, And Hostile Certification](./domain-capabilities/certification/goldens-boundaries-and-hostile-certification.md): evidence surfaces that keep the seam honest.

### Choosing a Workspace surface

- [Choosing The Right Surface](./domain-capabilities/choosing/README.md): index of chooser pages.
- [Binding Vs Orchestration Vs Helpers](./domain-capabilities/choosing/binding-vs-orchestration-vs-helpers.md): pick the next-step surface.
- [Grouped Authoring Vs Grouped Products Vs Grouped Contributions](./domain-capabilities/choosing/grouped-authoring-vs-grouped-products-vs-grouped-contributions.md): pick a grouped surface.
- [Inspection Vs Cross-Runtime Explanation](./domain-capabilities/choosing/inspection-vs-cross-runtime-explanation.md): pick an evidence surface.
- [Live View Vs Subscription](./domain-capabilities/choosing/live-view-vs-subscription.md): pick an ongoing-update surface.
- [Projection Consumption Vs Inspection](./domain-capabilities/choosing/projection-consumption-vs-inspection.md): projection facts versus retained inspection evidence.
- [Signal Compatibility Vs Continuation Pipeline](./domain-capabilities/choosing/signal-compatibility-vs-continuation-pipeline.md): pick the next step for retained declaration truth.

### Workspace workflow guides

- [Workflow Guides](./domain-capabilities/workflow/README.md): index of multi-surface Workspace task guides.
- [Single Declaration To Envelope](./domain-capabilities/workflow/single-declaration-to-envelope.md): one declaration to a retained boundary envelope.
- [Envelope To Signal Or Continuation](./domain-capabilities/workflow/envelope-to-signal-or-continuation.md): the next runtime-facing step from an envelope.
- [Retained Artifact To Next Step](./domain-capabilities/workflow/retained-artifact-to-next-step.md): move retained truth to the next explicit step.
- [Grouped Neighborhood Workflow](./domain-capabilities/workflow/grouped-neighborhood-workflow.md): end-to-end grouped work.
- [Runtime-Preflight Workflow Contributions](./domain-capabilities/workflow/runtime-preflight-workflow-contributions.md): confirmation-required workflow meaning on a runtime preflight.

### Workspace recipes

- [Recipes](./domain-capabilities/recipes/README.md): index of short, copy-oriented Workspace examples.
- [Attach Material With Declaration-Scoped Contributions](./domain-capabilities/recipes/attach-material-with-declaration-scoped-contributions.md): one run with attached declaration-scoped contributions.
- [Author A Grouped Neighborhood With Contributions](./domain-capabilities/recipes/author-a-grouped-neighborhood-with-contributions.md): one grouped operation with contributions.
- [Carry Query Facts Into A Downstream Runtime](./domain-capabilities/recipes/carry-query-facts-into-a-downstream-runtime.md): hand read facts to another runtime with sealed authority.
- [Prepare Preview From An Active Face Selection](./domain-capabilities/recipes/prepare-preview-from-active-face-selection.md): helper-driven path from a selection to a preview result.

### Working on Query itself

- [Query Iteration](./iteration.md): which package to build and test for a given edit.
