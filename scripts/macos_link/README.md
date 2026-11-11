# arm64 Darwin executable placement (#67)

Cargo declares `scripts/macos_linker` only for `aarch64-apple-darwin`.
The adapter forwards original arguments and response files to `cc`, preserving
the compiler override selected through `PATH`, as Rust's prior default did.
It does not replace `[build].rustflags`, disable PIE, reduce page-zero
protection, retry a launch, or impose a file-size budget. Other targets and
non-executable links retain their existing link behavior.

The entry points declare `/usr/bin/python3 -I` (Python 3.9 or newer, supplied by
Apple's Command Line Tools). Linking already requires those tools. A missing
interpreter is an installation error; there is no PATH-selected Python fallback.
Isolated mode ignores Python environment and user-site imports. Imports
explicitly select the script directory, including under
`PYTHONSAFEPATH=1`. The supported gate retains the repository's existing
Python 3.11+ requirement. Modules used by the linker are standard-library only.

On the affected Mac the shared-cache submap begins at `0x180000000`; dyld's
first reported cache content begins at `0x181e4c000`. Main-image ASLR alone
is bounded by 80 MiB. **That is not the whole refusal band.** XNU places dyld
after the main image's highest slid address plus its independent loader slide,
rounded to the 16 KiB page boundary. The loader slide is below 4 MiB, and the
installed arm64 dyld's mapped extent is read from `/usr/lib/dyld` load commands
(bounding every installed arm64 subtype). Here that extent is `0x16c000`: the combined
low-top reservation is about 85.4 MiB, with rounding applied explicitly.
Missing or unsupported loader evidence gives `LoaderBoundsUnavailable`, never
a guessed safe margin. The kernel refuses collisions as `OS_REASON_EXEC` (9),
`EXEC_EXIT_REASON_MAP_EXEC_FAILURE` (21).

The architectural sources are Apple's
[main and loader slide bounds](https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/vm/vm_map.c)
(`vm_map_get_max_aslr_slide_section`, `vm_map_get_max_aslr_slide_pages`,
`vm_map_get_max_loader_aslr_slide_pages`, lines 21905–21944 in the reviewed source)
and [dyld placement](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/mach_loader.c)
(lines 1286–1293). Published XNU predates this host; actual mappings and exec
refusals were measured independently. Some larger original images trigger a
private cache relocation, so file byte size is not a safe proxy.

The adapter places `__LINKEDIT` at 512 GiB, preserving code/data/TLS placement.
Every mapped segment must end below the 6 GiB cache boundary with room for
main ASLR, or begin at/above 512 GiB. Whole-image admission additionally checks
dyld's placement and the macOS user-address limit. **Published driver output
must have its top mapped segment at/above 512 GiB**, independently of the low
segment rules. This is beyond the fixed commpage/GPU reservations through
448 GiB. A non-TLS image with unsafe code/data gets a planned geometry/emit
stage translating them together; an unsupported TLS layout is refused at
build time. Final validation precedes artifact publication, retaining PIE and
identical page-zero geometry. No failed link is retried.

Original `@file` argv is passed unchanged to `cc`; GNU token expansion serves
inspection only. A final `-o` overrides the inspected destination for staging.
Command/response errors, including E2BIG, use `MACOS_LINK_COMMAND`; they are not
misclassified as layout refusal. The standalone inspector models historical
low-top images; the runner uses that same owner after a real failure while
preserving the original outcome and executing the child once.

Ad-hoc signing and native execution were verified. **Release notarization and
symbolication have not been verified**; follow-up [#106](https://github.com/recnepspencer/worth/issues/106)
tracks Developer ID signing/notarization/stapling and dSYM/debugger/crash
symbolication. Canonical SDK payloads and commitments are unchanged.

Run the registered tooling courts through:

```
python3 scripts/ci/query_gate.py tooling --out target/linker-tooling-gate
```

The full gate includes `tooling`. Merging the declared linker setting changes
the arm64-Darwin artifact fingerprint once. Rebuild old unsafe executables.
