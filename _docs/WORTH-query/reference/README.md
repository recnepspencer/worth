# WORTH Query Feature Reference Index

## Purpose

This index defines the feature-oriented documentation set for `worth-query`.

The rule is simple:

- milestone docs explain why and how a capability was built
- closeout docs explain what was certified
- feature docs explain how to use the capability correctly

If a developer or AI has to reverse-engineer usage from milestone prose,
certification tests, or internal harness code, the documentation set is
structurally incomplete.

## Why This Exists

`worth-query` now has enough surface area that "read the roadmap" is not an
acceptable onboarding or implementation strategy.

Feature references must be:

- feature-first rather than milestone-first
- explicit about stable, deferred, and unsupported boundaries
- concrete enough that an AI can build with the surface without inventing
  missing semantics
- organized around user intent, not around internal module names

## Adversarial Constraint

An AI or human implementing product features against `worth-query` must be able
to choose the correct public feature surface, understand its authority and
lifecycle boundaries, and combine it with related features without reverse-
engineering milestone docs, certification harnesses, or lower-runtime internals.

If using a feature correctly still requires archaeology through milestone
history, test fixtures, or runtime plumbing, the documentation set has failed.

## Governing Constraints

- `worth-relational` owns truth semantics
- `worth-signal` owns reactive evaluation, scheduling, and Signal component
  branch basis, lifecycle, and owner execution services
- `worth-query` owns typed query expression, lowering, live query semantics,
  result shaping, and the stabilized runtime facade
- documentation must preserve the same authority boundaries as the code
- every feature doc must say what the feature owns, what it composes, and what
  it does not own

## Documentation Architecture

This reference set should be authored as one doc per feature family.

Every feature doc must contain:

1. `Purpose`
2. `What It Owns`
3. `What It Does Not Own`
4. `Stable Public Entry Points`
5. `Core Mental Model`
6. `Lifecycle / Execution Shape`
7. `How It Relates To Other Features`
8. `Common Patterns`
9. `Anti-Patterns`
10. `Inspection / Debugging Surface`
11. `Current Limits`
12. `Worked Example`

If a feature is deferred, the doc must still exist, but it should be a
boundary/reference doc that names the future contract honestly rather than
pretending the feature is already usable.

## Feature Reference Matrix

| Priority | Status | Feature Family | Canonical Doc |
| --- | --- | --- | --- |
| P0 | required | Workspace overview and mental model | `reference/workspace-overview.md` |
| P0 | required | Live views | `reference/live-views.md` |
| P0 | required | Computed | `reference/computed.md` |
| P0 | required | Effects | `reference/effects.md` |
| P0 | required | Reads, observation, and materialization | `reference/reads-observe-materialize.md` |
| P0 | required | Writes and intent boundaries | `reference/writes-and-intents.md` |
| P0 | required | Branches and previews | `reference/branches-and-previews.md` |
| P0 | required | State and readiness surfaces | `reference/state.md` |
| P0 | required | Inspection | `reference/inspection.md` |
| P0 | required | Aspects and authority lanes | `reference/aspects-and-authority-lanes.md` |
| P0 | required | Public support matrix, admission, and compatibility posture | `reference/support-matrix-and-admission.md` |
| P1 | required | Projection consumption from read results, write receipts, and query-context execution | `reference/projection-consumption.md` |
| P1 | required | Typed query expressions and result shapes | `reference/query-expressions-and-result-shapes.md` |
| P1 | required | Schema validation and legality | `reference/schema-validation.md` |
| P1 | required | Historical basis, diff, and comparison queries | `reference/historical-diff-and-basis.md` |
| P1 | required | Lineage and correspondence queries | `reference/lineage-and-correspondence.md` |
| P1 | required | Scopes, templates, saved queries, and view shapes | `reference/scopes-templates-saved-queries-and-view-shapes.md` |
| P1 | required | Policy masking, tenant narrowing, and relationship-proof boundaries | `reference/policy-tenant-and-masking.md` |
| P1 | required | Subscription family overview | `reference/subscriptions-overview.md` |
| P1 | required | Subscription declaration families | `reference/subscription-declarations.md` |
| P1 | required | Subscription lifecycle, sharing, continuation, and preview isolation | `reference/subscription-lifecycle.md` |
| P1 | required | Automatic subscription family selection and diagnostics | `reference/subscription-selection-and-diagnostics.md` |
| P2 | required after 9.4 | Temporal basis and time-aware subscriptions | `reference/temporal-basis-and-time-aware-subscriptions.md` |
| P2 | required after 9.5 | Async/resource query families | `reference/async-resource-query-families.md` |
| P2 | required after 9.6 | Mixed truth/time/async delivery | `reference/mixed-cause-delivery.md` |
| P2 | required after 10 | Store-backed parity | `reference/store-backed-parity.md` |
| P2 | required after 11 | Durable query artifacts and reload semantics | `reference/durable-query-artifacts.md` |

## Coverage Rules

### P0

P0 docs are mandatory before we can claim the stabilized runtime facade is
actually usable by normal product work without spelunking tests.

These are the docs an AI or human should be able to read to build:

- a geometry kernel
- a workflow DSL
- a high-performance table/editor runtime
- a runtime-backed application surface with branches, derived state, and
  inspectable effects

### P1

P1 docs cover the broader query capability families that product code will
reach for once it moves beyond the core runtime facade.

These docs must make the feature boundaries understandable without forcing the
reader to reconstruct milestone history.

### P2

P2 docs are future-boundary docs tied to deferred milestones.

They should be added as soon as the owning milestone begins, and they must be
updated from "boundary/contract" to "usage/reference" only when the feature is
actually admitted.

## Non-Negotiable Rules

1. Every stable public feature gets a feature doc.
2. Subscriptions are not "covered by milestone docs." They require their own
   dedicated reference set.
3. Computed, effects, live views, branches/previews, and inspection each need
   a standalone doc even when examples overlap.
4. Docs must distinguish:
   - authoritative truth
   - branch-local truth
   - derived runtime state
   - effect delivery state
   - preview residue
   - external bridge state
   - deferred temporal/async state
5. Docs must say when something is vocabulary-only versus compatibility-stable.
6. Docs must prefer the stabilized public facade over lower-runtime plumbing.
7. If the code path is important enough to stabilize, it is important enough to
   document as a first-class feature.

## Suggested Authoring Order

1. `workspace-overview.md`
2. `live-views.md`
3. `computed.md`
4. `effects.md`
5. `subscriptions-overview.md`
6. `subscription-declarations.md`
7. `subscription-lifecycle.md`
8. `subscription-selection-and-diagnostics.md`
9. `branches-and-previews.md`
10. `inspection.md`
11. `aspects-and-authority-lanes.md`
12. `reads-observe-materialize.md`
13. `writes-and-intents.md`
14. `state.md`
15. `support-matrix-and-admission.md`

This order is intentional:

- it front-loads the docs most likely to unblock actual runtime/product work
- it gives AI systems a coherent runtime story before advanced capability docs
- it documents the surfaces where semantic misuse is most likely and most
  expensive

## Initial Assessment

Current documentation is strong on:

- vision
- roadmap sequencing
- milestone intent
- closeout boundaries
- certification requirements

Current documentation is weak on:

- feature-first usage references
- public runtime facade learnability
- subscription feature discoverability
- "what should I import and how should I think about it?" onboarding
- AI-oriented implementation guidance per feature family

That gap should be treated as real infrastructure debt, not just doc polish.
