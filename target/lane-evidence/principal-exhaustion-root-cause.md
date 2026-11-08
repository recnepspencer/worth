# Restored principal-resolution exhaustion: measured root cause

Scope: the unchanged `checkpoint_absence_rebuilds_every_owner_call_like_fresh` parent, HEAD dbe3280daa804f95a9ec6375f85ae78d1cbcc6dd. Native storage/residency is granted. Exact subtree dirty checks in H5 and P-C returned no files. H5's Query/Store cutover holds remain excluded.

## Measurement before changes

`principal_cold_measure_run.py` reruns the existing dev libtest binary under Windows software breakpoints. It changes no budget, source, checkpoint proof or input. `principal-cold-measurement-driver.log` records command/HEAD/status/times/exit; `principal-cold-measurement.log` and `.json` contain the actual events and PDB-attributed call sites. Breakpoints begin at the first cold Full admission and stop at the first genuine WorkCeiling, then restore instructions and allow the unchanged court to fail (101).

One Full admission attempted 8,000,096 units in 36,263 checkpoints; the last 111-unit checkpoint refused. Thus 7,999,985 units in this observed window were admitted. The small pre-Full request prelude is outside the window; these are not asserted to be an exact complete-request total. The partial required-symbol scan made 1,894 callbacks charging 155,300 units, with 1,961 attempted insertions / 45 distinct symbols. The final field-revision callback is the exhaustion site, not the dominant cause.

Dominant capacity-derived charges:

| Owner | Units | What the number is |
| --- | ---: | --- |
| materialization_funding/current_image.rs current | 3,473,364 | Conservative simultaneous wire allocation quote, including cursor backing, spare/geometric capacity, BTree node slack, and the largest retained schema contract for every state entry |
| resident_history.rs projection_funding | 682,480 | Conservative projection backing plus traversal cursor backing |
| materialization_funding/aspect_state.rs contracts | 67,442 | Separate catalog clone/storage bound; not the dominant state charge |

The fixture at this point has 354 arena slots (175 entity / 179 relation), 358 historical adjacency incidences, 30 retained contracts. Field-revision scan had visited 294 slots when stopped. These are observed state dimensions, not a proposed allowance.

## Classification and authority review

**Case 1: memory-capacity bounds are being charged as executed work.** `vector` admits geometric growth and old/new coexistence; `tree` admits unused BTree node slots. `current` also uses the largest contract's backing for every aspect, although Foundation's actual portable export looks up and clones only the selected contract and actual value. It then charges the whole memory quote as work. Projection preflight similarly charges cursor and spare backing as work, while the original-request reconstruction driver separately charges actual claim cloning, comparisons, visits and installation.

Nearest working analogue: `durability/checkpoints/aspect_state_images/readmission_work.rs` explicitly separates work from conservative backing bounds; `readmission_funding.rs` reserves the latter. Foundation `aspects/portable/state_export.rs` and Native `checkpoints/images/arena_images.rs` show the actual exported values. `kind_projection/reconstruction/request.rs` routes actual reconstruction checkpoints to the same original request.

This is not a reason to weaken Full admission, trim the symbol closure, warm another owner silently, raise the 8M allowance, or remove readmission charging. Memory bounds remain necessary and unchanged. No checkpoint format or public API changes.

## Planned owner fix (review before implementation)

1. Keep all current conservative quotes, reservations, retention and drop order. Remove the equivalence between `Quote.wire` and work.
2. Add a private current-export work owner under `region_load/materialization_funding/`. Before export, charge actual fixed column/row copies, actual ordered-map entries, actual diagnostics/adjacency/bitset entries, actual aspect values and keys, and selected contract cloning plus catalog lookup navigation. Use checked arithmetic and the original request. Do not charge unused vector/node/cursor capacity or the largest unrelated contract per value. Existing per-visit preflight checkpoints remain.
3. Pass the already-retained schema contracts to this private preflight at the two existing current-image callers. No new public getter or door. Foundation still decides all schema/export refusals; the preflight cannot issue a proof.
4. Projection preflight retains the per-incidence charge and entire memory reservation. Actual projection reconstruction already has original-request checkpoints. Remove only the additional capacity-byte work charge; verify that the capture path also meters actual copies before retaining this deletion.
5. Add honest owner-level regression evidence for unused-capacity independence and selected-state charging where practical; the unchanged five public Topology parents are the required real-boundary regression proof.

## Verification plan / exclusions

One coherent source batch, then one Root Native `cargo check -p worth-relational --tests` (private owner API only). Focused builds run serially, with C: >=25 GiB: five Topology parents together; three D18 controls and graceful oracle seeds 0..2 together; two Bank parents with RUST_MIN_STACK absent. Any added Native owner tests run by exact name. No ignored bodies, exhaustive oracle, scenario suite or query-scale. Classify any failures before further edits. Fresh independent source/evidence review follows; no reviewer rebuilds.

### Design review correction (before product edits)

The first review accepted case 1 but rejected a missing capture precharge. `resident_history::load_full` additionally calls the ordinary no-op `kind_projection::capture`; its work is distinct from later reconstruction. Revised design adds a private original-request capture owner: prepay actual map/set navigation and copied claim/incidence payloads from the preflight's counted incidences, and route capture_controlled's slot/incidence/output checkpoints to the same request. Only the owned-Head path calls it; cold-region Full has no extra capture. All existing projection memory custody remains.

For selected contracts, build a temporary borrowed ordered index of the retained schema slice under its own original-request memory reservation. Fund index construction and exact lookup navigation; price the selected contract clone from that index, including reverse-key fallback and typed-denial work. Do not assume the state's embedded contract equals a possibly inconsistent catalog. This index is private preflight scratch, not an authority or a replacement for Foundation validation. No changes to checkpoint/catalog owners are needed.

### Source review correction (before first test run)

Independent review found that existing projection reconstruction callbacks counted visits but did not fully price claim copies and ordered operations; therefore merely retaining them would undercharge after deleting the capacity umbrella. The correction prices those actual operations directly in their existing owners: `kind_projection/reconstruction.rs`, `claim_copy.rs`, and `owned_history.rs`. It covers shell/endpoint copies, expected-map insert/remove/get, owned Vec moves, Head subset checks, prefix/suffix physical indexing, adjacency lookups and endpoint-set insertion. Installation keeps its already-funded request methods. No backing byte argument is converted into Work. No authority, memory reservation or public signature changes. Source reviewer accepts the correction pending final checks and courts.

## Final focused result and remaining owner boundary

The unchanged five Topology parents finish **4 pass / 1 fail / 0 ignored**. The absence parent now passes the original ConsumerPrincipalBinding stage, and fails later at `ProviderCommit / WorkExhausted` (`oracle/demand.rs:92`). D18's three controls plus the two registered graceful seeds (exclusive `0..2`) finish **5/0/0**. The two Bank parents finish **2/0/0**, directly with `RUST_MIN_STACK` absent. No assertion, serving allowance, Full proof or fixture was changed.

One further **no-build** rerun attributes the later denial in the same newly built Topology binary. `principal-after-measurement.json` and `.log` preserve every observed charge chain and the first WorkCeiling stack. Disassembly verifies the denying closure's captured work/units/ceiling pointers before reading them. The denying child meter has **410,016 admitted**, **9,932 denied**, **419,948 attempted**, **414,608 ceiling**. The observed checkpoint window from the first cold Full entry through that denial totals **8,004,917 attempted** units / **57,655 checkpoints**; these event totals are not asserted to be an exact ancestor-meter total. No limit was increased, and the test still exits 101.

The exact remaining path is:

`indexes/authority/maintenance/work.rs:144` (`MaintenanceWork::prepare`) -> `indexes/authority/maintenance/candidate.rs:202` -> `runtime/state/subsystems/indexing/generation_catalog/admitted_publication.rs:152` (`quote_batch`, `prepare(visits, bytes)`). The final 9,932-unit preparation quote appears 36 times in the observed trace, including the refused one. The quote combines checked map/scope insertion and copied branch-name work. The enclosing original-request maintenance scan is `indexes/authority/maintenance/request_payment.rs:47-56`.

These owners are **outside the current Native storage/residency + required-symbols grant**. Stop that part here under the scoped-work rule. It is not established that all remaining index work is accurate/proportional, nor that the fixture legitimately requires a larger allowance (case 3). It may require an index/publication admission correction; choosing a replacement without resolving that owner's prepaid-publication contract would be an approximation. Do not raise the cap or call the entire HAPPY journey fixed. The coordinator must route/grant the remaining Native index-generation owner. Native residency's concrete capacity-as-work defect is repaired, but the full parent remains open.

Diagnostic-only failure: the first post-fix PDB script copied an old UTF-8 BOM using the Windows default text encoding, producing a Python syntax refusal before any test launched. Reading it as UTF-8-sig fixed the diagnostic script; `principal-after-pdb-fixed.log` passes. This was an author harness error, not a product or test failure.


### Remaining contract identified by source inspection (read only)

`GenerationCatalog::prepare_deferred_batch` (`admitted_publication.rs:37-49`) calls `quote_batch` with `SelectedIndexReadWork::MAXIMUM_ORDERED_CONTAINER_ENTRIES`. `indexes/data/selected_read_work.rs:15` defines that as **usize::MAX**. `insertion_quote.rs:6-17,20-47` derives worst representable BTree descent and split capacity from that bound, with work separated from bytes. This is a prepaid *future* publication contract: the candidate may wait while other branches grow the catalog (the method's doc comment), and installation must have no new payer/fallible preparation. It is not another demonstrated backing-bytes-as-work defect.

The next owner decision is therefore concrete: how deferred index publication carries an honest funded geometry while preserving its no-new-payer/no-fallible-installation promise under the original serving grant. Using current catalog cardinality in place of usize::MAX without a reservation/guard would silently weaken that promise. The residency grant does not authorize this separate generation-catalog contract change. The trace is insufficient to certify case 3 or a complete required grant. The full HAPPY parent remains unresolved and must not be reclassified CLEANUP.

### Fresh independent review response

Review range `dbe3280daa..ee164dda9f`: one P1, **REJECT completion** because the required absence court still fails ProviderCommit. Agree, and leave that case explicitly open at the Native index-generation owner above. No additional confirmed defect in the nine-file residency patch; all 101 frozen hashes and focused/gate counts verified by the independent reviewer without builds. Earlier advisory reconstruction-work finding was fixed before final verification; the inclusive seed-range concern was withdrawn after reading the registered exclusive `0..2` range. No full-task approval is claimed.
