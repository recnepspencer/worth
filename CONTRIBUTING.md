# Contributing to WORTH

Issues, design criticism, reproductions, documentation improvements, and code
contributions are welcome.

Before working on code, read [AGENTS.md](./AGENTS.md) and every document under
[`docs/coding-guidelines`](./docs/coding-guidelines). WORTH treats authority,
truth ownership, phase progression, physical placement, performance, and test
evidence as mechanically enforced architecture rather than local convention.

## Build and test speed

The repository is several Cargo workspaces over one dependency graph, so a
few conventions decide how much each build repeats.

- **Cold sessions** (fresh containers, CI): set `CARGO_INCREMENTAL=0`. The
  incremental cache is about half of a cold target directory and is never
  reused there, and sccache cannot cache incrementally built crates. Point
  sccache at a persistent backend and keep `CARGO_TARGET_DIR` identical across
  checkouts, because sccache keys on every `CARGO_*` variable. Building
  `worth-store-physical-backend` supports the observed transparent sccache 0.18.0
  release as well as direct rustc. Its existing pinned compiler, target, flags
  and dependency checks still apply; other wrappers and nested workspace
  wrappers remain unsupported. This assumes the installed compiler/cache and
  cache contents are trusted, not executable attestation.
- **Warm iteration**: keep the incremental default.
- **Automatic local cache**: put `rustc-wrapper = "<absolute path to sccache>"`
  in the `[build]` table of your user Cargo configuration. Keep the persistent
  cache outside Cargo's target directory; `cargo clean` should remove build
  output, never the compiler cache. Configure its size in sccache's own config.
  Do not disable this default with a blank `RUSTC_WRAPPER`; remove that override.
  Set `debug = false` under `[profile.dev.package."*"]` and
  `[profile.test.package."*"]` to omit dependency debug data while retaining
  workspace debug settings. On this Windows workstation, WORTH checkouts share
  `C:/worth-build-cache/target` and the separate 150 GiB persistent cache is
  `C:/worth-build-cache/sccache`. sccache 0.18's base-directory normalization
  does not normalize Rust's working directory or `CARGO_*` cache inputs; do not
  rely on that setting to make different Rust builds equivalent. Preserve
  matching compiler flags, features, dependency versions and environment.
- **Shared target directory**: every lockfile resolves each dependency line
  to the same version (`scripts/ci/check_lockfile_alignment.py`), so one
  `CARGO_TARGET_DIR` reuses third-party crates across workspaces. First-party
  crates still build once per workspace: Cargo keys a path package by its
  location relative to the workspace root.
- **Cross-workspace path dependencies** build at `opt-level = 0`. When a
  workspace gains a first-party crate owned elsewhere, add it to that
  workspace's `[profile.dev.package]` and `[profile.test.package]` lists, or
  `"*"` will optimize it.
- **Integration tests**: where a crate has a suite binary (`autotests =
  false`), add a scenario file to `tests/` and a `#[path]` module line to the
  suite instead of creating another test binary, each of which links the
  whole dependency graph.
  A file that constructs process-wide state, such as an execution authority
  or a global allocator, gets its own target under `tests/isolated/`.
- **Build scripts** declare `cargo::rerun-if-changed`; without it Cargo
  reruns them after any edit in the package and rebuilds every dependent.

## Pull requests

Keep a pull request centered on one coherent responsibility. Explain:

- the behavior or boundary being changed;
- the authority owner and destination topology;
- the focused tests or other evidence that protect the change; and
- any compatibility, recovery, or performance consequence.

Generated `AGENT_CONTEXT.md` files must be updated through repository tooling,
not edited manually.

## Contributor grant

WORTH uses a commercial/source-available licensing model. To preserve the
project's ability to issue commercial licenses and eventually convert released
versions to Apache-2.0, every contributor must agree to the
[Contributor License Agreement](./CONTRIBUTOR-LICENSE-AGREEMENT.md).

The pull-request template records that agreement. Do not submit a contribution
on behalf of an employer or another rights holder unless you are authorized to
make the grant.

## License

Contributions accepted into this repository are distributed under the same
[Business Source License 1.1](./LICENSE) terms as the repository and are also
covered by the contributor grant.
