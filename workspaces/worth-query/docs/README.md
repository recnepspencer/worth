# Worth Query Docs

This index lists every page under `docs/`, grouped by audience.

- **Application API.** Application code uses `worth-query-decl` to declare
  meaning and `worth-query-host` to install and run it. Certification code
  also uses `worth-query-replay` and `worth-query-certification`. Start with
  [Build an Application](../../../docs/build-an-application.md): the
  real calls for features, programs, installation, workflows, and adoption.
  Use the [API Map](../../../docs/api.md) to choose imports and the
  [Capability Index](../../../docs/capability-index.md) to find the
  facade call for a task. Then read the
  [Ordinary Application Front Door](./foundations/ordinary-application-front-door.md).

## Application API (worth-query-decl / worth-query-host)

Crate guides: [`worth-query-decl`](../crates/worth-query-decl/README.md),
[`worth-query-host`](../crates/worth-query-host/README.md), and
[`worth-query-replay`](../crates/worth-query-replay/README.md).
Executable journeys:
[ordinary product workflow](../crates/worth-query-certification/examples/ordinary_product_workflow.rs),
[advanced product branching](../crates/worth-query-certification/examples/advanced_product_branching.rs), and
[authored workflow](../crates/worth-query-certification/examples/authored_workflow/main.rs).

### Start here

- [Build an Application](../../../docs/build-an-application.md): declare features and a program, install the application graph, run it, author and run workflows, and adopt a new program on a branch. The centerpiece.
- [API Map](../../../docs/api.md): which crate and facade module to import.
- [Capability Index](../../../docs/capability-index.md): find the facade call for a task, and what to do before concluding a capability is missing.
- [Programs and Adoption](./foundations/programs-and-adoption.md): program revisions, rosters, branch adoption, migration, workflow dispositions, branch sets, and retirement.
- [Workflows](./foundations/workflows.md): workflow specs, definitions, publication, instances, approvals, assessments, conditions, retries, and cancellation.
- [Workflow Operation Inputs](./foundations/workflow-operation-inputs.md): capture a finite set of typed operation descriptions and let a native required workflow operation select one.
- [Workflow Succession Output Affinity](./foundations/workflow-succession-output-affinity.md): match an output captured by a completed workflow to the subject its successor operation selects.
- [Workflow-Enrolled Creations](./authoring/workflow-enrolled-creations.md): publish a new subject and fresh native instance in one authorized mutation.
- [Ordinary Application Front Door](./foundations/ordinary-application-front-door.md): declare, install, admit, execute, commit, and recover one application request.
- [Basis Capability Lifecycle](./capabilities/basis-capability-lifecycle.md): pointers to Admission's basis lifecycle, policy and tenant admission, and relationship-proof admission.
- [Inspect original evidence](./capabilities/cross-runtime-causal-inspection.md): audit or debug an earlier result with `request.inspect` and `request.inspect_source`.
- [Branches and Previews](./foundations/branches-and-previews.md): application product branches and branch-local programs.

### Declaring application meaning

- [Feature Capsule Authoring](./authoring/feature-capsule-authoring.md): state one feature's installed meaning as an `ApplicationFeatureSpec`.
- [Executable Scope-Field Operation Bodies](./authoring/scope-field-operation-bodies.md): author literal source guards and field assignments once, then execute their installed meaning.
- [Signed Operation Creations](./authoring/signed-operation-creations.md): publish an immutable authenticated snapshot independently of a workflow approval frontier.
- [Read Composition](./authoring/read-composition.md): turn a typed application-query declaration into one installed read graph.
- [Record And Aspect History](./capabilities/record-and-aspect-history.md): capture disclosed record generations and page canonical changes at a pinned product with fresh independent metadata permission.
- [Graph Read Access Planning](./authoring/graph-read-access-planning.md): how Query proves an installed read has the access structures and bounded cost it needs.
- [Graph Touch Obligation Authority](./authoring/graph-touch-obligation-authority.md): the graph work an installed query or operation must perform.
- [Schema Validation](./modeling/schema-validation.md): pointers to canonical query validation and Native boundary entry points.
- [Relation Laws](./modeling/relation-laws.md): stored graph laws, persistent orbit/component identities, scoped reads, disclosure and compiled topology examples; remaining projected CAD shell requirements.
- [Invariants and Read Observations](./foundations/invariants-and-read-observations.md): prepare and evaluate actual proposed-state reads with complete frozen dependencies.
- [Complete Relation Walks](./foundations/invariant-walks.md): typed complete traversal, minimum distances, access and interruption semantics.
- [Aspects And Authority Lanes](./modeling/aspects-and-authority-lanes.md): stable slices of domain meaning and which runtime may change them.
- [Portable Query Packages](./portable-packages.md): bounded, store-neutral carriage of validated Query meaning.
- [Complete Paths In Declared Queries](./foundations/declared-query-closure.md): select every live node reachable over one endorelation in a declared query.
- [Native Query Expression Contexts](./authoring/native-query-expression-contexts.md): retain a one-shot read's native projected row as an immutable expression record.
- [Atomic Child Creation](./authoring/atomic-child-creation.md), [Independent Creation Sites](./authoring/independent-creation-sites.md), [Complete-Content Selection Reuse](./authoring/complete-content-selection-reuse.md), [Retiring Selection Replacement](./authoring/retiring-selection-replacement.md) and [Retire Owned Selections When An Input Changes](./authoring/owned-selection-retirement.md): creation and selection authoring.
- [Input-Selected Existing References](./authoring/existing-reference-inputs.md) and [Compare Captured Input With A Field On A Related Entity](./authoring/path-input-guards.md): input-driven references and guards.
- [Shared Computed Body Values](./authoring/computed-body-values.md), [Computed Graph-Target Writes](./authoring/computed-path-writes.md) and [Authored Conditional External Effects](./authoring/conditional-body-effects.md): computed values, writes and conditional effects in operation bodies.
- [Graph-Certified Signature Subjects](./authoring/graph-certified-subjects.md): certify facts on related native entities in a signed body without copying them.

### Authorization

- [Application Authorization And Emergency Elevation](./capabilities/application-authorization-and-emergency-elevation.md): product permissions derived from current application-graph facts, plus audited elevation.

### Execution, aftermath, and recovery

- [Application Aftermath, External Effects, And Recovery](./execution/application-aftermath-and-recovery.md): post-commit consequences, bounded external effects, and settlement recovery.
- [Retain an original operation](./execution/durable-original-operation.md): durable custody across a graceful reopen.
- [Retained Delivery For No-Output Actions](./foundations/source-only-retained-delivery.md): a no-output action delivers a read-only observation of its original commit.
- [Reclaim Retired Storage](./execution/storage-retention.md): history windows, bookmarks and the release of obsolete owners.

### Installed operations and managed computation

- [Canonical Graph Obligation Progression](./domain-capabilities/canonical-graph-obligation-progression.md): how Query admits and executes the graph work an installed operation carries.
- [Conditional Installed Operations](./domain-capabilities/conditional-installed-operations.md): eligibility, trigger, comparison, and maintenance meaning on installed operations.
- [Granular Invalidation Batches](./runtime-surfaces/granular-invalidation-batches.md): pointers to the batch a clock observation carries.
- [Application Resources](./capabilities/application-resources.md): declare a process-local request/response resource and use it through Host.
- [Source-Aligned Output Availability](./capabilities/source-aligned-output-availability.md): observe an installed producer's current output, or its source-aligned Pending, without starting it.
- [Execution Resource Admission And Managed Runs](./domain-capabilities/execution-resource-admission-and-managed-runs.md): govern large, interruptible, or stateful domain work.
- [Installed Computation Artifact Contracts](./domain-capabilities/installed-computation-artifact-contracts.md): one portable, validated meaning for domain-produced working data.
- [Managed Artifact Ownership And Native Access](./domain-capabilities/managed-artifact-ownership-and-native-access.md): carry large or provider-native data through an installed computation.
- [Provider Sessions And Decision Read-Sets](./domain-capabilities/provider-sessions-and-decision-read-sets.md): sealed provider execution contexts and the facts that influenced a decision.
- [Provisional State And Invariant Execution](./domain-capabilities/provisional-state-and-invariant-execution.md): stage changes in an overlay and check invariants before they become authoritative.

### Certification

- [Installed Operation Certification Kit](./domain-capabilities/certification/installed-operation-certification-kit.md): `worth-query-certification` contracts for proving installed-operation meaning across providers.

## Working on Query itself

- [Query Engine Architecture Map](./AI_README.md): owner crates, request-stage ownership, engine laws, and prohibited shortcuts for engine maintainers. It names engine-internal types that application code must not import.
- [Query Iteration](./iteration.md): which package to build and test for a given edit.
