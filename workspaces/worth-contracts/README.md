# worth-contracts

This workspace owns platform-tier Road 1 contracts crates.

Allowed here:

- `worth-schema-*` crates

Not allowed here:

- `worth-entry-*`, `worth-derived-*`, `worth-pack-*`, or `worth-cert-*`
- root-level helper, scratch, misc, or temporary package lanes

Cross-workspace proof and tooling:

- boundary denial and orchestration proof belong in dedicated tool or
  certification surfaces, not in this workspace root

Package placement:

- Road 1 Rust packages in this workspace must be born under `crates/`
