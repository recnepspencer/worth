# Pinned compiler and confined macro substrate

This is a compiler/broker review checkpoint. The ordinary `implementation-edition
build` command still refuses proc-macro units. No new installed-edition authority
or consumer facade is exposed.

`macro-guest` provides a distinct guest `proc_macro` API. `macro-host` owns live
native compiler objects, invocation-scoped handles, source observations, typed
failure, Wasmi interpretation and bounded execution. There is no ambient WASI
implementation. File/environment calls trap even when guest source ignores their
Results. Unknown imports, operations and invalid object handles refuse.
Structural depth/cardinality and cumulative native traversal work are bounded
separately from Wasmi fuel, including invisible groups. Native text parsing has
a 256-byte per-call budget before entering the compiler parser; larger parsing
requests refuse with the typed work-limit failure. Token-based construction can
build larger streams within the carried structural bounds.

`tests/confined_serde.py` verifies the locked Serde Derive archive and its registry
dependencies, removes the two native derive entry attributes and adds explicit
Wasm exports. This is an effective confined variant, not the original native
invocation. Its recipe includes original/effective source, guest API, guest bytes,
runtime and lock identities. The runtime provides a deterministic sixteen-byte
seed instead of native entropy for WASIp1 standard-library HashMap seeding. Other
WASIp1 effects trap; arbitrary host/WASI behavioral equivalence is not qualified.

The fixture builds a fixed trusted native shim, then runs the actual Serde/Syn
chain over live compiler tokens. The consumer exercises nested structs/enums,
nondefault values, raw names, native cfg, hygiene, joined spans and delimiter
locations. A second guest build must reproduce identical bytes. This fixture's
recipe is descriptive evidence; supplying it or arbitrary guest bytes cannot
construct `CompiledImplementation`.

The compiler driver uses the pinned rustc callback after actual expansion and
before code generation. It rejects expanded native assembly/foreign declarations
and binds its audit digest to returned artifact bytes in returned dep-info. The
receipt travels with those artifacts through sccache; an unchanged hit returns
the original invocation receipt, and an included-source change invalidates it.
The audit digest is diagnostic, not a portable semantic edition. The driver
currently accepts a finite library-compilation argument subset and refuses
external artifacts, implicit library search, response files and unqualified
compiler options before rustc can load a native macro/backend. Joining actual
Cargo flags requires the qualified unit-context owner. There is no bypass or
consumer-provided allowlist.

## Owned toolchain staging

Run from the repository root:

```powershell
python tools/implementation-edition/compiler/stage.py --storage target/m271-editions/toolchain --driver-dir target/m271-editions/driver/debug
$env:CARGO_HOME=Join-Path (Get-Location) 'target/m271-editions/cargo-home'
$env:RUSTC_WRAPPER='sccache'
Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
$env:RUSTC_BOOTSTRAP='1'
$dev=Join-Path (Get-Location) 'target/m271-editions/toolchain/rustc-dev-1.94.0-x86_64-pc-windows-msvc/rustc-dev/lib/rustlib/x86_64-pc-windows-msvc/lib'
$env:CARGO_ENCODED_RUSTFLAGS=@('-L',"all=$dev",'-C','prefer-dynamic') -join [char]31
cargo build --locked --offline --manifest-path tools/implementation-edition/Cargo.toml --target-dir target/m271-editions/driver -j 4 -p implementation-edition-compiler
Remove-Item Env:CARGO_ENCODED_RUSTFLAGS
cargo build --locked --offline --manifest-path tools/implementation-edition/Cargo.toml --target-dir target/m271-editions/broker -j 4 -p implementation-edition-macro-host
Remove-Item Env:RUSTC_BOOTSTRAP
cargo build --locked --offline --manifest-path tools/implementation-edition/Cargo.toml --target-dir target/m271-editions/broker -j 4 -p implementation-edition-macro-guest --target wasm32-wasip1
python tools/implementation-edition/tests/compiler_driver.py --driver target/m271-editions/driver/debug/implementation-edition-compiler.exe
python tools/implementation-edition/tests/confined_serde.py --target target/m271-editions/broker --reproduce
```

Staging checks official pinned archive checksums and extracted compiler artifacts
(debugging source bundled with rustc-dev is not a driver compilation input). Only
owned target storage is written. The driver verifies adjacent runtime bytes and
the exact owned sysroot tree; their identity also enters its sccache compiler
identity. Bootstrap is confined to tooling using pinned compiler internals and
the trusted native Span API, not consumer or guest compilation.

The next dependency-complete owner must derive qualified guest/shim construction
from verified source/build context, admit exact extern units before loading, and
carry broker source observations into the same compiler's artifact/component
binding. These two subsystem proofs have not yet been joined. Generator build
dependencies, real SDK/CAD proc macros, Git/native closure and installed
producer/output/partition, checkpoint/writer and cold-reopen adoption remain
required. Ordinary macro admission must stay refused until those owners exist.
