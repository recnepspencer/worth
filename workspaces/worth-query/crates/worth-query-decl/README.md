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

## Application Program Meaning

`ApplicationProgramDefinition<Schema>` owns canonical authored program meaning.
Validation produces a declaration-owned `ApplicationProgramRevision` and a
deterministic versioned description. The revision includes feature/composition
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

## Related Docs

- [Ordinary Application Front Door](../worth-query/docs/foundations/ordinary-application-front-door.md)
- [Branches And Previews](../worth-query/docs/foundations/branches-and-previews.md)
- [WORTH Query Orientation](../worth-query/docs/AI_README.md)
- [Declarative Query Experience](../worth-query/docs/capabilities/declarative-query-experience.md)
- [Query Expressions And Result Shapes](../worth-query/docs/authoring/query-expressions-and-result-shapes.md)
