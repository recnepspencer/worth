# Independent funding landing review

Verdict: **REJECT for funding landing**. The current funding implementation has no additional supported material allocation/custody defect from this review, but its newly added selected-image court violates the sealed construction boundary. The user permitted only the three queued BC9002 sites to remain red.

## Review boundary and source state

Read-only review of `7d0e5189bc..ff0e8e88ca`, the complete funding source represented by `funding-final-source-state.json`, and the subsequent two-file Foundation commit `9d04a900506d5c435fb1a8f59538cde136d5b857`. No build, test, source edit, or proposed patch application was performed. This report is the only file written. No coordinator progress/report draft or previous review conclusion was used.

I read AGENTS.md, the coding laws, milestone-3 D12/D17, the approved cold-region and checkpoint-funding designs and the user ruling in `C:/programming/worth worktees/orchestration/m3/prompts/p3_reopen_cold_files.md` (especially line 31). I applied the QA-loop review criteria. Funding admission has an explicit user requirement and reproducing failures here; it is not speculative resource accounting.

All **60 recorded file hashes matched actual source bytes** when independently checked. HEAD is now `9d04a900...`; committing the Foundation files did not change their bytes relative to the snapshot. The final raw lane headers remain `ff0e8e88ca` plus dirty source. The root Clippy log predates two scoped, mechanical fixes already present in the final snapshot: moving history-directory impls before its test module and removing the redundant formatting reference in metadata/tests.rs. Those changes must not be presented as a measured successful Root Clippy rerun.

## Numbered findings

1. **HIGH / P1 — New selected-image recovery court calls a sealed factory without its exact caller grant.** `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/cold_materialization/selected_image.rs:52` calls `RelationalDurableRecoverySource::from_home_original_bytes`. The actual closed caller list contains the two granted contract-work entries, but no `native-home-original-source` entry for selected_image.rs. `funding-final-cost-boundary.log` exits **1**, with this **BC7013** and the three queued BC9002 sites. Thus it does not meet the user's “stays red only at your queued BC9002 sites” ruling. AGENTS.md's sealed-construction hard invariant and Architecture 3/16 apply: test usefulness does not authorize another construction door. The required outside-grant owner is **`tools/boundary-check/config/certification-construction-callers.txt`**, with the single exact role/path entry shown in `proposed-funding-final-selected-reader-caller.patch`. The proposal is reasonable for this real protected-source fixture, but it remains unapplied and ungranted. Obtain that exact boundary ruling, apply it, then rerun boundary-check and require only the three expressly deferred BC9002 diagnostics. Do not widen a pattern, relocate the caller merely to avoid the check, or pretend the proposed patch is current source.

2. **P2 / qualification limitation — Root Clippy has not produced a green result.** The actual `funding-final-root-clippy.log` reports **exit 101 and 18 lint errors**, with 10 in the relational certification court, one in the branch-reference compile-time court, and seven in the Native lib-test target. Two Native in-area errors were subsequently corrected in source as described above. There was **no 16-error rerun**; 16 is the source-based residual inventory. All 13 paths in `proposed-funding-final-outside-clippy.patch` are independently unchanged relative to `7d0e5189bc`, including dirty source. Under AGENTS.md they are untouched repository debt, not newly discovered authority to edit them or a funding-caused constitutional defect. They still prevent describing the required full Root lane as green. If that full lane must be closed in this task, its owners require an explicit exact-path handoff; the proposal alone supplies no authority. Relevant contract: AGENTS.md verification and scoped-debt rules; no separate semantic law violation is asserted for mechanically equivalent lint syntax.

   The 13 outside-grant owner paths, all relative to the repository root, are:

   - `crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/serial/relational/managed_source_work_account/public_only.rs`
   - `crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/serial/relational/managed_source_work_account.rs`
   - `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/index_catalog.rs`
   - `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/index_catalog/recovery.rs`
   - `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/nested.rs`
   - `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/row_read.rs`
   - `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/owner.rs`
   - `crates/worth-court-relational-branch-reference-compile-time/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/change_source_pass/selected_commit_mints_receipt.rs`
   - `crates/worth-relational/src/tests/durability/contracts/caller_association.rs`
   - `crates/worth-relational/src/tests/durability/medium/cold_envelope_matrix.rs`
   - `crates/worth-relational/src/derived_relation/evaluation/memo/tests.rs`
   - `crates/worth-relational/src/transactions/data/outcomes/created_entity_bindings.rs`
   - `crates/worth-relational/src/transactions/data/outcomes/created_relation_bindings.rs`

Finding 1 alone blocks approval. Finding 2 is deliberately distinguished from a new in-scope defect and from the specifically tolerated BC9002 queue.

## Allocation, lifecycle and authority assessment

The review traced these material risks against actual owners rather than inferring coverage from passing test counts:

| Risk | Actual implementation and distinguishing evidence |
| --- | --- |
| Ticket expires while native DATA survives edits | Native readmission reserves before allocation at SharedColumn/SharedMap nodes, fixed metadata backing, Extra/state backing, pin/slot/bitset owners, commitment nodes, and history openings. Shared owners retain those tickets; ResidentRegion holds its own shell ticket. The Head and Full lifetime controls retain actual payload after edits and release it at the final basis/runtime owner. |
| Wire/fold scratch expires before conversion consumes it | HistoryFolder returns `Arc<FundedHistoryImage>`, with its wire ticket and direct-base set attached. Each `FundedRow` owns its clone/Vec tickets. Loaders retain row custody through `partition_from_image_for_request`, and only then release scratch. The final native owners independently admit their backing; no ticket swap leaves both copies unfunded. |
| Wide contracts or malformed scalar columns cause work/allocation before admission | Contract quote charges the field traversal before visiting declarations. Raw REG6 preflight establishes each scalar sequence count before Serde construction; native arena conversion rejects mismatched scalar/SoA lengths before schema lookup and construction. The wide-contract work refusal and excess-scalar tests distinguish these failures. |
| Prefix/provider custody escapes or is borrowed from the wrong source | History custody walks real dependency locators under the request, builds a funded roster, calls the actual provider retention operation and stores the resulting shared owner in prefixes. The shared-custody control reaches provider admission, refuses with no owner issued, retries, then keeps one genuine provider hold through edits until the last basis drops. |
| Current/historical peer membership backing loses its bill | Request-aware projection reconstruction writes actual persistent map paths. Opening tickets include retained historical claims. The retired-peer test grows real retired generations in a remote relation partition, retains the endpoint basis through an edit, checks independently issued historical neighbor IDs, and observes final refund. |
| DATA emission bypasses normal materialization or another cell supplies authority | Complete capture obtains the counted Full lease by the ordinary demand path and invokes `capture_checkpoint_data_for_request` on that lease's own cell. Region emission requires prefix-free Full rows. It does not mint currentness, replace the actual root, or substitute another root. Backed serving admission remains in the existing root/kind admission path. |
| Covered history bodies or old mutation plans become capture cost | `FrozenOriginalInputs::read_canonical` skips the selected covered prefix; actual captured component roots replace historical mutation folding. Covered image meaning and flat region members come from the same selected base/directory; live ACK publishes base and directory together, and cold recovery readmits the bound source/image graph. Missing/inconsistent base refuses. The cold recapture filesystem assertion measures actual original-body read delta against the next selected coverage. |
| Incorrect general complexity claim | Complete emission is flat and covered reuse accepts only flat history members. Required retained history remains named S; uncovered work remains T. General admitted layered graphs are still legal and charge actual prefix hops and compared direct-base volume, with ordered navigation accounted separately. A once-decoded member is not falsely treated as a proof of linear cost for arbitrary alias graphs. |

The Foundation change is confined to `AspectValue::clone_allocation_bytes` and the pure large clone assertion in its existing tests file. InlineBytes now contributes the actual derived-clone length; the exhaustive match makes future variants visible. The Native `record_aspect_patch/controlled.rs` retained-versus-admitted guard is untouched, rather than bypassed. The previously failing 64 KiB fixture now constructs through the same real admission path before its controlled cold reads.

The reviewed test adaptations preserve asserted meanings and use actual selected member custody instead of requesting unsupported Embedded recapture. The cold-tail assertion is tightened to the coverage actually selected by this capture, rather than conflating another journal head with original Native frames. I found no removed/ignored regression, raised existing fixture budget, widened <=40 bound, or changed codec version in the reviewed funding source. The large byte/peer tests now live under the cost category. No new feature or generalized authority lane was introduced.

## Literal evidence and limits

- `funding-final-check.log`: check exits 0. `funding-final-cost-category-check.log`: final cost-category check exits 0.
- `funding-final-top-cost-controls.log`: **3 passed, 0 failed, 0 selected ignored**, after the Foundation correction and cost-category placement. `funding-final-cost-foundation-quote.log`: **1/0/0**.
- `funding-controls-native.log` is the older `05755fb...` **13-case packet, exit 100, 10 passed and 3 failed**. It is not all-green. Its nine existing Native cases plus shared-custody case passed. The three original failures were byte Head, byte History and the earlier duplicate-peer fixture; the final three-case log is their replacement evidence, not a rewrite of that old packet. The recorded Native production files in that older source-state capture match final source; Foundation is the deliberate later functional change. The later byte/peer cases also moved to cost and changed their fixtures, so the old failures cannot be quoted as current executions.
- `funding-final-host.log`: **5/0/0**, including cold image+tail recapture and close under Host allowance. `funding-final-bounded.log`: unchanged bounded-tail control **1/0/0**.
- `funding-final-store.log`: selected checkpoint/observer/recovery control **1/0/0**. `funding-final-cdc-recovery.log`: **3/0/0**, including scalar-length rejection.
- `funding-final-store-clippy.log`: **exit 0**, run after the two scoped lint fixes. `funding-final-root-clippy.log`: **exit 101**, as qualified above.
- `funding-final-cost-line-caps.log`: **exit 0, 59 dirty Rust files checked**. I also checked that every Rust file in the 60-file snapshot is physically at most 400 lines. `funding-final-agent-context.log`: **exit 0**. No generated context edit is needed.
- `funding-final-scope-audit.log`: **exit 0**, checks the six exact outside-area expression fixes, append-only two factory entries, the ten excluded paths, the exact Foundation function patch, and unchanged Native guard. The two later proposed handoffs remain unapplied.
- `funding-final-cost-boundary.log`: **exit 1**, **one BC7013 plus exactly the three permitted BC9002 sites**. It cannot be reported as “only the queued sites.”

I did not rerun any lane. I did not run an ignored, scenario, exhaustive, or query-scale test. These are scoped funding findings and evidence, not decoder/D13/digest/BC9002 queue clearance or whole-milestone approval.

**Final decision: REJECT funding landing until Finding 1 is resolved under an explicit exact boundary grant and the mandatory gate is rechecked.**
