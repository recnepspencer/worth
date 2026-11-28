# worth-relational-contracts

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `worth/relational layer 0`
- Domain noun: `contracts`
- Crate root: `crates/worth-relational-contracts`
- Road 1 exemplar role: Owns the physical Relational contracts layer.
- Deferred next homes:

- Public surface: Consumer code uses worth-relational::facade. The internal_port serves Relational owners.
- Declared facade exports (glob routes remain explicit): `none`
- Owned internal modules: `aggregate_identity, aspect_wire, authority, branch, canonical_basis_ready_sequence, change_source, commit_strategies, config, content, derived_relation, diagnostics, durability, errors, execution, graph_member, history, identity, identity_authority, indexes, inline_type_sizes, internal_port, lineage, mvcc, performance, projected_member_domain, publication, query, replay, runtime, schema, snapshots, storage, structural_traversal, symbols, transactions, validation, visibility`
- Allowed in-tree dependency bands: `none`

Boundary guidance (the compiler and retained checks own enforcement):
- Relational dependencies must follow the Relational owner map.
- Workspace dependency denials seal the package to admitted Relational callers.

Skeleton fence:
- The Relational DAG audit checks physical ownership and source placement.
