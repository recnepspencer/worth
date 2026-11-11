# Runtime-length mixed history gestures

Start an owning group with `document.gesture(program, request, gesture_key)`.
Each `group.push::<Intent, Action>(request, intent, member_key)` captures one
ordinary operation description. A runtime loop can select different installed
actions and scopes for successive members. Every scope still needs the
history's recorded membership proof. Applications use only Decl and Host.

The first `advance` freezes the ordered list, including when construction
refuses. Empty gestures refuse `EmptyGroup`; later pushes refuse
`MembershipFrozen`. Installed member and retained-result limits apply before
input capture. Storage and each original typed owner are charged under the
request. A refused push retains the described prefix.

Use the existing `parts`, `advance`, `complete`, `finish` and `abort` lifecycle.
All members run on one private candidate. The document keeps its old selected
endpoint until fresh guarded adoption records one step. A later denial can
abort that candidate through the actual source and result owners. An original
gesture key reuses the ordered membership meaning; changed order, inputs or
member keys refuse drift. Redo choices retain their existing policy.

Heterogeneous completion entries retain their original typed result.
`entry.original::<Intent, Action>()` borrows it; `into_original` consumes the
entry and returns it, or returns the unchanged entry for a different type.
No receipt or output-settlement evidence is reconstructed. The result collection
and its owned iterator keep their resource backing alive.

The [compiling generic gesture court](../workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/owned_model/gesture.rs)
shows runtime lengths, mixed model/member edits, staged atomicity and typed
results. `group_edits` remains the homogeneous convenience entrance with flat
typed storage. Explicit homogeneous root annotations now name the program and
`WorthQueryHistoryTypedMembers<Schema, Intent, Program, Action>` storage type.
Both storage forms use the same group execution and Native reservation owner.

Owned-graph undo is #53's separate restoration contract. This recording increment
does not relax entity-only navigation eligibility or producer custody.
