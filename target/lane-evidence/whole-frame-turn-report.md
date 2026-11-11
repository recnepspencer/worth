# Whole-binary census checkpoint — IN PROGRESS

Commit a9e043a06f, `docs(perf): record the complete dev frame census and release
comparison`, records the tools, full sorted inventory, protected owners and
independent diagnostic reviews. No Rust/product code changed. No push.

The seven existing dev artifacts contain1658deduplicated owner functions
>=32KiB:375>=128KiB,485at64–128KiB,798at32–64KiB. Of860>=64KiB owner functions,
830are integration-classified and30are protected owner functions. The largest
is Query HistoryEdit::finish,1863336bytes. Causes are explicitly probable
source-family classifications, not measured Rust local layouts.

All572initially unnamed dev entries were resolved without a new dev build:
PDB section contributions identify the originating module; retained COFF
section size/checksum and every nonrelocation byte agree, and all matching
retained copies name the same normalized owner. Exact PE/PDB GUID/age pairs
and SHA-256 hashes are retained in the identified JSON.

The same3dev examples have368owner functions>=32KiB(31/117/220). The single
committed-profile release build has83named owners>=32KiB(0/11/72), plus88
explicitly listed unnamed entries. Temporary release example objects were
removed by the normal build, so its named totals are a lower bound. Every
release physical frame is measured; the maximum is114152bytes. Large frames
remain in release. No release or dev runtime success is claimed this turn.

The initial independent census review rejected duplicate display aliases,
incorrect generic-arrow parsing, receiver-only ownership filtering, incomplete
COFF attribution, comparison scope and missing recorded artifact identity.
All6were corrected. Fresh independent review
`whole_frame_corrected_census_review`: APPROVE for the diagnostic deliverable,
no material findings. It independently checked all10artifacts and rebuilt
nothing. This is not approval of an implementation plan or product fix.

## Evidence packet

Every listed log records exact command,HEAD,status,start/end and exit code.
All use source HEAD dae78e6ff76560ebbbc50a03c165bfc6be8f5055. They are diagnostic
lanes, so test pass/fail/ignored counts are N/A. The release build contains no
test executions. `whole-frame-release/build.log` additionally records the owned
Cargo command, compiler inputs, manifest hashes and unit-plan identity.

|Log under target/lane-evidence|Command role|Final exit|
|---|---|---:|
|whole-frame-dev-census.log|whole_binary_frames.py,7existing dev inputs|0|
|whole-frame-release-build.log|whole_frame_release_build.py,owned cargo build --release,3examples|0|
|whole-frame-release-census.log|whole_binary_frames.py,3release inputs|0|
|whole-frame-pdb-publics.log|initial raw-PDB pass,stopped at hyphenated consumer PDB path|1|
|whole-frame-pdb-complete.log|raw publics/procedures with actual underscore PDB fallback|0|
|whole-frame-release-publics.log|raw release publics/procedures|0|
|whole-frame-pdb-attribution-v2.log|raw dev linkage names retained separately|0|
|whole-frame-coff-attribution.log|initial COFF driver,invalid dynamic-reader import|1|
|whole-frame-coff-attribution-v2.log|all572dev COFF owner matches|0|
|whole-frame-pdb-identity-final.log|dev public/procedure attribution+GUID/age+hashes|0|
|whole-frame-release-identity-final.log|release public/procedure attribution+identities|0|
|whole-frame-release-coff-premise.log|release object availability:88missing temporaries explicitly retained|0|

Both failed driver logs are diagnostic-script defects, not product failures:
the consumer PDB basename uses underscores, and the initial embedded reader
import lacked __file__. The corrected paths/import pass. No lane is presented
as an executed product test or a green default-stack journey.

## Remaining work

The stack P1 remains integration-owned. The batch implementation design has
not been approved or implemented; all3implementation verification rounds remain.
The three dev mains, consumer-root and Bank parents have not been rerun this
turn. Prior accepted product evidence is unchanged, not relabeled as this turn.
Protected P-A Native storage/checkpoint/root and P-C World/Lifecycle files are
untouched. The other held findings remain with their existing owners.

Before designing retained operation/history backing, preserve original funding
AND cancellation: RequestAllocationControl owns a pinned cancellation listener
that otherwise drops when input capture returns. A lone zero-byte memory ticket
does not retain that listener. Existing phase, memory reservation and prepaid
storage APIs were read; no new funding door or partial product edit was added.

Counts:0cargo check/clippy,1release build,0focused dev builds,0test runs,
0ordinary main executions. No ignored,scale,exhaustive or scenario execution;
no stack/profile override,new target directory or cleanup. Phase gates are
unrun because no implementation phase completed. Disk remains above25GiB.
