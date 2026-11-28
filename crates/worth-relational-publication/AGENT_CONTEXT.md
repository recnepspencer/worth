# worth-relational-publication

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `worth/relational layer 5`
- Domain noun: `publication`
- Crate root: `crates/worth-relational-publication`
- Road 1 exemplar role: Owns the physical Relational publication layer.
- Deferred next homes:

- Public surface: Consumer code uses worth-relational::facade. The internal_port serves Relational owners.
- Declared facade exports (glob routes remain explicit): `none`
- Owned internal modules: `authority, branch, durability, history, internal_port, lineage, mvcc, publication, runtime, transactions, validation, visibility`
- Allowed in-tree dependency bands: `worth-relational-contracts, worth-relational-storage, worth-relational-branch, worth-relational-lineage, worth-relational-mutation, worth-relational-commit-admission, worth-relational-merge, worth-relational-indexes, worth-relational-durability`

Boundary guidance (the compiler and retained checks own enforcement):
- Relational dependencies must follow the Relational owner map.
- Workspace dependency denials seal the package to admitted Relational callers.

Skeleton fence:
- The Relational DAG audit checks physical ownership and source placement.
