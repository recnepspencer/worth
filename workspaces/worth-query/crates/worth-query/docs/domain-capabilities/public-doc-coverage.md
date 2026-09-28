# Domain Capability Documentation Certification

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Documentation certification keeps the installed-domain public surface,
representative executable examples, focused authority boundaries, hostile
tests, and product guides pointed at the same current API.

## Why You Use It

- verify that installed-domain discovery still names the public facade
- reject removed setup roots and consumer-authored authority
- keep domain-capability and native-aspect examples synchronized with the code

## Stable Entry Points

- `worth_query_domain_capability_certification_surface()`
- `certify_milestone_nine_thirteen_installed_domain(...)`
- `certify_milestone_nine_thirteen_native_values(...)`

## Core Mental Model

The certification surface is an inventory. The milestone certification bundles
are executable evidence. Product docs remain the usage source of truth; the
certification lane proves that their public names and prohibited boundaries
still agree with compiled code.

## How It Executes

1. compile representative installed-domain public examples
2. compile-fail only the authority violations whose guarantee is type-level
3. run hostile runtime and consumer-residue checks
4. review product docs against current public types and value carriers
5. seal the resulting evidence rows into certification bundles

## Small Example

```rust
let surface = worth_query_domain_capability_certification_surface();
assert!(surface.category_count() > 0);
```

## Real Example

```rust
let installed = certify_milestone_nine_thirteen_installed_domain(repository_root)?;
let native = certify_milestone_nine_thirteen_native_values(repository_root)?;

assert!(!installed.certification_digest().is_empty());
assert!(!native.certification_digest().is_empty());
```

## How It Relates To Other Features

- [Runtime-Installed Domains](./runtime-installed-domains.md) is the canonical
  installed-domain guide.
- [Native Aspect Values](../capabilities/native-aspect-values.md) is the
  canonical value-authority guide.
- [Goldens, Boundaries, And Hostile Certification](./certification/goldens-boundaries-and-hostile-certification.md)
  explains the proof layers.

## Inspection And Debugging

Use the evidence rows to locate which compiled example, boundary, hostile case,
or documentation audit disagreed. Do not use certification artifacts as
runtime authority.

## Anti-Patterns

- treating file existence or link resolution as semantic documentation proof
- creating a doc-specific test when an existing public integration journey is
  already the stronger evidence
- documenting an internal transition helper as an ordinary consumer entry

## Current Limits

Certification proves synchronized public teaching and tested boundaries. It is
not a substitute for runtime support admission or performance evidence.

## Related Docs

- [Certification Surface And Closeout Bundle](./certification/certification-surface-and-closeout-bundle.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
