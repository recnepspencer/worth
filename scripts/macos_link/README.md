# arm64 Darwin executable placement (#67)

Cargo declares `scripts/macos_linker` only for `aarch64-apple-darwin`.
It forwards the original compiler arguments to `/usr/bin/cc`. It does not
replace `[build].rustflags`, disable PIE, change page-zero protection, inspect
process memory, retry a launch, or impose a file-size budget. Other targets
and non-executable links retain their existing link behavior.

On the affected Mac, `mach_vm_region_recurse` observes the shared-cache
submap beginning at `0x180000000`; the first cache content reported by dyld
begins at `0x181e4c000`. An executable ending just below the submap can
overlap its reservation after a positive ASLR slide. The kernel refuses exec with `OS_REASON_EXEC` (9),
`EXEC_EXIT_REASON_MAP_EXEC_FAILURE` (21). Some larger images instead cause a
private cache relocation, so executable byte size is not a safe proxy.

The adapter places `__LINKEDIT` at 512 GiB, leaving code, data and TLS where
Apple's linker expects them. Every mapped segment must then either end below
6 GiB with 80 MiB reserved for ASLR, or start at/above 512 GiB while
remaining inside macOS's 47-bit user address range, including the slide. These addresses
come from the arm64 cache boundary and fixed commpage/GPU reservations, not a
fitted byte limit. The latter reservations extend through 448 GiB.

If code/data still overlap the reserved range, an image without TLS gets a
planned geometry/emit stage: translate its code/data together above 512 GiB,
then place linkedit after them. Apple's current linker cannot translate Rust's
TLS-bearing image this way (`thread-locals too large. Max 4GB`). Such an unsafe
TLS layout is refused **at build time**, with `MACOS_EXEC_LAYOUT`, rather than
admitted for intermittent execution. This is the explicit unsupported-layout
fallback. Safe images have one link; no failed link is retried. Output is
published only after final geometry, PIE and page-zero validation.

`python3 scripts/check_macos_layout.py EXECUTABLE` checks historical binaries
without launching them. The test runner also diagnoses unsafe historical
layouts after an actual launch error or SIGKILL, preserving that result and
executing the child exactly once.

The architectural evidence is in Apple's
[shared region definitions](https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/mach/shared_region.h)
and [VM mapping implementation](https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/vm/vm_map.c)
(`vm_map_get_max_aslr_slide_section`: three possible 32 MiB section offsets,
plus less than 16 MiB page randomization; maximum 80 MiB).
The published source predates this host's kernel; actual cache placement and
exec reason were measured separately. ASLR remains enabled in all acceptance
launch samples.

Run tooling courts with:

```
python3 -m unittest discover -s scripts/macos_link -t .
python3 -m unittest discover -s scripts/test_cost -p test_account.py
```

Merging the declared linker setting invalidates all arm64-Darwin Cargo caches
once. Rebuild artifacts rather than reusing the old unsafe executable.
