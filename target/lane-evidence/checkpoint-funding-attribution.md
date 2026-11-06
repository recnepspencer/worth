# One instrumented head41 capture

Command and full output: checkpoint-funding-instrumented-head-forty-one.log. Exactly one selected test and one explicit checkpoint; pass1/fail0/ignored0. This is diagnostic evidence, not qualification of instrumented production artifacts.

The earlier16.9M figure predates the single-framing repair. Current measured charge is16,236,875. Head41/covered0 -> generation1/covered41. Actual frame sizes: CHK529,549; CompleteSource651,914; admission image655,343; World16,561; companion261; Home1,324,405 bytes.

| Exclusive phase | Charged units |
|---|---:|
| store/world_image | 107,616 |
| store/world_coordinates | 16,561 |
| native/freeze_indexes | 0 |
| native/original_readmission | 2,045,743 |
| native/dictionary | 0 |
| native/root_replay | 818,066 |
| native/branch_history | 0 |
| native/schemas | 0 |
| native/initial_inputs | 0 |
| native/normalized_originals | 1,035,764 |
| native/history_metadata_indexes | 0 |
| native/canonical_inventory | 0 |
| native/current_regions | 0 |
| native/manifest | 0 |
| native/checkpoint_serialization | 0 |
| native/selection_custody | 0 |
| native/complete_source_serialization | 0 |
| native/verified_custody | 0 |
| native/admission_projection | 2,463,326 |
| store/native_and_image_seal | 2,631,494 |
| store/world_companion | 672,133 |
| store/package_hash_copy | 3,796,499 |
| store/previous_selection | 0 |
| store/manifest_copy_members | 1,324,405 |
| store/manifest_hash | 1,324,405 |
| Installation/selection/ACK residual | 863 |
| Total | 16,236,875 |

The Store native-and-image-seal sample contains Native child phases; its exclusive value subtracts those children. All exclusive phases plus the863 residual equal the enclosing request exactly.

History work is3,899,573: original readmission2,045,743; root replay818,066; normalized original-leaf hashing1,035,764. These unconditional whole-prefix passes are product defects; bigger caller limits are not a repair.

Operation subdivision (overlaps the owner phases above, so DO NOT sum these two tables):

| Instrumented operation | Charged units |
|---|---:|
| hash_serialization | 2,045,743 |
| hash | 6,814,614 |
| serialization | 1,282,268 |
| copy | 3,287,266 |

Dominant measured operation class is hashing plus serialization:10,142,625 units,62.5% of total. Instrumented copy charges are3,287,266. The remainder2,806,984 includes root replay818,066, World image/coordinate/companion work796,310, Native projected-directory joins1,191,745 and863 bookkeeping. World phase work is retained separately rather than falsely classifying its unsampled internal operations.

Freeze/index resolution, dictionary, branch/schema/catalog, region/CHK serialization and selection preparation report zero charged units in this run. Zero charged work does NOT mean zero CPU or no traversal: their existing consult-only contracts remain visible source work. The final producer bound must cover actual state traversal as well as metered byte operations.

Temporary instrumentation and its diagnostic-only court are removed. Cargo source fingerprints are invalidated by restoration; actual executed-artifact residue checking follows the next owned rebuild. No instrumented binary is qualified as production.
