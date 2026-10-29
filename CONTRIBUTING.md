# Contributing to WORTH

Issues, design criticism, reproductions, documentation improvements, and code
contributions are welcome.

Before working on code, read [AGENTS.md](./AGENTS.md) and every document under
[`docs/coding-guidelines`](./docs/coding-guidelines). WORTH treats authority,
truth ownership, phase progression, physical placement, performance, and test
evidence as mechanically enforced architecture rather than local convention.

## Rust toolchain and canonical sort qualification

The repository toolchain and CI select stable Rust **1.99.1**, compiler commit
`58cf8f9c35b355199727627d07ae34a4e2846d67`. This is the current
[Phase 18 qualification candidate](plans/WORTH-query/milestone-9.18.1.md#phase-18-mandatory-current-rust-toolchain-qualification);
runtime and Proprietary acceptance remain pending. The minimum language version
remains Rust 1.94. The supported older canonical-sort identities remain exact
Rust 1.94.0 / `4a4ef493e3a1488c6e321570238084b38948f6db` and Rust 1.98.0 /
`88d9e12ae178fab0fb5cc050a94da85685d449ea`.

`worth-foundational/build.rs` admits only the listed release/full-commit pairs;
the canonical sort boundary also requires a 32-bit or 64-bit target. Record
`rustc --version --verbose` for qualification. A different compiler or target
returns typed `UnsupportedSortImplementation` with the actual release, full
commit and pointer width before admission callbacks. It produces no canonical
artifact and must not be relabeled as arithmetic overflow or request exhaustion.
The closed refusal courts continue to exercise this boundary after an upgrade.

When upgrading, first verify the latest official stable release and full commit
against the [published toolchain manifest](https://static.rust-lang.org/dist/channel-rust-1.99.1.toml)
and [release announcements](https://blog.rust-lang.org/releases/). Review that
exact revision's core stable/shared sort, alloc slice/Vec buffer allocation and
any changed compiler specialization semantics against the existing admitted
comparison, move and scratch bounds. The reviewed 1.99.1 algorithms and allocation
path preserve the 1.98.0 bounds; the two renamed specialization attributes still
select the same compiler `Marker` meaning.

Keep ordinary ordering and actual comparison counts unchanged: counts participate
in canonical material, so a changed algorithm requires the canonical compatibility
owner's version decision. Run the existing ordering, digest, admission and typed
refusal courts on emitted artifacts, then update the repository/CI pins together.
Qualify the same immutable SDK and development compiler through Proprietary's
correction probe, installed consumer courts and mandatory repository gates before
acceptance. Retain older exact entries only while their compatibility commitment
remains valid; a wildcard, version comparison or disabled guard is insufficient.

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
  whole dependency graph. Nextest isolates runtime process state per test.
  Keep an intentional target under `tests/isolated/` only where link/harness
  isolation is required, such as conflicting global allocator definitions.
- **Windows linking**: the target `linker` setting chooses Rust's bundled
  `rust-lld`. Keep compiler safety/platform flags in `build.rustflags`; a
  target rustflags table replaces that array rather than extending it.
  For Windows doctest qualification, append `-C link-arg=/STACK:67108864`
  to process-local `RUSTDOCFLAGS`, preserving existing flags. This gives
  rustdoc's merged main thread the same 64 MiB reserve as the repository's
  test-thread policy; `RUST_MIN_STACK` only configures spawned threads.
- **Build scripts** declare `cargo::rerun-if-changed`; without it Cargo
  reruns them after any edit in the package and rebuilds every dependent.

### Nextest lanes and build reuse

The root workspace requires cargo-nextest 0.9.140 or newer. Use
`cargo inner -p <package> -E 'test(name_fragment)'` for focused library-owner
tests. `--lib` avoids building integration targets. Use
`cargo integration -p <package> --test <suite> -E 'test(name_fragment)'` for
consumer journeys and compiler-fence suites. `cargo verify --workspace` selects
all root-workspace tests for CI. Other-team workspaces retain their own lanes.

The default profile reports slow tests at 10 seconds and fails them at 60
seconds; integration/CI report at 60 seconds and fail at ten minutes. Fail-fast
and retries are disabled. Treat a slow result as a performance finding. Inspect
the workload and production owner; neither moving the test to another tier nor
raising its timeout fixes a performance defect.

Build the selected test artifacts once, then rerun them without compiling:

```text
cargo nextest archive -p <package> --lib --locked --archive-file C:/worth-build-cache/archives/owner-tests.tar.zst
cargo nextest run --archive-file C:/worth-build-cache/archives/owner-tests.tar.zst --no-fail-fast -E 'test(name_fragment)'
```

Keep matching source and fixtures alongside the archive; it does not contain
source. Rebuild after source, compiler, features or profile changes. Nextest
does not run doctests: keep the affected owners' rustdoc compiler contracts in
the integration/CI checks with `cargo test --doc --no-fail-fast -p <package>`.
Cargo compiler jobs stay at the CPU default. Queue builds against the shared
target rather than launching competing rebuilds for individual failures.

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
