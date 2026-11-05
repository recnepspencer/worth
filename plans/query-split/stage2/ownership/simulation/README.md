# Helper-move simulation (phase-2 checkpoint, b78fc0c6b6)

Analysis only; no code moved. `scripts/query_split/ownership_simulation.py` (plain Python, no
cargo) runs against each parent's current greedy layering, at both levels:

1. **Helper judging.** Every function, constant or static at the root of a behaviour-coupled
   unit's blockers is judged beside the types it classifies, at the lowest destination among
   the units it holds.
2. **Fixpoint.** Every helper that can move does, then every unit referenced from below that
   can then move whole (carrying the types it needs, as the ownership analysis does). This
   repeats until nothing moves.
   - A moved item's references come from its new module.
   - The references it makes leave its old one.
3. **Remaining seams.** The script reports the seams that still hold the largest component,
   with the ownership sheet's default for each.

Macro-made `thread_local!` observation counters have no declaration to read. They carry no
dependency of their own and move with the code that uses them.

Two variants:
- **strict**: a helper moves only when everything it names already sits at or below the
  destination. This is the brief's definition.
- **helpers-carry-types**: a helper may take the types it names with it, but never another
  helper. This variant exists because the strict run shows that the heavy helpers are held by
  exactly the refusal type they build: `equal_cause` needs `PaidProviderDenial`, and `release`
  needs `LifecycleCountUnderflow`. A helper and the type it classifies are one unit in
  practice.

## Results

| level | variant | before (component / refs up) | after (component / refs up current / fresh layering) | helpers moved | units moved | helpers that could move at once |
|---|---|---|---|---|---|---|
| primary_graph | strict | 59 / 1,506 | **59** / 653 / 739 | 199 | 326 | 166 of 1,224 |
| primary_graph | helpers carry types | 59 / 1,506 | **57** / 395 / 520 | 440 | 628 | 276 of 1,224 |
| domain_computation | strict | 12 / 581 | **11** / 264 / 264 | 223 | 216 | 206 of 1,557 |
| domain_computation | helpers carry types | 12 / 581 | **9** / 205 / 200 | 577 | 618 | 351 of 1,557 |

Moving helpers and units cuts the references against the layering by 57 to 74 percent. It
barely moves the component.

What is left is spread thin. In the carry-types variant, the 63 primary-graph seams hold 395
references, and the largest seam holds 22.

## What holds the remaining component

The seams are broken down in each `simulation.md`, by pair, items and the impl that blocks
them. The heaviest primary-graph seams in the carry-types variant:

| seam | refs up | items | what holds it |
|---|---|---|---|
| `output_lineage` → `application_contribution` | 22 | 8 | incremental partitioned-computation retention |
| `product_operation` → `application_attempt` | 19 | 18 | workflow correction, effect program, the attempt host |
| `application_attempt` → `provider` | 18 | 8 | `WorthQueryPrimaryGraphProvider` |
| `product_operation` → `application_query` | 17 | 10 | the `WorthQuerySelectedProductOperation` query blocks (proposal 03) |
| `product_operation` → `application_contribution` | 16 | 3 | — |

At the domain level, two seams dominate:
- `authorization` → `primary_graph`: 120 references in 42 items. These are the primary graph's
  layout, principal, entity identity and workflow-actor observations.
- `convergence_epoch` → `managed_run`: 32 references in 22 items.

## Contract, nest or merge

Classified from the carry-types variant. Strict is in brackets.

| level | contract (3+ items) | one or two items (hand move or a tiny contract) | hub or host order (proposal 03, host supertraits) | nest or merge |
|---|---|---|---|---|
| primary_graph | 13 (27) | 42 (53) | 7 (9) | 1: `product_operation` ↔ `application_attempt` (0) |
| domain_computation | 4 (3) | 3 (6) | 0 (2) | 2: `authorization` ↔ `primary_graph`, `convergence_epoch` ↔ `managed_run` (2) |

The nest-or-merge rows are where the items crossing are many and wide: 18 to 42 items. They
read as one responsibility split across two modules:
- **Authorization's admission.** It reads the primary graph's principal and entity vocabulary
  at 42 items; that vocabulary sits in primary graph modules that authorization is placed below.
- **Product operation and attempt.** The workflow correction and commit pipeline is shared
  between them.

The rest are narrow seams. About two thirds of them hold one or two items, which a hand move of
the item (often a refusal kind or a small observation struct) settles without a trait.

## Caveat

The destination rule (the lowest module that references a unit) sends most of the heavy
refusal helpers to `application_program`. The greedy layering ranks it near the bottom, 21st;
the draft stack ranks it 50th. A simulation against the draft stack would place them
differently. The layering, not only the moves, is part of the decision.

## Wave-1 check: where the carried units would land (sink)

Before applying the helpers-carry-types moves, I checked where they send items.

| variant | primary_graph: largest destination | domain: largest destinations | after: component, PG / DC |
|---|---|---|---|
| greedy layering (applied as briefed) | `application_program` 709 of 1,073 | `domain_evidence_binding` 589, `artifact_owner` 410 of 1,205 | 57 / 9 |
| draft stack | `record_reservations` 706 of 1,081 | `execution_runtime` 947 of 1,068 | 56 / 10 |
| greedy, at most 6 items carried per move | `application_program` 644 of 899 | `domain_evidence_binding` 482, `artifact_owner` 335 | 57 / 9 |
| greedy, users may also be lifted to what they need (cap 6) | `application_program` 644 of 1,010 | same | 56 / 9 |
| as above, lifting first | `application_program` 649 of 1,029 | same | 55 / 9 |

The destination rule sends a unit to the lowest module that references it, and each move
takes the unit's types with it. Every variant therefore drains the refusal and evidence
closure into one low module that touches any member of it.

`application_program` makes only 6 references upward, among them `WorthQueryOutputDemandDenial`
in a public signature. That denial transitively holds most of the refusal vocabulary, so
round by round about 650 items collect there. Applying this would put the crate's refusal,
observation and evidence types into modules that mean something else, against the
domain-structure law. It is left for the owner to rule on (see the wave-1 report).
