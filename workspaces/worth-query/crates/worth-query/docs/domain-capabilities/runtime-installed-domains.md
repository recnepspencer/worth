# Runtime-Installed Domains And Operations

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Runtime-installed domains let a domain declare stable Query operations once
and bind them to one concrete runtime. Use this when operation identity,
parameters, reads, graph participation, workflow, replay, lineage,
publication, promotion, support, or cost must mean the same thing for every
caller.

The public surface is `worth_query::facade::domain`. Domain packages contain
portable meaning. Runtime construction supplies volatile providers. A
workspace then exposes one operating-world root that mints runtime-affine bound
operations.

## Why You Use It

- Give domain operations one canonical definition and conflict boundary.
- Keep provider callbacks and runtime-local resources out of portable packages.
- Reject foreign runtimes, stale generations, and marker lookalikes before
  graph or executor work.
- Carry basis, graph, provider, support, result-state, receipt, and counter
  evidence through one move-only journey.
- Publish and consume Query facts without reconstructing authority from rows or
  digests.
- Re-execute, certify replay equivalence, and carry identity lineage without
  starting parallel runtime systems.

## Stable Entry Points

- `domain::WorthQueryDomainPackage::declare(...)`
- `domain::WorthQueryDomainOperationDefinition::new(...)`
- `domain::WorthQueryDomainOperationSemanticClosure`
- `domain::WorthQueryExecutableDomainOperation`
- `domain::WorthQueryDomainOperationExecutor`
- `runtime::WorthQueryRuntime::builder()`
- `runtime::WorthQueryRuntimeBuilder::domain_package(...)`
- `runtime::WorthQueryRuntimeBuilder::domain_operation_executor(...)`
- `runtime::WorthQueryWorkspace::domain(...)`
- `runtime::WorthQueryWorkspace::observe_operating_world()`
- `runtime::WorthQueryWorkspace::prepare_mutation_operating_world()`
- `domain::WorthQueryOperationFamilyView::bind(...)`
- `domain::WorthQueryBoundDomainOperation`
- `domain::WorthQueryBoundDomainOperation::reexecute(...)`
- `domain::WorthQueryCompletedWorkflowTrace::lineage_report()`

Conditional nodes, graph providers, and workflows extend this same setup and
binding path. They are not separate runtime roots.

## Core Mental Model

There are four distinct things:

1. A package declares portable domain meaning.
2. Runtime construction registers exact volatile mechanics.
3. An installed domain handle proves one package is present in one runtime
   generation.
4. An operating world combines that runtime with one admitted basis and mints
   a bound operation.

The portable operation definition is authoritative for semantics. It includes
the canonical query and result shape, graph-read contract, workflow and
conditional declarations, replay comparator posture, reversal and
postcondition contract, external-effect and application-aftermath contracts,
lineage and promotion contract, publication posture, terminal states, failure
classes, support requirements, cost contract, and lowering identity.

The executor is authoritative only for volatile lowering mechanics. It must
report the same lowering family, determinism, read declaration, and cost shape
as the installed definition. Runtime construction rejects disagreement.

The installed execution index is derived. Query can rebuild it from installed
artifacts with identical identities, lookups, denials, and counters.

## How It Executes

```text
declare portable package and operation meaning
  -> register exact volatile providers
  -> build and publish one runtime
  -> obtain the installed domain handle
  -> create one operating world from an admitted basis
  -> borrow an operation-family view and bind
  -> execute directly or advance the installed workflow
  -> retain commit, aftermath, and external-dispatch posture
  -> publish, consume, and settle when publication is declared
```

Package and runtime construction perform structural admission once. Binding
performs current runtime, generation, basis, graph, required-domain, support,
and provider admission before execution can contact lower runtimes.

## Installed Domain Handles And Operating Contexts

A package is semantic setup data, not executable authority. Installation is
atomic: Query validates and lowers every declared family before publishing the
runtime. A failure cannot leave a partial operation, invariant, graph
obligation, contribution, or declaration-family registry behind.

The installed handle proves three facts together:

1. which domain package authorized the work;
2. which runtime installed it;
3. which installation generation is current.

The handle also remains the entry for installed declarations, contributions,
live work, inspection, rebind, and operation resolution. Those surfaces consume
the retained handle authority; they are not parallel setup roots.

An operating context names stable semantic fields of one domain world, such as
tenant, unit regime, tolerance policy, or modeling assumptions. Domain code
supplies those fields through
`WorthQueryDomainOperatingContextIdentityDeclaration`; Query canonicalizes and
seals them. Field order is not identity, and domain code never authors the
identity digest.

A domain crate can add native vocabulary with an extension trait over the
generic installed handle:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CatalogDomain;

impl domain::WorthQueryDomainEntryMarker for CatalogDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.catalog.products"
    }

    fn display_name(&self) -> &'static str {
        "Product Catalog"
    }

    fn required_capability_families(
        &self,
    ) -> &'static [domain::WorthQueryCapabilityFamily] {
        &[domain::WorthQueryCapabilityFamily::QueryRead]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CatalogContext {
    tenant: &'static str,
    pricing_regime: &'static str,
}

impl domain::WorthQueryDomainOperatingContext<CatalogDomain> for CatalogContext {
    fn required_capability_families(
        &self,
    ) -> &'static [domain::WorthQueryCapabilityFamily] {
        &[domain::WorthQueryCapabilityFamily::QueryRead]
    }

    fn required_config_sections(
        &self,
    ) -> &'static [domain::WorthQueryConfigSectionFamily] {
        &[domain::WorthQueryConfigSectionFamily::Query]
    }

    fn context_identity(
        &self,
    ) -> domain::WorthQueryDomainOperatingContextIdentityDeclaration {
        domain::WorthQueryDomainOperatingContextIdentityDeclaration::from_fields([
            ("tenant", self.tenant),
            ("pricing_regime", self.pricing_regime),
        ])
        .expect("static context field names are valid")
    }
}
```

An ergonomic extension should delegate to `declarations_in(...)`,
`contributions_in(...)`, or the operating-world binding surface. It must not
construct a declaration context, copy a generation, or recreate package
identity locally. The result therefore retains the same package, runtime,
generation, and world witnesses as the generic call.

## Small Example

Use marker types to make domain, operation, and family identity compiler
visible:

```rust
use worth_query::facade::{domain, read};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CatalogDomain;

#[derive(Clone, Copy, Debug)]
struct ReadRecord;

#[derive(Clone, Copy, Debug)]
struct ReadFamily;

impl domain::WorthQueryExecutableDomainOperation<CatalogDomain, ReadFamily>
    for ReadRecord
{
    type Input = ReadRecordInput;
    type Output = read::WorthQueryReadCompletion;
    type Publication = domain::WorthQueryPublishingOperation;
    type Execution = domain::WorthQueryDirectOperation;
}
```

The associated types close important paths:

- `WorthQueryPublishingOperation` permits the typed publication phase.
- `WorthQueryTerminalOperation` ends after execution.
- `WorthQueryDirectOperation` uses one operation executor.
- `WorthQueryWorkflowOperation` enters the installed workflow progression.

Do not use a semantic key string as a substitute for a marker type. Both are
useful, but they protect different boundaries.

## Declare Portable Meaning

Build one `WorthQueryDomainOperationSemanticClosure` and install it in the
domain package:

```rust
let operation = domain::WorthQueryDomainOperationDefinition::<
    CatalogDomain,
    ReadRecord,
    ReadFamily,
>::new(
    domain::WorthQueryDomainOperationIdentity::new("read-record", 1),
    semantics,
);

let package = domain::WorthQueryDomainPackage::declare(
    CatalogDomain,
    domain_identity,
)
.operation(operation);
```

Every semantic field must state its posture. Use the typed `NotRequired`
variant when a capability is absent. Empty provider registries, callbacks,
display names, and support defaults do not define operation meaning.

The semantic closure participates in canonical package identity. Declaration
order is canonicalized. A duplicate marker tuple with one-field semantic drift
fails package installation atomically.

## Register Volatile Mechanics

Register the executor separately during runtime construction:

```rust
let builder = runtime::WorthQueryRuntime::builder()
    .domain_package(package)?
    .domain_operation_executor(
        CatalogDomain,
        ReadRecord,
        ReadFamily,
        ReadRecordExecutor,
    );
```

An executor implements:

```rust
impl domain::WorthQueryDomainOperationExecutor<
    CatalogDomain,
    ReadRecord,
    ReadFamily,
> for ReadRecordExecutor {
    const LOWERING_FAMILY: &'static str = "read-record-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn installed_read_declaration(
        &self,
    ) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn execute(
        &self,
        input: ReadRecordInput,
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        let completion = context.execute_installed_read(workspace)?;
        Ok(domain::WorthQueryOperationExecutionMaterial::new(
            completion,
            input.result_state,
        ))
    }
}
```

The context is the executor’s only door to installed reads and bound graph
projections. The executor returns output material. Query mints the execution
identity, receipt, warnings, counters, publication, and phase proof.

## Real Example

Obtain the installed domain and bind through one operating world:

```rust
use worth_query::facade::{domain, read};

let installed_domain = workspace.domain(CatalogDomain)?;
let world = workspace.observe_operating_world()?;
let bound = world
    .family(ReadFamily)
    .bind(&installed_domain, ReadRecord)?;

let consumer = bound.consumer_projection_contract()?;

let executed = bound.execute(input, &mut workspace).unwrap();

let settled = executed
    .publish()
    .unwrap()
    .consume(consumer, read::project_facts().entity_identities())
    .unwrap()
    .settle()
    .unwrap();
```

This order matters. `execute` consumes the bound operation. Mint the consumer
contract first when the operation will publish.

The chain unwraps only to keep the successful progression readable. Match the
typed `TransitionOutcome` at every boundary in production code.

The final settlement retains the exact execution and publication receipts,
result state, warnings, counters, and consumption authority from the same
chain.

## Non-Publishing Operations

A non-publishing operation declares:

```rust
type Publication = domain::WorthQueryTerminalOperation;
```

Its execution result exposes terminal output and inspection, but it has no
`publish` method. This is a type-level distinction, not a runtime flag.

Use terminal operations for computations whose result is consumed directly by
the caller and is not a Query projection publication.

## Graph Participation

An operation declares graph reads and touches by role. The package binds a
role to an exact graph marker. Runtime construction registers the matching
definition and provider.

The binding phase verifies:

- exact graph marker and role
- graph contract compatibility
- required domain installation
- basis lane and basis family
- mutation and effect authority
- one atomic commit owner or declared compensation for multi-graph work
- complete conditional lowering set

Provider calls occur only after binding succeeds. Equal role strings do not
make different graph markers interchangeable.

One logical graph remains the default. Add another graph participant only for
a real independent authority or provider boundary.

## Consumer Support

`bound.consumer_projection_contract()` derives Query requirements from the
installed operation and the runtime support profile. It does not infer support
from available callbacks.

The contract covers basis, live work, continuation, async/result state,
typed-stop remediation, inspection, projection consumption, dependency impact, sharing,
invalidation, collection delivery, and conditional capabilities.

Downstream presentation and allocation requirements use
`WorthQueryConsumerBoundary`. They remain beside the Query contract and cannot
weaken or rewrite it.

Compatibility returns a pair-bound witness or a typed denial identifying the
failed dimension. Reports and digests are useful for explanation, but they
cannot authorize consumption.

## Workflows

Workflow operations declare a canonical DAG through
`WorthQueryPortableWorkflowDefinition`. Runtime construction registers the
exact stage executor and any parallel-admission provider.

Query owns:

- workflow run identity
- legal predecessor and frontier progression
- stage execution snapshots and receipts
- conditional stage decisions
- warnings, result state, and exact counters
- terminal trace and publication

Application code advances the returned `WorthQueryWorkflowRun`. It does not
keep a second stage ledger.

Completed workflow traces also support fresh ordinary re-execution and
trace-bound lineage inspection. Certification replay is deliberately absent
from the ordinary host surface; import it through
`worth_query_replay::facade`.

## How It Relates To Other Features

- [Conditional Installed Operations](./conditional-installed-operations.md)
  adds portable eligibility, trigger, comparison, and incremental evaluation
  to the same operation definition and binding path.
- [Installed Operation Re-Execution And Replay](./installed-operation-reexecution-and-replay.md)
  explains fresh execution identity and cert-only semantic comparison.
- [Installed Operation Lineage And Promotion](./installed-operation-lineage-and-promotion.md)
  explains effect-bound identity evolution, naming, and sparse promotion.
- [Bound Projection Lifecycle, Sharing, And Consumer Invalidation](./bound-projection-sharing-and-invalidation.md)
  explains declaration-indexed native access, relationship-specific
  compatibility, managed transitions, dependency impact, shared live owners,
  consumer leases, and exact invalidation deltas.
- [Projection Consumption](../capabilities/projection-consumption.md) owns the
  production fact extraction delegated to by the installed progression.
- [Aspects And Authority Lanes](../modeling/aspects-and-authority-lanes.md)
  explains the semantic truth read by graph and conditional contracts.
- Ordinary `facade::read` declarations remain appropriate for
  application-local reads that do not need installed operation identity.

## Inspection And Debugging

Useful surfaces include:

- `workspace.domain_installation_receipt(marker)`
- `workspace.verify_domain_execution_index_rebuild()`
- `bound.binding_identity()`
- `bound.commit_posture()`
- `bound.graph_roles()`
- `executed.receipt()`
- `executed.graph_receipts()`
- `executed.conditional_provenance()`
- `executed.counters()`
- the host receipt's published aftermath and external-effect posture when the
  installed application operation declares them
- `settled.publication_receipt()`
- `settled.result_state()`
- `settled.warnings()`

Use typed denial kinds and counters first. Messages explain a denial but are
not stable admission keys.

`workspace.domain_installation_receipt(marker)` reports installed package
identity, definition and derived-index counts, warnings, and construction
counters. Operational receipts retain their installation authority, so a
read, workflow, contribution, projection, live continuation, or inspection can
be traced to the package and runtime that authorized it. Rebind asks the target
runtime to admit the domain again; it never copies a generation or digest into
a new handle.

## Anti-Patterns

- Calling an executor directly.
- Rebuilding a canonical read inside each domain extension method.
- Putting callbacks or Signal slot numbers into a portable operation package.
- Treating an installation receipt or digest as executable authority.
- Constructing a family facade or graph-provider bag outside the operating
  world.
- Combining an execution from one bind with a consumer contract from another.
- Flattening `TransitionOutcome` into a local success/error flag.
- Contacting providers to discover whether an operation was declared.

## Current Limits

- Runtime installation occurs before runtime publication.
- Runtime-local application-aftermath recovery does not imply Store-backed
  handle reload or restart recovery.
- Handles and bound operations are not portable across runtimes or installation
  generations, even when packages are semantically equivalent.
- Collection windows and query-shaped patch delivery still require their
  dedicated later authorities. Native access, compatibility, dependency
  impact, sharing/leases, and invalidation are implemented, but declaration or
  support posture alone does not mint them; enter through the bound projection
  lifecycle described above.
- Installed-operation certification replay remains cert-only, and historical
  execution currently supports retained snapshots rather than delta replay or
  full reconstruction.
- Rich reports remain derived projections and never become admission authority.

## Related Docs

- [Conditional Installed Operations](./conditional-installed-operations.md)
- [Installed Operation Re-Execution And Replay](./installed-operation-reexecution-and-replay.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
- [Installed Operation Lineage And Promotion](./installed-operation-lineage-and-promotion.md)
- [Aspects And Authority Lanes](../modeling/aspects-and-authority-lanes.md)
- [Projection Consumption](../capabilities/projection-consumption.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
- [Downstream Runtime Integration](../foundations/downstream-runtime-integration.md)
- [Domain Capabilities](./README.md)
