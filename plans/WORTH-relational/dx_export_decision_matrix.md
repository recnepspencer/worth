# WORTH Relational DX Export Decision Matrix

## Purpose

This is the decision pass over the current public surface of
`worth-relational`.

This doc answers:

- what stays on the main path
- what stays public but moves behind a more deliberate boundary
- what should be condensed into a guided flow
- what is just leakage

This is not about making the crate smaller by hiding real architecture.

It is about making the facade more honest.

The repo standards are the reason for these decisions:

- the facade is the public contract
- declarative surfaces beat scattered coordination
- names must teach the right mental model
- configuration must mirror architecture
- real subsystem power should stay available
- support scaffolding should not become product API by accident

This pass is intentionally conservative.

If a surface is architecturally real, the default move is not deletion.

The default move is:

- keep it
- contain it
- guide it better

---

## Inputs

This document builds on:

- [`dx_export_inventory.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_export_inventory.md)
- [`dx_export_exhaustive_audit.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_export_exhaustive_audit.md)
- [`dx_method_decision_matrix.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_method_decision_matrix.md)
- [`dx_plan.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_plan.md)
- [`architectural_guidelines.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/architectural_guidelines.md)
- [`MENTALITY.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/MENTALITY.md)

This file is the module-level pass.

The method-level pass now lives in
[`dx_method_decision_matrix.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_method_decision_matrix.md).

That split is intentional.

The crate has enough public verbs on runtime-owned helper surfaces that a
module-only matrix is not enough anymore.

---

## Boundary Legend

- `Primary`
  - should define the main public memory shape
- `Contained`
  - should stay public, but behind a more deliberate path
- `Guided`
  - should stay public, but should be reached through a condensed workflow
- `Leak`
  - support or accidental exposure that should stop shaping the public boundary

These are exposure decisions, not difficulty rankings.

---

## Action Legend

- `Keep`
  - keep public and keep prominent
- `Condense`
  - keep the capability, but replace flat/raw usage with a clearer guided flow
- `Contain`
  - keep public, but move it out of the main facade story into a narrower lane
- `Remove`
  - remove from the public-facing boundary story; may later become internal or
    test/support only

---

## Root Public Entry

### `worth_relational::facade`

- Boundary: `Primary`
- Action: `Keep`

Reason:

- this matches the repo rule that the facade is the public contract
- we should strengthen this boundary, not bypass it

### `RelationalRuntimeApi`

- Boundary: `Guided`
- Action: `Keep`

Reason:

- this is a good quick-start door
- it helps create a declarative, low-friction setup path
- it should support the facade, not replace it

---

## Main Facade Modules

## `facade::runtime`

- Boundary: `Primary`
- Action: `Condense`

Reason:

- this is the center of the product
- it is currently carrying too many architectural jobs at once
- the fix is not to demote it
- the fix is to make it teach the runtime more clearly through a smaller number
  of obvious flows

What that implies:

- keep runtime setup and runtime access central
- condense setup around the builder/profile story
- contain runtime-contract and certification-shaped vocabulary that does not
  need to sit flat on the main path

## `facade::transactions`

- Boundary: `Primary`
- Action: `Condense`

Reason:

- truth mutation is a core product story
- this namespace already has the right power, but it is too flat
- the repo strongly prefers declarative effects over scattered ceremony

What that implies:

- keep intents, batching, commit results, rollback, and summaries highly visible
- move trace-heavy or specialist mutation vocabulary behind a clearer guided flow

## `facade::query`

- Boundary: `Primary`
- Action: `Condense`

Reason:

- query is a real first-class runtime capability
- planning internals should remain available
- but the main mental model should be query intent and query result, not packet
  plumbing

What that implies:

- keep query public and important
- contain low-level planning and worker-fragment vocabulary
- grow a clearer top-level query story

## `facade::identity`

- Boundary: `Primary`
- Action: `Keep`

Reason:

- this is already clean, compact, and foundational
- it teaches the runtime honestly
- there is no reason to hide or flatten it

## `facade::schema`

- Boundary: `Primary`
- Action: `Condense`

Reason:

- schema is structurally central to Relational
- the current module collapses authoring, integrity, transition, reconciliation,
  and compatibility into one giant door
- that is a boundary problem, not a capability problem

What that implies:

- keep schema first-class
- condense schema authoring into a clearer primary path
- contain transition/reconciliation/bridge-adjacent schema machinery

## `facade::payloads`

- Boundary: `Primary`
- Action: `Keep`

Reason:

- small, honest, useful
- no obvious boundary problem here

## `facade::config`

- Boundary: `Primary`
- Action: `Condense`

Reason:

- profile-first setup is right
- the repo rule says config should mirror architecture instead of becoming a bag
  of knobs
- this area should become more guided, not less powerful

What that implies:

- keep profiles prominent
- keep deep knobs available
- group and teach config by subsystem intent rather than as a flat override pile
- pull the config story back into one public lane

Deep audit note:

The underlying config architecture is actually strong.

The facade story is what is split.

Today:

- `facade::config` exposes profile/policy vocabulary
- [`RelationalRuntimeConfig`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/config/data/runtime_config.rs)
  is exported from `facade::runtime`
- section config structs are not exposed through the top-level `facade::config`
  module
- nested override section structs exist, but are not directly re-exported
- the builder covers many config axes, but not every one directly

So "condense config" does not mean reduce configurability.

It means:

- make configuration structurally legible
- expose the resolved config model and section model more coherently
- keep full policy power without making users assemble the picture from several
  different public lanes

## `facade::errors`

- Boundary: `Primary`
- Action: `Keep`

Reason:

- structured error topology is part of the architecture
- this surface is already small and honest

---

## Important But Contained Modules

## `facade::diagnostics`

- Boundary: `Contained`
- Action: `Condense`

Reason:

- diagnostics are a real strength of the runtime
- today this reads more like artifact vocabulary than a user-facing subsystem
- it should stay public, but through job-shaped entry points

What that implies:

- preserve artifact richness
- add clearer diagnostics entry flows
- stop treating raw artifact nouns as the primary product story

## `facade::inspection`

- Boundary: `Contained`
- Action: `Condense`

Reason:

- this is a real subsystem and belongs in the facade
- but it needs a clearer story around what questions it answers
- observation should feel phase-typed and deliberate, not like a pile of read
  helpers

## `facade::history`

- Boundary: `Contained`
- Action: `Condense`

Reason:

- history is core to the product, but not the first door
- current history exports blur ordinary historical access with merge-adjacent
  concerns

What that implies:

- preserve the full capability
- make the path from current truth to history to replay/merge more legible

## `facade::publication`

- Boundary: `Contained`
- Action: `Condense`

Reason:

- publication is real authority-derived product surface
- but patch streams, subscriber recovery, and bundle lifecycle should not all be
  forced into the center of the facade story

What that implies:

- keep public
- group around publication jobs, not flat vocabulary lists

## `facade::snapshots`

- Boundary: `Contained`
- Action: `Keep`

Reason:

- compact
- honest
- easy to understand
- good candidate for a clean contained surface with minimal cleanup

## `facade::indexes`

- Boundary: `Contained`
- Action: `Contain`

Reason:

- architecturally real subsystem
- should stay available
- should not compete with runtime, transactions, query, or schema for product
  center of gravity

## `facade::symbols`

- Boundary: `Contained`
- Action: `Contain`

Reason:

- useful support vocabulary
- not part of the main public memory shape

## `facade::storage`

- Boundary: `Contained`
- Action: `Contain`

Reason:

- tiny support-level vocabulary
- fine to keep, but should stay out of the main story

---

## Specialist Modules

## `facade::merge`

- Boundary: `Contained`
- Action: `Contain`

Reason:

- merge is a real power surface
- it should not be erased or hidden
- it absolutely should not dominate the first impression

What that implies:

- preserve the raw merge vocabulary
- contain it behind a clear specialist lane
- eventually add a more guided merge workflow on top

## `facade::replay`

- Boundary: `Contained`
- Action: `Contain`

Reason:

- replay is architecturally real and authority-derived
- it belongs in the facade
- it should be taught as an escalation path from history/publication, not as an
  isolated first-class entry door

## `facade::lineage`

- Boundary: `Contained`
- Action: `Contain`

Reason:

- lineage is real architecture, not optional flavor
- but it should be reached as a contained power surface, not ambient facade
  noise

## `facade::durability`

- Boundary: `Contained`
- Action: `Contain`

Reason:

- durability and recovery are real subsystems
- they should stay public
- they should not shape the first-use runtime story

## `facade::commit_strategies`

- Boundary: `Contained`
- Action: `Contain`

Reason:

- this is architecturally real and likely strategically important
- it is also one of the biggest examples of real power needing a better
  declaration story

What that implies:

- keep it public
- keep it specialist
- over time, prefer more declarative/guided entry over flat raw orchestration

---

## Clear Leak

## `facade::harness`

- Boundary: `Leak`
- Action: `Remove`

Reason:

- this is support and certification scaffolding
- it does not earn a central place in the public boundary
- it weakens the facade contract by exposing structure that exists for us, not
  for the product consumer

This is the cleanest removal candidate.

---

## Summary Table

| Surface | Boundary | Action | Why |
| --- | --- | --- | --- |
| `runtime` | `Primary` | `Condense` | central product surface, but too flat today |
| `transactions` | `Primary` | `Condense` | core truth mutation story, needs guided flow |
| `query` | `Primary` | `Condense` | core capability, planning internals need containment |
| `identity` | `Primary` | `Keep` | already clean and foundational |
| `schema` | `Primary` | `Condense` | too many architectural jobs in one door |
| `payloads` | `Primary` | `Keep` | small and honest |
| `config` | `Primary` | `Condense` | should mirror architecture through profiles and grouped knobs |
| `errors` | `Primary` | `Keep` | structured and clean |
| `diagnostics` | `Contained` | `Condense` | should be job-shaped, not artifact-first |
| `inspection` | `Contained` | `Condense` | needs clearer observation story |
| `history` | `Contained` | `Condense` | should separate historical access from deeper escalation |
| `publication` | `Contained` | `Condense` | should group around publication jobs |
| `snapshots` | `Contained` | `Keep` | already compact and understandable |
| `indexes` | `Contained` | `Contain` | real subsystem, not main product identity |
| `symbols` | `Contained` | `Contain` | support vocabulary |
| `storage` | `Contained` | `Contain` | support vocabulary |
| `merge` | `Contained` | `Contain` | real power surface, needs specialist lane |
| `replay` | `Contained` | `Contain` | real power surface, should be escalation path |
| `lineage` | `Contained` | `Contain` | real power surface, should be contained |
| `durability` | `Contained` | `Contain` | real subsystem, not first-use story |
| `commit_strategies` | `Contained` | `Contain` | real subsystem, needs guided declaration story |
| `harness` | `Leak` | `Remove` | support/certification leakage |

---

## Immediate Follow-Through

This matrix says the next docs should do three things:

1. define the canonical primary path
2. define the contained/specialist lanes
3. define where condensation should happen first

So the next deliverables should be:

1. `dx_canonical_surface_spec.md`
2. `dx_boundary_spec.md`
3. boundary cleanup and lane-ownership decisions recorded in
   `dx_method_decision_matrix.md` and `dx_boundary_cleanup_list.md`

And then the first code cleanup pass should target:

1. `harness`
2. `runtime`
3. `transactions`
4. `schema`

Those are the highest-leverage moves for making the facade honest before bridge
work starts hardening around it.
