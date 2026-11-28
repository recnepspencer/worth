# Proposed State and Invariant Execution

Installed operations validate their proposed post-state before publication.
Ordinary reads continue to observe the committed graph while Relational owns
the proposed state, its validation evidence, and its publication reservations.

## Authoring an invariant

Application authors use the
`worth_query_host::facade::application_invariants` audience. An invariant
factory receives `WorthQueryApplicationInvariantSchemaResolver`, resolves
typed fields and relations, and returns a `WorthQueryApplicationInvariantRule`.
The rule's `prepare_scope` reads the proposed view to establish the affected
closure. Its `evaluate` method receives proposed and committed views
constrained to that scope.

The views expose typed `field`, `relations_from`, and `relations_to` access.
Foreign bindings, undeclared reads, wrong entity kinds, unavailable records,
and exhausted work return `WorthQueryInvariantAccessDenial`. An absent
optional field is the only successful `None`.

## Application commit progression

1. The provider session captures and compares the operation's decision facts.
2. Query lowers the effect program against the fresh read-set, verifies
   read-before-write and dependency closure, and matches the effects to the
   exact registered attempt.
3. Relational validates the branch-qualified proposal. Query checks its
   custom invariant receipts, operation touch grants, and coupled writer
   obligations before retaining the validated candidate.
4. Query rechecks freshness and advances the candidate through a single-use
   prepared commit. The publication owner decides the resulting disposition.

Stale, refused, and abandoned attempts release candidate custody. A stop
requiring recovery retains its own typed custody. An invariant refusal can
expose the original Relational fields and decode declared domain refusal
payloads under their configured wire bounds.

See [Canonical Graph Obligation Progression](./canonical-graph-obligation-progression.md),
[Provider Sessions and Decision Read-Sets](./provider-sessions-and-decision-read-sets.md),
and [Application Aftermath, External Effects, and Recovery](../execution/application-aftermath-and-recovery.md).
