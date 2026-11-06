# Independent funding finish review

Reviewer: `/root/funding_finish_independent_review`. Read-only review of
`bf5e74e0c3..03eb43385c` and the complete current dirty funding source. No
builds, tests, formatting, or source changes were performed by this reviewer.
The sole write is this requested review. All file hashes in
`funding-finish-source-snapshot.json` match the inspected files; deleted entries
are absent. The logs describe dirty snapshots, not a landed source commit.

Read: AGENTS.md, every coding-guideline document, database-foundation M3
D11/D12/D17 and the operational durable-owner contract, exact funding grants,
the lineage carry patch, prior independent rejection, the material source
clusters and their consumers, and the raw funding-finish logs. The coordinator
clarified that checkpoint_admission/ already belongs to this owner; the four
native carrier files are additional grants. Storage/residency, history-directory,
and integration opening ownership remain excluded.

1. **P1 — Changed cold roots cannot produce the next Complete checkpoint.**
   `checkpoint_complete/regions/covered.rs` indexes leaves by the selected
   root's exact commit, partition, and schema. In `regions/emission.rs:67-84`,
   a changed root misses that index and goes straight to
   `region.cell.capture_complete_for_request`. That existing storage method
   (`storage/residency/checkpoint_capture/complete.rs:68-70`) accepts only
   `lease_resident_full`; `cell/resident_leases.rs:96` excludes a backed cell
   even if its Full image has been loaded. A recovered root plus actual tail
   therefore has no admitted DATA emission path here. The removal of the
   historical Full reconstruction exposed this missing prerequisite.

   This is an observed product failure, not a hypothetical cost concern.
   `funding-finish-cold-recapture-corrected.log` shows the original-input second
   Complete, cold index continuation, and cancelled-plan retry courts failing
   with `Region(PartitionId(0)):SourceUnavailable`. The later Host log
   `funding-finish-host-bounded-reopen-kill-ack.log` independently shows both
   `killed_home_reopens_normalized_image_plus_nonempty_tail` and the unchanged
   `automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom`
   failing at `checkpoint_restart.rs:176` with that same capture denial.

   This violates D12's continuing automatic checkpoint/recovery contract and
   arch law 8's checkpoint-plus-bounded-tail architecture. Required correction:
   obtain the owning storage handoff and supply an honest request-funded
   current/cold DATA capture path, preserving exact component custody and member
   dependency closure. Do not revive canonical-body replay, weaken the Full
   serving admission, or treat a successful ordinary unfunded loader as a
   funding proof. Verification must keep the second-Complete assertions and
   pass the three Native cases, the two Host cases above, and the new cold
   zero-covered-body-read case under the ordinary policy.

2. **P1 — Required finish qualification remains incomplete.**
   The latest nine-case Host lane is **6 passed / 3 failed**, not green. Besides
   finding 1, the unchanged initializer-refusal court gets
   `CheckpointRecoveryRejected` for missing
   `application-principal.IdentityBinding`, before its expected typed
   `ExistingWorldRoot/OwnerUnavailable` refusal (`refusals.rs:51-62`). Its
   callback and retained-lease checks therefore never qualify that route.
   The new
   `cold_image_plus_tail_recapture_reads_only_the_uncovered_original_frames`
   exists in source but is absent from the supplied Host lane selection. No
   funding-finish log establishes the affected CDC cases or final boundary,
   agent-context, dirty line-cap guard, and Clippy gates for this complete
   snapshot. The Store court-check log also ends during compilation without a
   completion status; it cannot be counted as a passed check.

   This is the remaining qualification part of the prior rejection, under
   Testing laws 2/3/24/28, MENTALITY section 6, the grant's explicit proofs and
   AGENTS.md verification requirements. Required correction: resolve the
   initializer/index preservation or refusal-order issue through its actual
   integration owner, retain its existing typed assertions, then complete only
   the affected named lanes and mandatory gates after the source correction.
   I do not attribute that untouched Query installation defect to this funding
   diff, authorize edits to its protected owner, or infer green from earlier
   lane names. Earlier accepted proof can be reused where its seams remain
   unchanged; it cannot establish changed cold capture.

3. **P2 — The new growth court is outside the required cost category.**
   `checkpoint_admission.rs:261` mounts `covered_history_growth` directly.
   Its public test path is consequently
   `...checkpoint_admission::covered_history_growth::selected_state_recapture_work_does_not_follow_discarded_covered_update_bytes`.
   The test compares charged-work growth and constructs 1 MiB payloads
   (`covered_history_growth.rs:12-30,69`), but has no `cost` module segment.
   Iteration law 15 explicitly makes `cost` the category for growth/count/large
   scale courts and prohibits inferring the category from a filename. This
   places the new cost workload in ordinary-court membership. Required
   correction: put this growth responsibility beneath a real `cost` module,
   leaving the functional refusal courts in their appropriate ordinary family.
   Verify the resulting exact identity and focused selection without widening
   the workload or changing its assertions. Arch/perf law mapping: N/A;
   direct Iteration 15 and Testing 18 violation.

The previously missing lineage closure is repaired in the inspected source:
prior node/event descriptors and their immutable members are carried, the new
event-page directory copy retains request funding, and tail rows join the
existing inventories. Actual destination membership is checked for every
Carried member. Selected recipes now provide the actual saved roster; the
fresh helper alone uses an empty roster. Selected Complete Embedded refuses
inside the capture epoch before protected source reads, index resolution, or
canonical body demand. The separate original-frame export path remains
distinct. No wire/version change or covered-history fallback was found.

The Host close call uses the same installed, scoped, joined maintenance worker
and Home-limits root allowance; the explicit checkpoint still takes its caller
request. Exact boundary entries name the single Host registration file and the
direct Native refusal-court caller. I found no owner-excluded source edits and
no current changed/untracked Rust file above 400 lines by direct counting.

Credible positive evidence: `funding-finish-host-live-auto-close.log` passes
live zero-covered-body recapture, default-allowance automatic head41 with
ordinary 8M mutation callers, and D11 close exhaustion/journal reopen (3/0/0).
`funding-finish-discarded-branch-growth.log` passes the final discarded-branch
workload (1/0/0); source checks a common main state, genuinely larger original
covered frames, no covered canonical member reads, and broad comparative work
headroom. The later operational retry passes in
`funding-finish-growth-and-operational.log`; its earlier CaptureNotBound result
is superseded by sealing the same image before ACK. The growth failure in that
older two-case log is superseded by the final discarded-branch workload.
Earlier `funding-state-fresh-and-close-corrected.log` establishes the funded
close and explicit/tiny-Host controls (2/0/0), within that earlier snapshot.

The requested attribution is honestly scoped: old **16,236,875**, new
**16,240,012** units at head41/covered0, all originals uncovered. The raw
accounted log supports the total and the JSON reconciles the exclusive phase
split. It is not selected-prefix performance evidence. Package hash/copy is
3,798,489, Native projection 3,282,361, and Native/image seal exclusive work
2,631,494. The operation subset overlaps the phase view and is correctly not
an additional partition. Marker-free rebuild evidence is historical, not a
substitute for final snapshot qualification.

The three BC9002 sites in the older funding-host boundary log are untouched
repository debt, not permission to expand this funding scope. Their presence
does not excuse missing current checks or findings 1-3.

REJECT
