# Installed Domain Inspection

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Installed domain inspection explains an installed-domain runtime product while
preserving the installed package and execution authority attached to it.

## Why You Use It

- correlate a read, mutation, workflow, live handle, or projection with the
  domain package and runtime generation that authorized it
- inspect a stopped installed-domain operation without promoting diagnostics
  into authority
- distinguish installation posture from execution posture

## Stable Entry Points

- `WorthQueryInstalledDomainReadCompletion::inspect()`
- `WorthQueryInstalledDomainInspectionDeclaration::using(...)`
- `WorthQueryInstalledDomainInspectionRequest::run(...)`
- `WorthQueryWorkspace::domain_installation_receipt(...)`
- `WorthQueryWorkspace::inspections()` for ordinary retained runtime products

## Core Mental Model

Installation receipts describe package installation. Installed-domain
inspection describes an operation or retained product. Neither one authorizes
new execution; the installed handle and its authority witness do.

## How It Executes

1. Start from an installed-domain read completion.
2. Declare the inspection through the installed-domain inspection surface.
3. Bind the inspection context and run it against the owning workspace.
4. Read the typed outcome, retained authority, and diagnostic facts.

## Small Example

```rust
let outcome = read_completion
    .inspect()
    .using(domain::inspection_basis(basis))
    .run(&workspace);
```

## Real Example

```rust
let installation = workspace.domain_installation_receipt(CatalogDomain)?;
let outcome = read_completion
    .inspect()
    .with_rich_inspection()
    .using(domain::inspection_basis(basis))
    .run(&workspace)?;

compare_package_identity(installation.package_identity(), outcome.receipt());
```

## How It Relates To Other Features

- [Runtime-Installed Domains](./runtime-installed-domains.md) owns installation
  and handle authority.
- [Inspection](../capabilities/inspection.md) owns ordinary retained-product
  inspection.
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md) owns descriptive next-step guidance after
  typed stops.

## Inspection And Debugging

Check package identity, installation generation, runtime affinity, execution
receipt, and the typed terminal posture together. A matching label or digest is
not sufficient evidence of authority.

## Anti-Patterns

- using an installation receipt as an execution handle
- reconstructing installed authority from package or diagnostic fields
- treating an inspection outcome as permission to retry or mutate

## Current Limits

- Rich inspection is an optional diagnostic projection.
- Cross-process inspection requires a support profile that admits durable
  installed-domain state.

## Related Docs

- [Domain Capabilities](./README.md)
- [Runtime-Installed Domains](./runtime-installed-domains.md)
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
