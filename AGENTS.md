# AGENTS.md

## Review and verification policy

Before reviewing, read [REVIEW.md](REVIEW.md). Apply the repository skills
[qa-loop](skills/qa-loop/SKILL.md), [qa-tests](skills/qa-tests/SKILL.md) and
[code-quality-qa](skills/code-quality-qa/SKILL.md) to each final platform slice.
Work from the assigned phase or goal, break it into slices as useful, and verify
work after each slice. Continue until the assigned phase is complete.
The implementation-batch, plan-implementation, optimize-code and
feature-doc-writer skills are retired; older plans do not require invoking them.
CAD kernel work uses cad-qa-loop and cad-qa-tests instead of the generic pair.
Review briefs must name the files; a skill label alone is not application.

Compile time is sacred. Routine edits, reviews and slice landings use cheap
source checks and focused affected evidence. Expensive gates belong at
milestone completion, release verification or nightly jobs. Do not automatically
expand to all reverse dependencies or workspace CI. Reuse valid evidence and
report deferred checks; never describe a focused pass as a full gate pass.

## Consumers And Contributors

If you are consuming WORTH (building an application), read
`docs/build-an-application.md` and `docs/capability-index.md`. An application
imports only `worth-query-decl` and `worth-query-host`. Before concluding that a
capability is missing, follow the capability index's procedure; never build a
second way to do something the platform already does. The coding guidelines
below bind platform contributors.

**The kernel does not reestablish trust: see
[the governing rule](docs/coding-guidelines/arch_laws.md#the-kernel-does-not-reestablish-trust).**
Public WORTH owns currentness, invalidation, publication and recovery; a gap
there is fixed in public WORTH, never rebuilt in a consumer.

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
- `iteration_laws.md` — iteration speed is architecture: court crates,
  generated build roles, derived scope, self-checking configuration.

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
The rule is enforced by `tools/boundary-check/src/graph_rules.rs` and
`tools/boundary-check/src/text_rules.rs`.

Law 1, reach: `tools/boundary-check/src/seal_rules.rs` checks test-door names
matching `*_for_test` and `*ForTest`, without visibility or cfg exemptions.
References belong to their owner or the exact callers in `control-callers.txt`. Law 2, scope: a
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

Run cheap source checks, focused owner tests and affected integration or UI
evidence. Formatting and line-cap checks remain source checks. The review
policy above governs scheduling: broad expensive lanes are milestone, release
or nightly work. A concrete defect may require an expensive focused reproducer;
name the uncertainty and why cheaper evidence is insufficient.

## Generated Files

Per-crate `AGENT_CONTEXT.md` files are generated by enforcement tooling. Never
hand-edit them.

## Destructive-Action Safety

- Do not exfiltrate private data.
- Do not perform destructive operations outside the user-authorized scope.
- Resolve and verify exact targets before deletion or replacement.
- Prefer recoverable deletion when practical.
