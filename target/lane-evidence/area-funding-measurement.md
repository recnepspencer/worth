# Funding capture measurement

The required instrumented head-41 capture completed with **16,234,730** charged work units, versus the original **16,240,012**. Difference: -5,282. Both samples have covered position zero: all 41 canonical frames are uncovered inputs T. This comparison is an honest first-capture baseline, not evidence that selected capture has become cheaper. The discarded-history growth court and the selected live/cold body-read courts supply that evidence separately.

Exact phase observation: `area-funding-head41-instrumented.log` (1 passed, 0 failed, 0 ignored). Exact arithmetic reconciliation: `area-funding-head41-attribution.log`; machine-readable result: `area-funding-attribution.json`. The parser first reconciled the old log in `area-funding-attribution-parser-control.log`.

| Exclusive phase | Charged units |
|---|---:|
| Native uncovered-original readmission | 2,045,703 |
| Native normalized-original hashing | 1,035,744 |
| Native admission-image projection | 3,281,065 |
| Store Native capture/image seal remainder | 2,631,494 |
| World image | 107,616 |
| World coordinates | 16,561 |
| World companion | 672,133 |
| Store package hashing and copying | 3,795,818 |
| Store manifest member copying | 1,324,218 |
| Store manifest hashing | 1,324,218 |
| Selection/ACK residual | 160 |
| Total | 16,234,730 |

Store package hashing/copying is the dominant measured phase. Store's inclusive Native/image-seal phase is 8,994,006; the table subtracts its three charged Native child phases to avoid double counting. The remaining named Native phases observed zero metered units; zero metered units does not mean zero CPU work. The frame size was 1,324,218 bytes, compared with 1,324,898 in the old measurement. These byte counts differ; the measurement does not establish the cause of that difference.

The temporary meters were confined to the granted Native Complete preparation and Store checkpoint preparation owners. A one-run Host diagnostic recipe selected head41 under an explicit finite diagnostic request; ordinary fixture policy remained8M. This diagnostic recipe was deliberately retired after satisfying the coordinator's one-run attribution request. It was not a permanent regression court. `area_funding_instrument.py restore` restored each original file byte-for-byte against `area-funding-instrumentation-originals.json`, and removed only its own newly created diagnostic recipe. The regular cold filesystem court is qualified afterwards with the marker-free runner.

The cost contract remains O(S+T), plus ordered-index operations, for the selected captured state S and uncovered tail T. Selected capture carries verified selected metadata/member references and traverses state needed for changed regions; it neither reads covered canonical bodies nor folds those originals again. Materializing changed cold regions is charged to the actual caller request at allocation owners. Automatic and close-triggered checkpoints use the Host allowance; explicit checkpoints use the caller's allowance. No fixture budget is raised.

This exact attribution predates the final review correction that prepays the contract-field counting walk during cold materialization. It is evidence for the source state and command captured in its log. No second instrumented sample or claim of an unchanged exact charge at the later dirty source state is made.
