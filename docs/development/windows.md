# Windows development setup

This guide accompanies the [iteration migration plan](../../plans/cross-runtime/iteration-speed.md).
It covers native Windows x64 with MSVC, PowerShell, and sccache. Mac and Linux
setup are not qualified here.

**Migration status:** prerequisites and version inspection below are usable now.
The repository-managed configuration and `loop` commands are the planned G1-G5
destination; they are not installed by adding this document. Until those phases
land, a checkout without `loop.ps1` cannot follow the loop examples. Do not treat
this guide as evidence that caching is already active.

## 1. Install native prerequisites

Use a short, ordinary local checkout path, for example `C:/src/worth`. A short
path avoids problems with Relational's existing deep source trees; it does not
change the requirement to fix invalid or excessive topology during migration.
Keep the checkout outside a synchronized cloud folder.

Install Git for Windows, PowerShell 7, Python 3.12, rustup, and Microsoft's C++
Build Tools. In Visual Studio Installer, select **Desktop development with C++**,
the x64/x86 MSVC toolset, and a supported Windows SDK. The full IDE is optional.
Use a **Developer PowerShell for VS** session if the native SDK/toolset is not
discoverable from an ordinary shell. `rust-lld` still needs the appropriate
Windows import libraries and SDK; choosing LLVM's linker does not supply them.
See the official [Rust MSVC prerequisites](https://rust-lang.github.io/rustup/installation/windows-msvc.html)
and [Microsoft command-line setup](https://learn.microsoft.com/en-us/cpp/build/building-on-the-command-line?view=msvc-170).

Inspect installation from PowerShell:

```powershell
git --version
$PSVersionTable.PSVersion
python --version
Get-Command rustup, rustc, cargo, python
```

The guide assumes `python` resolves to Python 3.12. If Windows opens an App
Installer prompt instead, fix the Python installation/PATH or app execution
alias before setup. Do not hide an interpreter mismatch inside the build script.

## 2. Install the pinned Rust and helper tools

Use the exact qualified stable pin adopted by
[Query 9.18.1 Phase 18](../../plans/WORTH-query/milestone-9.18.1.md#phase-18-mandatory-current-rust-toolchain-qualification).
It requires Rust 1.99.x qualification and adoption of a newer official stable
release if one is available when implemented. The observed 1.94.0 installation
is legacy inventory, not the selected development destination.

After G1 supplies the repository pin, read it and install its components without
changing the machine's default toolchain:

```powershell
$taskRustPin = python -c 'import pathlib, tomllib; print(tomllib.loads(pathlib.Path("rust-toolchain.toml").read_text())["toolchain"]["channel"])'
rustup toolchain install $taskRustPin --profile minimal --component rustfmt --component clippy
rustup run $taskRustPin rustc --version --verbose
rustup run $taskRustPin cargo --version
```

Once G1 adds `rust-toolchain.toml`, ordinary commands from the checkout select
that pin. An `edition` or `rust-version` manifest entry alone does not pin the
active compiler. Avoid a `rustup override` or `RUSTUP_TOOLCHAIN` setting that
silently defeats the repository file.
If the pin has not landed, complete or consume Phase 18's qualification first;
do not substitute `stable`, an arbitrary 1.99 patch, or a disabled sort guard.
Cache and LLVM baselines must be established on the adopted qualified compiler.

Use sccache `0.18.0` and cargo-nextest `0.9.140`. Check before reinstalling:

```powershell
sccache --version
cargo nextest --version
```

Both versions were observed installed on the inspected Windows host on
2026-10-28. That inspection did not compile a cache probe or qualify a native
build. Installation and activation are separate requirements.

If sccache is missing, obtain the Windows binary from its
[official pinned release](https://github.com/mozilla/sccache/releases/tag/v0.18.0)
and place it on PATH. Follow the release's integrity instructions. A working
prebuilt binary avoids the bootstrap problem where Cargo is configured to use
a wrapper that has not been installed yet.

Alternatively, from a neutral directory outside the checkout and any ancestor
with a missing wrapper configured, install pinned tools from source:

```powershell
cargo "+$taskRustPin" install --locked --version 0.18.0 sccache
cargo "+$taskRustPin" install --locked --version 0.9.140 cargo-nextest
cargo "+$taskRustPin" install --locked --version 0.4.46 cargo-llvm-lines
```

These are installation commands, not ordinary iteration commands. They can be
expensive. Do not repeatedly install tools at session startup. The
[`cargo-llvm-lines` 0.4.46 manifest](https://github.com/dtolnay/cargo-llvm-lines/blob/0.4.46/Cargo.toml)
declares Rust 1.86 as its minimum. Its actual use on this Windows toolchain is
part of G1 qualification, not a completed check in this guide.

Open a new shell after PATH changes, then check which binaries resolve:

```powershell
Get-Command sccache, cargo-nextest, rustc, cargo | Select-Object Name, Source
cargo llvm-lines --version
```

## 3. Understand repository-owned configuration

G1-G2 will own the effective settings in checked-in files. The intended Cargo
keys below must be merged into the existing tables by that implementation,
preserving safety/windowing flags and the Windows linker. They are not a
replacement for the complete `.cargo/config.toml`:

```toml
[build]
rustc-wrapper = "sccache"
target-dir = "target"
incremental = false

[env]
SCCACHE_BASEDIRS = { value = ".", relative = true }
```

Cargo expands that config-relative environment value to an absolute checkout
root. The setting is path normalization configuration, not a guarantee of Rust
cross-worktree hits. The pinned Rust cache implementation also keys the compile
directory and Cargo environment. G2 must qualify same-checkout library reuse
and shared immutable-dependency reuse separately.

Use the shared persistent local sccache store outside disposable Cargo targets.
The configured cache location and capacity are storage policy; setup must print
them. Do not put the persistent store inside `target`, run `cargo clean` to
"fix the cache", or routinely restart a daemon used by other lanes.

Each worktree/lane owns its writable target. Independent workspace/tool targets
are named by setup, and no two active writers share one. Existing ancestor or
user Cargo configuration can otherwise redirect several worktrees into the
same target. Repository settings and preflight must resolve that explicitly.

Generated role profiles align ordinary dev/test settings. sccache-compatible
compilation has incremental disabled in files. Do not export `RUSTFLAGS`,
`CARGO_PROFILE_*`, `CARGO_INCREMENTAL`, or `RUSTC_WRAPPER` to make a command work.
Raw Cargo and the loop must see the same declared settings. Do not force an
arbitrary linker, remove overflow checks, or disable debug data as a local fix.

Reference: [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html),
[pinned sccache configuration](https://github.com/mozilla/sccache/blob/v0.18.0/docs/Configuration.md),
and [Rust cache limitations](https://github.com/mozilla/sccache/blob/v0.18.0/docs/Rust.md).

## 4. Set up and diagnose the loop after G1-G5 land

From the repository root:

```powershell
./loop.ps1 doctor
./loop.ps1 setup
./loop.ps1 doctor
```

If local PowerShell scripts are blocked by the machine's execution policy, use
the same dispatcher through Python after it lands:

```powershell
python scripts/iteration/loop.py doctor
python scripts/iteration/loop.py setup
```

Do not globally weaken execution policy to run the loop.

The initial doctor may refuse unprepared tools and name `setup` as the fix.
Setup prepares the existing checkers, checks generated configuration, identifies
the owned targets, and qualifies actual sccache use. It must not silently change
the machine's toolchain, clear another lane's artifacts, or install OS components.

Doctor reports the compiler/target, wrapper path/version, effective config sources,
profile state, cache location/base directories, target ownership, and package/test
registration. Configuration failure stops before expensive compilation and names
the setting and source that need correction.

To inspect names of possibly conflicting ambient settings without dumping values:

```powershell
Get-ChildItem Env: |
    Where-Object Name -Match '^(RUSTFLAGS|RUSTDOCFLAGS|RUSTC|RUSTUP_TOOLCHAIN|CARGO_ENCODED_RUSTFLAGS|CARGO_PROFILE_|CARGO_INCREMENTAL|CARGO_BUILD_|SCCACHE_)' |
    Select-Object -ExpandProperty Name
```

Inspect the named setting and relevant Cargo files locally. Cargo reads ancestor
configuration and `%USERPROFILE%/.cargo/config.toml` in addition to the repo file.
Do not delete unrelated global settings or bypass a refusal with another override.
Do not share a complete environment dump; it can contain credentials.

## 5. Verify caching rather than installation

`loop setup` must compile a registered small library with the ordinary cached
configuration, then compile unchanged inputs into a separate owned verification
target and observe retrieval. It also verifies immutable dependencies can be
reused from another worktree without sharing writable Cargo artifacts. Keep
other builds quiet during this attribution check or report ambiguous statistics.

After setup, inspect the server used by that configuration:

```powershell
sccache --show-stats
```

The setup report distinguishes a stored cache miss, an actual subsequent hit,
a compiler error, and a non-cacheable invocation. Cargo saying `Fresh` means it
did not call rustc; it does not demonstrate a cache hit. Metadata-only checking
and final linking are not promised hits. An installed wrapper with no demonstrated
eligible compilation does not complete setup.

If another checkout's server/base-directory configuration is active, doctor must
explain the mismatch. Coordinate any required daemon transition when its users
are idle. Do not use global `--zero-stats` or `--stop-server` during another lane's
work. First-party relocation misses must be reported honestly; do not strip
identity-bearing inputs or add an unsafe cache-key override to force hits.

## 6. Ordinary iteration after setup

These are destination commands; they become usable when the dispatcher lands:

```powershell
./loop.ps1 where crates/worth-relational/src/branch/owner_services/basis_port.rs
./loop.ps1 edit
./loop.ps1 court
./loop.ps1 court --area relational::branch
./loop.ps1 gate
```

Use `where` on a current path after subsystem extraction. It names the production
owner and covering courts, so no one needs to remember their manifests.
`edit` compiles changed owners; `court` runs registered affected evidence;
`gate` runs the complete handoff portfolio once and prints narrow rerun commands.
An area-limited rerun does not replace the gate. CI stores observations against
the source revision; generated run journals do not belong in source control.

## Common refusals

| Symptom | Action |
|---|---|
| Wrapper missing/wrong version | Install the pinned binary, fix PATH, open a fresh shell, rerun doctor |
| Wrong Rust compiler | Check `rust-toolchain.toml`, rustup directory overrides, and `RUSTUP_TOOLCHAIN`; use the repository pin |
| Incremental or profile drift | Correct the named file/environment source; regenerate profiles through setup |
| No hits on a check or final executable | Read eligibility; use setup's library proof instead of inferring a failure |
| Same-input eligible build misses | Compare compiler, target, features, profiles, cwd and tracked inputs; preserve the report |
| Missing SDK/native linker libraries | Repair the MSVC/Windows SDK installation or use Developer PowerShell; retain repo linker policy |
| Target already owned by another writer | Queue for that lane or use another registered lane/worktree |
| Stale checker/configuration | Run explicit setup; do not execute a binary built for another root's embedded policy |
| Unregistered package/target or zero selected tests | Repair registration/selection; do not manually choose a smaller passing suite |

## Qualification boundary

This documentation was checked against the inspected Windows tool inventory and
official upstream references. Its future loop/configuration steps require G1-G5
implementation and a real Windows setup/cache qualification before they can be
reported working. No Mac/Linux installation instructions or native qualification
are supplied by this guide. External-oracle, GPU, WASM, and private-application
prerequisites remain owned by their actual product lanes.
