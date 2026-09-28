# Worth UI 3.9 Phase 7 Boundary Brief

## Outcome

Phase 7 makes the ordinary lane execute the native meaning already admitted by
source lowering. Component shells, layout regions, child ranges, commands,
tokens, and mosaic state slots become directly addressable plan rows. A frame
borrows those rows from the active session; it does not recover declarations,
consult registries, or route command or state work through a component fallback.

## Real authority entering the slice

- The prepared application lowering authority owns the admitted candidate
  artifact and its bound component, surface, command, token, and mosaic facts.
- The durable-state reconciliation plan owns succession decisions. Plan
  lowering may carry an exact receipt, but may not choose or reinterpret its
  outcome.
- The regional plan store owns stable slots, slot generations, family indexes,
  and exact active-plan resolution.
- The active application session owns publication and lends execution. Tests
  may not submit a plan to the product executor.

## Architectural corrections required before ordinary execution is complete

1. Imports are source-composition facts, not executable child-range rows.
2. Production command rows must be lowered from bound surface command
   references. The test-only component hook is not command authority.
3. State-slot rows must be lowered from admitted mosaic state-slot descriptors.
   A node lifecycle transition is not a state-slot claim.
4. Child ranges must contain exact, bounds-checked child locators. Root-region
   counts may describe topology, but they are not ranges.
5. One canonical handle arena is retained. Family-specific handles are typed
   projections over it, not repeated vectors that can drift from arena truth.
6. Root-shell breadth is an indexed, named target set. Nested layout rows do
   not become roots merely because they share a family tag.

## Regional representation

Each changed top-level artifact node lowers to an owner bundle:

- one root row;
- zero or more layout-region or mounted-surface rows;
- one child-range row for each structural parent with children;
- bound command rows;
- admitted state-slot rows; and
- native token rows where the authored node is a token declaration.

The root row carries the exact member manifest for its bundle. Replacement can
therefore reconcile one affected owner in `O(A_owner)` work without scanning the
predecessor plan. Existing member identities are upserted so unchanged member
handles remain stable; removed members retire through the root manifest.

Child-range rows are finalized only after their target rows have stable slots.
They carry compact row-local locator slices in authored semantic sibling order;
storage canonicalization must never sort that meaning. Construction rejects
missing, duplicate, overlapping, foreign-owner, or out-of-bounds claims before
plan publication. Frame traversal follows locators and direct row indexes only.

## Durable-state succession

Mosaic slot kinds map to the already-governed durable families:

- splitter position -> splitter position;
- scroll position -> scroll anchor;
- focused region -> focus chain;
- selection token -> selection range;
- draft input state -> text-edit buffer;
- active stack or primary/auxiliary surface -> tab state; and
- visibility, collapsed, or pinned posture -> panel visibility.

Launch rows carry explicit launch succession. Replacement rows carry the exact
reconciliation receipt for their top-level owner and mapped family. Mismatched
owner, family, artifact basis, or foreign predecessor authority denies lowering.

## Execution and inspection

- Root shell traverses the indexed root set and names that breadth in its
  receipt.
- Component or layout targets traverse only their admitted subtree.
- Child-range, command, token, and state-slot targets resolve directly.
- Receipts distinguish direct target touches, intentional subtree touches, and
  root-shell breadth. A full-plan scan remains forbidden.
- A bounded ordinary summary reads family indexes and retains the exact shared
  immutable executable meaning. Its read-only projections expose admitted
  component, command, token, and state facts plus target breadth without
  reconstructing the artifact or flat plan; a digest alone is not native-
  meaning evidence.

## Proof plan

1. Compiled-once runtime tests prove canonical order equivalence, family-swap
   denial, range denial, foreign succession denial, stable unrelated handles,
   and scale-bounded owner replacement.
2. Repeated target execution proves zero parsing, string resolution, registry
   lookup, artifact scan, full-plan scan, and per-frame general allocation. A
   hostile executable-row test separately proves exact admitted command and
   native token values survive into the plan edge so zero-valued counters
   cannot bless a digest-only executor.
3. The application-contract certification binary writes real `.wui` files,
   lowers through the production filesystem provider, launches the public
   active session, discovers typed targets through bounded active-plan summary,
   and executes component, child-range, command, token, and state-slot rows
   through the production headless host.
4. The hostile QA loop rejects tests that mint plan rows, handles, receipts, or
   command/state meaning above the authority under test.

## Phase 8 handoff

Phase 7 does not add collection semantics. It leaves Query view references as
already-admitted plan facts so Phase 8 can add virtualized visible-range
execution without reopening ordinary lowering or identity authority.
