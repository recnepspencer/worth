# Round2 frame result

Seven actual dev-profile artifacts, complete PE unwind scan with PDB/COFF
attribution; zero unresolved or demangle failures. Exact input metadata and
hashes are retained in layout-after-frame-inputs.json and raw identified JSON.
The measurement completed at 29b4ecb5ce, before the unit-only topology repair.
Do not call this a rebuilt final-HEAD ordinary executable measurement.

| Owner function band | Before accepted census | After round2 |
| --- | ---: | ---: |
| >=128 KiB | 375 | 323 |
| 64--128 KiB | 485 | 470 |
| 32--64 KiB | 798 | 770 |
| Total >=32 KiB | 1658 | 1563 |
| Total >=64 KiB | 860 | 793 |

All four actual ordinary executables retain 1,048,576-byte PE stack reserves.
Consumer-root exits0; invariant_reads, required_feature and shared_execution
exit0xc00000fd. H10 is not met and remains integration-owned. The round2
remaining-path attribution is round2-remaining-main-frames.md. No stack setting,
thread or profile change was used.

The earlier single release comparison is reused: 83 named >=32KiB functions,
11 >=64KiB, 114152-byte maximum. 88 physical release entries remain unattributed
because their temporary objects are unavailable; do not claim complete release
symbol attribution. No new release build or release runtime was run this resume.

## Final artifact refresh, b2ba8dc3b1

layout-final-frame-driver.log verifies all six unchanged executable/PDB pairs,
refreshes the changed Root court and deduplicates the same complete seven-
artifact set. Counts remain1563:323/470/770; >=64KiB793. Zero unresolved or
demangle failures. PE reserves remain1048576 for all4ordinary executables.
No extra compilation or ordinary execution was used for this refresh.
