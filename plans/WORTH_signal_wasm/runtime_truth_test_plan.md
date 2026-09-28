# worth-signals-wasm Runtime-Truth Test Plan

> **Status:** Completed closeout note
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Certification parent:** [test-requirements.md](./test-requirements.md)

## Outcome

The wasm product certification tree now runs on runtime-truth harnesses instead
of fabricated certification fixtures.

This document remains only to record the boundary rule that now governs the
package:

- certification and product-surface tests must prove semantics against the
  real wasm package/runtime boundary
- capability-denial tests must use runtime-backed or typed-unavailability
  paths, not fabricated history or replay objects
- narrow local doubles are allowed only for pure package-local logic that does
  not claim runtime parity, lifecycle truth, publication truth, or history
  truth

## Closed Work

The runtime-truth migration is complete for the wasm product certification
lanes:

- resource inspection/history lanes
- resource delivery lanes
- resource download lanes
- resource lifecycle lanes
- resource request lanes
- resource transfer lanes
- resource reconciliation lanes
- resource authoring lanes
- resource closeout lanes

The old resource-runtime fabrication layer is gone.

## Permanent Rule

The key rule is no longer a migration goal. It is now an active repository
constraint:

- certification-grade wasm tests must not fabricate `history()`,
  `replay_for(...)`, `lineage_for(...)`, `current_branch(...)`, or synthetic
  signal handles to prove runtime/publication/history truth

If a test asserts anything about:

- replay
- lineage
- branch
- restore
- delivery basis truth
- publication identity
- lifecycle/history parity
- verification packages

then it belongs on a runtime-truth harness.

## Why This Stays

The migration exposed real boundary bugs that fabricated certification support
had been hiding, including:

- restore/line rematerialization parity gaps
- visible-value version drift caused by reference-based equality
- history/availability contract mismatches
- denial surfaces leaking raw runtime failures instead of product-owned errors

That is the lasting value of the work. The wasm package now certifies its
history, restore, delivery, and diagnostics claims against the real substrate.

## Ongoing Verification

Representative verification remains:

```powershell
node --test crates/worth-signal-wasm/package/product/resource.runtime.test.mjs
node --test crates/worth-signal-wasm/package/product/signals.runtime.test.mjs
```

The repository should continue to reject drift back into fake certification
support inside wasm product certification directories.
