# Goldens, Boundaries, And Hostile Certification

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Feature Is

This doc explains the evidence surfaces that keep the domain-capability seam
honest:

- ordinary compile-checked examples and focused compile-time boundaries
- hostile executable certification tests

## Why You Use It

- you want to know which proof surface should catch a regression
- you need to extend the seam without weakening its public-lane guarantees
- you want confidence that domain-package code cannot bypass Query through
  pseudo-Query shortcuts

## Stable Entry Points

Executable closeout:

- `certify_domain_capabilities()`
- `domain_capabilities::certification_closeout_tests`
- `certify_milestone_nine_thirteen_installed_domain(...)`
- `certify_milestone_nine_thirteen_native_values(...)`

## Core Mental Model

Ordinary examples prove the public path compiles the way the docs teach it.

Compiler-visible boundaries make illegal progression, degradation, or
construction unrepresentable. Prove the intended public journey through the
owning package's ordinary tests and doctests; do not mirror runtime assertions
in a separate compiler-fixture matrix.

Hostile certification proves the public, checked, proof, and raw lanes converge
or stay distinct in the ways the spec requires.

## How It Executes

1. focused public examples compile the intended ordinary journeys
2. boundary enforcement rejects illegal package and authority dependencies
3. hostile tests compare equivalent and intentionally different surfaces
4. certification reports summarize runtime evidence without trying to certify
   the test harness itself

## Small Example

```rust
let bundle = certify_domain_capabilities();
assert!(!bundle.outputs().is_empty());
```

## Real Example

When you add a new domain-capability feature, the honest path is:

1. add or update the feature doc
2. add or update the smallest compiling public example that exercises it
3. add a compile-fail boundary only when type-system rejection is the product
   guarantee
4. extend hostile certification if the feature changes canonicalization,
   support posture, or lane behavior

## How It Relates To Other Features

- [Certification Surface And Closeout Bundle](./certification-surface-and-closeout-bundle.md)
  explains the public readout that summarizes these proofs
- [Domain Capability Documentation Certification](../public-doc-coverage.md)
  binds installed-domain and native-aspect docs to compiled evidence
- [Installed Domain Closeout Evidence](../platform-entry-closeout.md) composes
  package, execution, boundary, consumer, and documentation proof
- category docs should point to the strongest existing executable evidence;
  they do not each require a dedicated golden or boundary test

## Inspection And Debugging

- if the docs teach a path that no longer compiles, the owning public example
  should fail
- if a forbidden shortcut becomes legal, the compile-fail boundary should fail
- if two lanes drift semantically, hostile certification should fail

## Anti-Patterns

- multiplying compile fixtures when an ordinary integration test proves the
  same behavior faster and more clearly
- relying on runtime denials when the boundary should be compile-fail
- writing tests whose only purpose is to prove that another test ran

## Current Limits

- these proof surfaces are only as good as their synchronization with the live
  public API
- they do not replace product-facing docs; they prove the docs are still honest

## Related Docs

- [Certification Surface And Closeout Bundle](./certification-surface-and-closeout-bundle.md)
- [Domain Capability Documentation Certification](../public-doc-coverage.md)
- [Installed Domain Closeout Evidence](../platform-entry-closeout.md)
- [Domain Capabilities](../README.md)
