# Installed Domain Closeout Evidence

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Installed domain closeout evidence composes package installation, public
capability execution, boundary rejection, consumer adoption, and documentation
agreement into one machine-checkable result.

## Why You Use It

- prove that a domain begins inside Query through one installed package
- prove that handles remain runtime- and generation-affine
- prove that consumers cannot recreate the installation or execution authority
- verify that the public docs describe the same path as the compiled examples

## Stable Entry Points

- `certify_milestone_nine_thirteen_installed_domain(...)`
- `WorthQueryMilestoneNineThirteenInstalledDomainCertificationBundle`

## Core Mental Model

The closeout bundle is certification evidence, not a setup API. Ordinary code
still starts with `WorthQueryDomainPackage::declare`, installs through
`WorthQueryRuntimeBuilder::domain_package`, and obtains a handle through
`WorthQueryWorkspace::domain`.

## How It Executes

1. certify package declaration and atomic installation
2. certify installed read, mutation, workflow, live, contribution, projection,
   inspection, and rebind journeys
3. run compile-fail and hostile authority-boundary cases
4. audit downstream consumers for competing authority residue
5. audit product documentation against the installed public facade

## Small Example

```rust
let bundle = certify_milestone_nine_thirteen_installed_domain(repository_root)?;
assert!(!bundle.certification_digest().is_empty());
```

## Real Example

```rust
let bundle = certify_milestone_nine_thirteen_installed_domain(repository_root)?;
assert!(!bundle.certification_digest().is_empty());
assert!(!bundle.domain_capability_certification_digest().is_empty());
```

## How It Relates To Other Features

- [Runtime-Installed Domains](./runtime-installed-domains.md) teaches ordinary
  setup and use.
- [Domain Capability Documentation Certification](./public-doc-coverage.md)
  explains documentation agreement.
- [Certification Surface And Closeout Bundle](./certification/certification-surface-and-closeout-bundle.md)
  explains the contribution-level certification inventory.

## Inspection And Debugging

Inspect the failing evidence row and its digest source. The row should point to
a compiled journey, boundary, consumer audit, or documentation audit rather
than a hand-authored completion claim.

## Anti-Patterns

- calling certification helpers from product execution
- treating the bundle digest as installed authority
- declaring closeout from a link-only or source-text-only check

## Current Limits

The bundle certifies the runtime-backed installed-domain surface. Durable and
cross-process behavior is certified only when its support profile is admitted.

## Related Docs

- [Domain Capabilities](./README.md)
- [Runtime-Installed Domains](./runtime-installed-domains.md)
- [Goldens, Boundaries, And Hostile Certification](./certification/goldens-boundaries-and-hostile-certification.md)
