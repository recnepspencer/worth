# worth-query-decl

`worth-query-decl` is the declaration-audience facade for application schema
and Query meaning. Application and entry-band domain crates use it to declare
typed entities, relations, aspects, fields, queries, operations, capabilities,
policies, and principal bindings.

```rust
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

## Ordinary Resource Declarations

Declare query result cardinality with `ApplicationQueryBindingLimits::results(n)`
or the query-binding macro's `limits results n`. The installed host owns the
finite work safeguard. `bounded(n, work)` (macro `limits results n, work w`)
adds a deliberate cap; ordinary code does not count internal traversal work.

For candidate resources, `ApplicationCandidateResourceCeiling::representation_bytes(n)`
lets Query derive validator allowance from installed invariant contracts. The
mutation-binding macro likewise allows `resources retained_representation_bytes n`
without `validator_work`. Explicit validator caps, effect cardinalities and byte
limits remain enforced. These declarations grant no runtime capacity or authority.

## Application Program Meaning

**Writing an application?** Start with [Build an Application](../../../../docs/build-an-application.md). Its
sections 3 and 6 show features, the program, and workflow authoring with real
code.

A program is built from features. Each `ApplicationFeature` owns actions,
declared with an `ApplicationFeatureSpec`:

```rust,ignore
use worth_query_decl::facade::application_program::*;

impl ApplicationProgramDefinition<MySchema> for MyProgram {
    type Contributions = (MyContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("example.program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<MySchema, MyFeature>()
            .mutation::<MyMutationBinding>()
            .operation::<MyOperation>()
            .finish()]
    }
}

let validated = ApplicationProgramAuthoring::<MySchema, MyProgram>::begin()
    .validated_program()?;          // ValidatedApplicationProgram<MySchema, MyProgram>
let revision = validated.revision(); // ApplicationProgramRevision
```

`ApplicationProgramDefinition<Schema>` owns canonical authored program meaning.
Validation produces a `ValidatedApplicationProgram`, whose `revision()` is the
declaration-owned `ApplicationProgramRevision`, and a deterministic versioned
description. The revision includes feature/composition
identity, ports and connections, action contracts, rule owner/scope/version,
output lineage, derived-resource posture, and continuation/effect meaning.
Rust file names, registration order, diagnostics, and generated text do not
define compatibility.

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
- [WORTH Query Orientation](../worth-query/docs/AI_README.md)
- [Feature Capsule Authoring](../worth-query/docs/authoring/feature-capsule-authoring.md)
- [Read Composition](../worth-query/docs/authoring/read-composition.md)
- [Query Docs Index](../worth-query/docs/README.md)
