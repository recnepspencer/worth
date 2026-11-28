# worth-relational-branch

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `worth/relational layer 2`
- Domain noun: `branch`
- Crate root: `crates/worth-relational-branch`
- Road 1 exemplar role: Owns the physical Relational branch layer.
- Deferred next homes:

- Public surface: Consumer code uses worth-relational::facade. The internal_port serves Relational owners.
- Declared facade exports (glob routes remain explicit): `none`
- Owned internal modules: `authority, branch, change_source, content, durability, execution, history, indexes, inline_type_sizes_tests, inspection, internal_port, mvcc, publication, query, runtime, schema, snapshots, storage, symbols, transactions, validation, visibility`
- Allowed in-tree dependency bands: `worth-relational-contracts, worth-relational-storage`

Boundary guidance (the compiler and retained checks own enforcement):
- Relational dependencies must follow the Relational owner map.
- Workspace dependency denials seal the package to admitted Relational callers.

Skeleton fence:
- The Relational DAG audit checks physical ownership and source placement.
