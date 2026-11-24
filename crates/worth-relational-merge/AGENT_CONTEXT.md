# worth-relational-merge

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `worth/relational layer 3`
- Domain noun: `merge`
- Crate root: `crates/worth-relational-merge`
- Road 1 exemplar role: Owns the physical Relational merge layer.
- Deferred next homes:

- Public surface: Consumer code uses worth-relational::facade. The internal_port serves Relational owners.
- Declared facade exports (glob routes remain explicit): `none`
- Owned internal modules: `authority, commit_strategies, inspection, internal_port, merge`
- Allowed in-tree dependency bands: `worth-relational-contracts, worth-relational-storage, worth-relational-branch`

Boundary guidance (the compiler and retained checks own enforcement):
- Relational dependencies must follow the Relational owner map.
- Workspace dependency denials seal the package to admitted Relational callers.

Skeleton fence:
- The Relational DAG audit checks physical ownership and source placement.
