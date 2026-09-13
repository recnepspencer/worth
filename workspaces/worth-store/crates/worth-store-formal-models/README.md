# worth-store-formal-models

This crate owns the cold, non-authoritative checked semantics for Worth Store
physical protocols. Runtime owner crates remain the only issuers of executable
outcomes and receipts. Focused integration tests exercise owner behavior and
compare concrete observations with this crate's model mappings.

The authority direction is:

```text
runtime owner outcome
  -> read-only owner observation
  -> exhaustive abstraction mapping
  -> checked model action
  -> diagnostic checker verdict
```

The pinned TLC runner is described in `formal-toolchain.toml` and exercised by
`scripts/ci/verify_worth_store_formal_toolchain.ps1` or the matching `.sh`
command. Protocol model artifacts land inside their responsibility-shaped
`src/protocols/<family>/` directory. CI checks each current protocol model
directly. Adversarial behavior belongs in focused model and runtime regression
tests, not in a certification, coverage, or mutation-reporting system.
Counterexamples are transient diagnostics from the failed command; CI may keep
its ordinary command output, but the crate does not manufacture reports,
receipts, matrices, or evidence bundles.
