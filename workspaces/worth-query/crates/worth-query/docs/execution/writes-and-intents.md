# Writes and Intent Boundaries

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

This is the authority boundary for changing truth or staging future change.
`workspace.insert(...)`, `workspace.update(...)`, `workspace.delete(...)`,
`workspace.delete_with(...)`, explicit `workspace.submissions()` calls, and
write-intent surfaces are the preferred public mutation paths.

Intent surfaces now include real covered families, but callers still need to
distinguish between:

- stable scalar mutation surfaces
- explicit submission lanes for command-shaped mutation
- covered intent families with shared admission and handoff behavior
- broader intent-shaped vocabulary that is still support-gated or deferred

On primary backends, `workspace.submissions()?.submit_batch(commands)` and
`workspace.write_batch_intent(commands).execute()` are one backend commit
boundary, not nice-looking wrappers around separate per-command commits. That
atomicity rule is what makes invariant-complete graph closures trustworthy
instead of only "correct if nobody looks in the middle."

The important posture is simple: ordinary runtime code should not need direct
workspace write helpers. If it works with `WorthQueryWriteCommand::*`, it
should enter through an explicit intent or submission lane.

On admitted families, graph-composition existing-target lanes are the
identity-preserving relation rewrite surface. The resulting receipt keeps the
existing-truth relation binding intact instead of treating the rewrite as a
delete-plus-recreate disguise.

## Stable Entry Points

Stable:

- `workspace.insert(...)`
- `workspace.update(...)`
- `workspace.compose_graph(...)`
- `workspace.delete(...)`
- `workspace.delete_with(...)`
- `workspace.submissions()?.submit(...)`
- `workspace.submissions()?.submit_batch(...)`
- `workspace.public_mutation_surface_report()`

Covered through the shared intent lattice:

- graph-composition verified-existing lanes
- `workspace.probe_existing_intent(...)`
- `runtime.write_intent(...)`
- `workspace.write_intent(...)`
- `runtime.write_batch_intent(...)`
- `workspace.write_batch_intent(...)`
- `runtime.next_effect_write_intent(...)`

Still support-gated or deferred beyond the covered families:

- broader intent-shaped vocabulary outside the named covered surfaces
- future temporal, async/resource, and durable restart intent families

Important boundary:

- direct workspace write and batch helper seams are sealed from consumers
- graph-shaped same-batch authoring belongs on `workspace.compose_graph(...)`,
  not on caller-owned command-batch choreography
- graph touch obligations must be selected through Query's touch descriptor,
  operating world descriptor, obligation index, dispatch envelope, executor
  verdict, and support row evidence rather than local validator tables
- backend-verified existing-truth lanes are public and typed through graph
  composition, but callers must read bridge-backed verification support rows
  before teaching them as ordinary bridge-backed production flows
- covered intent execution is real, but it is not the same thing as blanket
  stable facade-family intent support
- callers must treat support admission and backend capability as authoritative
- the mutation surface report is the source of truth for which mutation
  surfaces are preferred, lower-level, or support-gated

## Core Mental Model

Use scalar workspace mutation when product code already knows one mutation to
perform.

Use a submission lane when product code already has command-shaped mutation and
needs an explicit authority boundary.

Use an intent path when product code is naming strategy-shaped or runtime-gated
change that must pass through the shared admitted intent path.

The difference matters:

- `submissions()?.submit(...)` mutates authoritative truth through the explicit
  workspace submission lane
- `write_intent(...)` and `write_batch_intent(...)` are covered mutation
  families that execute through the shared intent lattice
- `next_effect_write_intent(...)` consumes one staged pending write-intent unit
  from an effect, if the runtime admits that path
- when the effect declaration carries write-adjacent trigger posture, the
  resulting pending write-intent and effect-triggered receipt preserve that
  trigger class and origin identity instead of flattening temporal or async
  follow-on work into an ordinary callback-shaped write

Do not blur those models together.

## How It Executes

Direct mutation path:

1. Declare the live/computed/effect surfaces that care about the truth.
2. Execute `workspace.insert(...)`, `workspace.update(...)`,
   `workspace.compose_graph(...)`, `workspace.delete(...)`, or an explicit
   `workspace.submissions()` / write-intent path.
3. Receive a canonical write receipt.
4. Live, computed, and effect consequences route from that write.

Direct write receipts now carry:

- mutation family
- structured target evidence with distinct declared and resolved target views
- existing-truth binding evidence when graph composition targets admitted
  authoritative preexisting truth
- existing-truth assertion evidence when graph composition declares or
  backend-verifies authoritative truth without mutating stored values
- verified assumption-set evidence on backend-verified existing-truth lanes,
  including assumption snapshot token, assumption snapshot digest, verified
  precondition digest, and verification read-set breadth
- generic graph-composition program, resolution, breadth, lifecycle, and
  lineage evidence when the write was authored as one graph batch

Installed application-operation obligations are separate. Their owner work is
proved by the managed session and application commit terminal, not by a direct
workspace write receipt.
