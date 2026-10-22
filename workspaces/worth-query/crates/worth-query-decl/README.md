# worth-query-decl

`worth-query-decl` is the declaration-audience facade for application schema
and Query meaning. Application and entry-band domain crates use it to declare
typed entities, relations, aspects, fields, queries, operations, capabilities,
policies, and principal bindings.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
use worth_query_decl::facade::{
    application_capability,
    application_query,
    application_schema,
};
```

Declarations describe portable application intent. They do not install a
runtime, authenticate a caller, authorize a request, execute a provider, or
publish a result. Application hosts perform those transitions through
`worth-query-host`.

Pure reusable schema-meaning crates remain Query-agnostic. Query declaration
integration belongs in the application entry band.

## Query Result Controls

Declare query result cardinality with `ApplicationQueryBindingLimits::results(n)`
or the query-binding macro's `limits results n`. This is a semantic result limit,
not a traversal-work estimate. The Host query profile controls simultaneous graph
work; ordinary query declarations supply no work or byte invoice.

Mutation bindings declare typed input, source expectations, scope, operation and
outputs. They do not declare candidate byte estimates or validator-work quotes.
Handlers build the actual candidate through the Host's `CandidateWriter`;
installed invariants check it before atomic publication. These declarations grant
no runtime capacity or authority.

## Application Program Meaning

**Writing an application?** Start with [Build an Application](../../../../docs/build-an-application.md). Its
sections 3 and 6 show features, the program, and workflow authoring with real
code.

A program is built from features. For ordinary installed features, use the Host
facade's [complete feature definition](../../../../docs/build-an-application.md#complete-feature-definitions).
One typed member body supplies actions, delivery roots, handler slots and required
Initial/Preserve roles. `ApplicationFeatureDefinition` projects that body into
schema members, canonical feature specs and program outputs; Host adds contracts
and explicit runtime configuration. Select the definitions once instead of
maintaining independent declaration and registration lists.

For specialized declaration authoring, begin an action with
`.action::<Binding, Delivery>()` on `ApplicationFeatureSpec::root` or `at`.
Compose the paired `.locality_and_change::<Scope, Shape>()`, binding-scoped
`.correspondence::<C>()`, operation-scoped `.evaluated_requirement::<Rule>()`
and `.external_input::<Provider>()` as applicable. Each clause can be attached
only once. `.finish_action()` appends one complete declaration and returns the
feature parent; only that parent can add members or `.finish()` the feature.
Delivery is mandatory for ordinary actions and derives required-output meaning.

The macro's `action(Binding, Delivery) { ... };` block delegates to the same
progression, including empty blocks and clause-order-independent meaning.
Deliberately specialized native actions use `.native_action::<Binding>()` or
`native_action(Binding) { ... };` with the same optional clauses. They have no
delivery or output-source toggle; actual roots derive native membership.
Ordinary mutations and retries require their real delivery.

`ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>`
implements sealed `ApplicationAction`. Its `selector()` returns only
`ApplicationActionSelector`; Installation resolves the complete actual action.
No public selector conversion manufactures full declaration meaning. The
existing validator refuses duplicate native binding types and portable
occurrence/feature/binding identity keys, while allowing distinct legal instances.

Correspondence capabilities remain binding-scoped; evaluated rules and providers
remain operation-plus-rule/provider scoped across declared actions. Installing
metadata neither evaluates a rule nor admits an external revision. The installed
evaluator supplies guidance and submission enforcement, not a new authorization
engine. Use admitted external values in the source intent. Validate complete
feature specs with `ApplicationProgramAuthoring::<Schema, Program>::begin()`
followed by `.validated_program()`.

This [real consumer declaration](../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_program/composed_action.rs)
uses the source module's surrounding typed imports and feature members:

<!-- compiled: composed_action.rs -->
```rust
pub(crate) fn source_feature_spec() -> ApplicationFeatureSpec {
    features::source_builder()
        .action::<PlanarSourceAdjustmentBinding<ConsumerSchema>, ConsumerFixedDelivery>()
        .locality_and_change::<SourceLocality, SourceChange>()
        .evaluated_requirement::<CatalogInputRequirement>()
        .external_input::<external_input::NeutralExternalProvider>()
        .finish_action()
        .finish()
}
```

`ApplicationProgramDefinition<Schema>` owns canonical authored program meaning.
Validation produces a `ValidatedApplicationProgram`, whose `revision()` is the
declaration-owned `ApplicationProgramRevision`, and a deterministic versioned
description. The revision includes feature/composition
identity, ports and connections, action contracts, rule owner/scope/version,
output lineage, derived-resource posture, and continuation/effect meaning.
Rust file names, registration order, diagnostics, and generated text do not
define compatibility.

`ApplicationOutputGraph<Connection, Dependents>` remains the required fixed-root
posture. `ApplicationDemandOutputGraph<Connection, Dependents>` installs the same
typed connection tree for explicit assessment without adding its source binding
to the required-output publication roster. The latter records a declaration-owned
`demand-only` root posture in canonical meaning; existing required graphs keep
their canonical records. Explicit required-source action declarations still apply.

Both implement the sealed `ApplicationFixedOutputRoot` used by native
`start_program_outputs` and explicit `demand(...).start_in_program` progression. Only
`ApplicationRequiredOutputRoot` may enter performed/prepared-source, required
recovery, or required-unavailable completion APIs. A demand-only assessment does not make its source a required-output action.
Ordinary source mutation uses its declared source-only owning delivery; a
separate explicit assessment demand retains native output owners and checks the installed artifact's actual source locality, change, dependency,
request authority and currentness. A receipt cannot substitute for those owners.
For an optional assessment, declare its artifact `REQUIRED = false`; retained
storage and the artifact's other lifecycle meanings are separate choices.

Hosts may roster multiple validated revisions simultaneously. A decoded
program description is bounded diagnostic/compatibility input only: it cannot
be cast into a validated program, installed support, branch activation,
migration preparation, or recovery authority. Installation compares canonical
meaning and returns typed support, migration, and custody requirements; the
execution owner decides whether one exact branch can adopt the target.

## Workflow Definitions

`application_program` also declares authored workflow definitions.
`ApplicationWorkflowDefinitionBuilder` builds a definition from nodes,
optionally with reusable components (`ApplicationWorkflowComponentBuilder`),
and validation produces a `ValidatedWorkflowDefinition`. The
`worth_query_workflow!` macro is shorthand for the same builder. Hosts
publish, discover, start, and progress those definitions through
`worth-query-host`; the
[authored workflow example](../worth-query-certification/examples/authored_workflow/main.rs)
shows the complete journey, and the
[workflows guide](../worth-query/docs/foundations/workflows.md) documents it
step by step.

Workflow declarations also make three boundaries explicit:

- `ApplicationWorkflowSpec::observation_policy()` and `actor_policy()` declare
  governed disclosure and recorded-role policy; see [workflow actors](../worth-query/docs/foundations/workflows.md#recorded-workflow-actors).
- `ApplicationMutationBinding::workflow_execution_requirements()` returns an
  explicit `ApplicationWorkflowExecutionRequirements`; see [guarded operations](../../../../docs/build-an-application.md#65-who-may-run-a-guarded-operation).
- `ApplicationCapabilityRecordedActorRule` in `application_capability` declares
  how a command relates to a recorded native actor; see [actor declarations](../worth-query/docs/foundations/workflows.md#declare-evidence-not-authorization-modes).

These declarations describe policy. Hosts still require fresh native admission;
an observed actor or workflow locator grants no execution permission.

An external-effect operation may declare one fixed inbound source, protocol,
version and finite `ApplicationInboundOccurrenceLimits` with
`external_effect_with_inbound`. The same typed binding can be used by a
definition's `await_inbound` node, which must name the preceding operation
node. These declarations grant neither verifier authority nor a workflow
resume capability. Bank's
[payment declaration](../../../worth-query-bank-world/crates/bank-domain/src/schema/contributions/payments.rs)
and [definition](../../../worth-query-bank-world/crates/bank-server/src/application_definition/workflows.rs)
are compiled consumers of this shape.

## Related Docs

- [Build an Application](../../../../docs/build-an-application.md): the centerpiece guide
- [Programs And Adoption](../worth-query/docs/foundations/programs-and-adoption.md)
- [Workflows](../worth-query/docs/foundations/workflows.md)
- [Ordinary Application Front Door](../worth-query/docs/foundations/ordinary-application-front-door.md)
- [Branches And Previews](../worth-query/docs/foundations/branches-and-previews.md)
- [Query Engine Architecture Map](../worth-query/docs/AI_README.md): engine-internal, for Query maintainers
- [Feature Capsule Authoring](../worth-query/docs/authoring/feature-capsule-authoring.md)
- [Read Composition](../worth-query/docs/authoring/read-composition.md)
- [Query Docs Index](../worth-query/docs/README.md)
