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

This increment's declared-relation implementation is under qualification.
Source-scope-owned publication membership and exclusivity must join the existing
producer custody owner before the public batch is ready. The implementation must
not reconstruct a second publication-ownership ledger.

Owned-graph navigation is separately tracked by #53. Entity-only checkout and
entity-only context refresh refuse definitions with declared ownership edges;
they cannot silently restore or validate only the model record. #53 restores the
union of the current and selected owned closures atomically, preserving each
publication's custody. Corrections sharing that tree follow #56. Neither
restoration capability is claimed by the recording entrance alone.

Existing history definitions with no ownership declarations retain their original
wire format and canonical identity. Owned definitions use a distinct archive
version carrying the exact relation names and endpoints. Decoded declarations
carry meaning only; installation and each current request still establish their
own correspondence and authority.
