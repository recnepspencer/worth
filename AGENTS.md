# AGENTS.md

## Consumers And Contributors

If you are consuming WORTH (building an application), read
`docs/build-an-application.md` and `docs/capability-index.md`. An application
imports only `worth-query-decl` and `worth-query-host`. Before concluding that a
capability is missing, follow the capability index's procedure; never build a
second way to do something the platform already does. The coding guidelines
below bind platform contributors.

Platform contributors adding a capability first check the capability index and
its known gaps, so the platform never gains a second way to do the same thing.
Add each new consumer capability to the index in the same change.

## Engineering Constitution

This repository builds the WORTH platform under a strict,
mechanically-enforced constitution. These laws bind all code implementation
and review work.

Before editing code, read every document under `docs/coding-guidelines/` in
full.

### Governing Laws

- `MENTALITY.md` — foundation-first build order under adversarial pressure.
- `arch_laws.md` — typed phase progression, contractual facades, and
  compiler-visible authority boundaries.
- `composition_laws.md` — one named semantic responsibility per file; no
  catch-all bags such as `helpers.rs`, `common.rs`, `util.rs`, or `shared.rs`.
- `domain_structure_laws.md` — physical boundaries preserve meaning,
  authority, and truth source.
- `perf_laws.md` — hot-path honesty, carried proof, and separation of ordinary
  from reconstructive cost.
- `testing_laws.md` — fixture honesty, independent evidence, real boundaries,
  and compile-cost discipline.
- `dx_laws.md` — design the developer experience as a real code target.

When a law blocks the cheapest local move, the law is right and the move is
wrong. Take the honest path. If that is genuinely impossible, stop and surface
the conflict instead of inventing a wrapper, alias, compatibility surface, or
parallel authority lane.

## Hard Invariants

Every workspace and package root has one build except the named other-team
roots `workspaces/worth-store`, `workspaces/worth-ui`, and `crates/worth-proof`.
No Cargo features are declared beyond Signal's retained product profiles/default
(including WASM forwarding) and Signal-only `test-peak-allocation`. Empty feature
tables and feature arrays on governed dependencies are refused. Unit-test-only
hooks use `cfg(test)`; other controls and observers compile and use owned handles.
The rule is enforced by `tools/boundary-check/src/one_build.rs`.

Law 1, reach: exported and crate-visible test doors are discovered throughout
their owners by the `*_for_test` and `*ForTest` conventions and sealed to their
owner and closed test/certification callers in `control-callers.txt`. Law 2, scope: a
control's arm lives on its receiver. Pre-runtime placement and read-only
recorders may use thread-local guards that clear their state during unwinding.

The four Bridge source factories and Installation's `certification_query` are
compiled public doors with a named source boundary. Only the owners and the
closed caller list in
`tools/boundary-check/config/certification-construction-callers.txt` may reference
them. Consumer-facing declaration and host production facades do not re-export
them. A new caller requires an explicit boundary ruling.

- Pure meaning stays Query-agnostic: `worth-schema-*` crates must never import
  `worth-query`. Query is consumed only through audience facades
  (`worth-query-decl`, `worth-query-host`) from `entry`-band crates, and
  `worth-query-replay` only from `cert`-band crates.
- Replay and reconstruction are cert-only. Ordinary lanes must not import
  replay surfaces.
- Tier direction is one-way: `worth-*` must never depend on `worthy-*`.
- Authority is proof-carrying. Governed public surfaces demand concrete
  platform authority types from `worth-proof`, never generic
  `AuthorityMarker` bounds. Forged authority must open no doors.
- Place a new authority, phase, proof, binding, or lifecycle type before
  designing it. Three questions, first `yes` wins: does it decide whether an
  operation is **legal** — `worth-proof`; would it mean the same thing crossing
  a boundary in another runtime — `worth-foundational`; does it need a clock, a
  counter, a live table, or `Drop` to exist — the owning runtime, never a
  substrate crate. A substrate a crate cannot see from its own manifest will be
  hand-rolled instead; verify the dependency exists before concluding the
  vocabulary does not.
- A constitutional failure caused by or present in the complete scoped dirty
  set blocks completion. Do not route around enforcement. A pre-existing
  failure in an untouched file is reported as repository debt but does not
  expand a scoped task or block its completion.

## File And Module Discipline

- Code and test files must stay at 400 lines or fewer by default.
- This applies to production code, tests, support code, fixtures, and helper
  modules.
- Exceeding 400 lines requires an explicit exemption in the relevant
  specification, plan, or workspace rule.
- The CI-enforced Rust allowlist is
  `scripts/ci/workspace_rust_line_cap_allowlist.txt`.
- The guard is `scripts/ci/check_workspace_rust_line_caps.sh`.
- Scoped implementation and review work runs the guard with `dirty`; that
  scope includes staged, unstaged, renamed, and untracked Rust files. The
  repository-wide mode remains the CI and dedicated debt-remediation check.
- Specifications and roadmap documents are not subject to this cap unless a
  specification explicitly says otherwise.
- If a touched code or test file already exceeds the limit, splitting or
  restructuring it is part of the work unless an explicit exemption exists.
- Never infer authority to restructure untouched files merely because the
  repository-wide guard reports pre-existing debt.

## Search And Repository Navigation

Use search tools with the prudence required for a codebase with millions of
lines.

Scope searches to the narrowest relevant directory, file family, or semantic
owner. Treat truncated or overbroad search output as incomplete evidence.

## Verification Requirements

Before claiming a boundary-relevant change is complete, run:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
```

Also run the focused owner tests, affected integration or UI proofs, formatting,
line-cap checks, and broader lanes required by the governing specification and
the risk of the change.

## Generated Files

Per-crate `AGENT_CONTEXT.md` files are generated by enforcement tooling. Never
hand-edit them.

## Destructive-Action Safety

- Do not exfiltrate private data.
- Do not perform destructive operations outside the user-authorized scope.
- Resolve and verify exact targets before deletion or replacement.
- Prefer recoverable deletion when practical.
