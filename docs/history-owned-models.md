# Recording edits to a model's owned entities

Declare model ownership on `ApplicationHistoryDefinition` with `.owns(relation)`.
Enroll an operation scoped to a reachable entity type with
`.owned_operation::<Binding>(label)`. The existing `.operation::<Binding>(label)`
retains its exact model-type constraint. Applications continue to import only
`worth-query-decl` and `worth-query-host`.

The compiling generic example is the [Model/Member declaration](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/owned_model/schema.rs).
It declares a model-to-member ownership relation and a member-to-child relation.
An unrelated reference relation is deliberately excluded. The [recording journey](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/owned_model/recording.rs)
uses the ordinary owning edit, advancement, finish and discard methods.

Installation resolves the exact relation kinds and endpoints and verifies that
each enrolled operation's scope type is reachable through the declared graph.
At recording admission, Native proves actual instance membership from stored
full-generation endpoints at the selected current head. Supplying a model name,
an entity list or an arbitrary reachable path proves no ownership. A child of a
different model receives `UnownedOperationScope`, naming the attempted subject
and the document's model. The operation keeps its original authorization,
invariants, idempotency and completion owners.

The framework installs relation exclusivity automatically. Adding or reparenting
an ownership edge checks its affected descendants and their declared ancestors.
Several paths to the same model remain one owner. A proposed second model owner
refuses before publication, including through ordinary operations outside
document history. `denial.model_ownership_conflict()` reads the exact refusing
subject. It does not disclose unadmitted owner identities or grant READ authority.
An ownership move removes the previous relation and adds the new relation in one
ordinary candidate; its handler must read the relation before unlinking it.

Each edit retains the admitted operation scope through its private candidate and
checks the actual receipt against that scope before adopting a history step.
Document readers keep seeing the old selected model until guarded adoption.
Request stops, missing material and refused cleanup keep their original typed
outcomes and owning continuations.

Source-scope-owned publication membership now uses the Native current roster and
its independently checked route presence. Exclusivity also judges the complete
candidate after those facts are installed, so a publication's first candidate
cannot introduce a second document owner. Cold material and work refusals retain
their original owners and subjects. No publication-ownership ledger is rebuilt
from history. The combined public batch remains under qualification.

For graph navigation, select
`ApplicationHistoryModelDomain::OwnedModelGraph` and declare `.navigation::<N>()`.
The graph domain covers the union of current and selected owned entities, their
outgoing relations, and incoming declared ownership edges. An ordinary incoming
edge from an outside source stays with that source. If it prevents retirement,
`OwnedGraph(cause)` carries `cause.foreign_incoming_relation()`: its exact entity,
relation, source owner and borrowed installed deletion contracts.

The [navigation court](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/owned_model/navigation.rs)
uses a recorded NAV grant and a real child-scoped edit. This checkpoint prepares
the declared structural selection, then returns the explicit
`RequiresCascadeCustody` refusal before creating a candidate. Atomic graph
publication, source-scope publication restoration and the applied checkout
courts await the Native historical-publication selection handoff described in
the owned-model plan. The cascade batch is merged; graph navigation remains
unavailable for adoption.
`ScopeEntityRecord` navigation cannot include ownership declarations, and
entity-only context refresh still refuses them. Corrections sharing the tree
follow #56.

Existing domains with no ownership declarations retain their original wire
format and canonical identity. Owned definitions and the explicit graph domain
use archive version 4 carrying the exact relation names and endpoints. The graph
domain has its own canonical meaning even before any members are owned. Decoded declarations
carry meaning only; installation and each current request still establish their
own correspondence and authority.


Owned graph navigation must restore producer outputs and their recorded source
observations and witnesses with authored state. Every producer family is then
verified by the existing required-set currentness owner at one final endpoint.
Only failed verification can run the affected producer; navigation does not
regenerate healthy outputs. The Native current ownership roster restores and
undergoes currentness checks with those outputs. Native installations must
register current publication storage; reaching owned publications without it
refuses CurrentOwnershipStorageMissing and never falls back to history.
