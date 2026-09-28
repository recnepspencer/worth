# Certification Surface And Closeout Bundle

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Feature Is

The domain-capability certification surface is the machine-checkable readout of
what the public domain-capability seam claims to support, while the closeout
bundle is the executable proof artifact that summarizes representative outputs,
boundary digests, and width/slope evidence.

Installed-domain closeout composes this contribution-level evidence with
package, execution, boundary, consumer, and documentation evidence. See
[Installed Domain Closeout Evidence](../platform-entry-closeout.md).

## Why You Use It

- you want one place to inspect the public domain-capability lanes
- you need the certification bundle outputs for closeout or tooling
- you want to compare the declared public lanes with executable evidence

## Stable Entry Points

- `worth_query_domain_capability_certification_surface()`
- `worth_query_domain_capability_public_surface_inventory()`
- `certify_domain_capabilities()`
- `worth_query_domain_capability_representative_report()`
- `worth_query_domain_capability_slope_report(...)`

## Core Mental Model

The certification surface is descriptive inventory.

The certification bundle is executable evidence.

You use the surface to answer "what are the ordinary, inspectable, proof, and
raw lanes?"

You use the bundle to answer "what digests and counters prove those lanes are
still synchronized?"

## How It Executes

1. build or query the public surface inventory
2. build the representative report
3. derive the slope report
4. assemble the certification bundle outputs

## Small Example

```rust
let surface = worth_query_domain_capability_certification_surface();
let digest = surface.public_surface_digest();
let rows = surface.category_count();
```

## Real Example

```rust
let bundle = certify_domain_capabilities();

let public_boundary = bundle.output_digest("public_boundary_digest");
let support_digest = bundle.output_digest("support_artifact_digest");
let slope_digest = bundle.output_digest("contribution_materialization_slope_digest");
```

For geometry-kernel-grade use, this is where you inspect whether public lane
teaching and representative runtime evidence have drifted apart. Compiler
boundaries and runtime tests remain direct evidence; the bundle does not prove
those tests.

## How It Relates To Other Features

- [Goldens, Boundaries, And Hostile Certification](./goldens-boundaries-and-hostile-certification.md)
  explains the proof surfaces the certification bundle summarizes
- [Installed Domain Closeout Evidence](../platform-entry-closeout.md) composes
  package installation, public execution, compile-fail, hostile, consumer, and
  documentation evidence
- category docs in this tree explain the individual public lanes the surface
  inventories

## Inspection And Debugging

- compare inventory rows when a doc or example seems to teach the wrong lane
- compare bundle outputs when a category artifact appears to drift from
  certification expectations
- use the representative and slope reports when you need to inspect the bundle
  inputs directly

## Anti-Patterns

- treating the certification surface as executable proof by itself
- assuming a green inventory means the ordinary lane examples are still honest
- reading width and slope outputs as benchmarking data instead of closeout
  evidence

## Current Limits

- the certification surface is closeout infrastructure, not an ordinary product
  feature
- width and slope outputs prove live synchronized evidence, not end-to-end
  performance benchmarking

## Related Docs

- [Goldens, Boundaries, And Hostile Certification](./goldens-boundaries-and-hostile-certification.md)
- [Installed Domain Closeout Evidence](../platform-entry-closeout.md)
- [Support Matrix And Admission](../../foundations/support-matrix-and-admission.md)
