# Independent revised cold funding design and local Query review

Reviewer: `/root/cold_funding_revised_design_review`.

Read-only source review against AGENTS.md, the coding guidelines, M3 D12 and
its durable-owner contracts, the prior independent rejection, and the funding
design including its final bound correction. The source boundary included the
six granted residency owners and their actual demand/cache, REG6 framing,
history directory/capture, Complete emission/covered-reuse, and Query index
registration/readmission owners. No builds, tests or source edits were run.
This report is the only reviewer write.

1. **P1, resolved in design — Escaping allocation custody is now explicit.**
   `Arc<FundedHistoryImage>` keeps the decoded member and reservation together
   after `drop(folder)`; funded folded rows keep their wire storage charged
   through native reconstruction. The independently admitted final Full grant
   transfers to `ResidentRegion._memory`. Head decode/rebuild and the retained
   raw Head cache also receive their own grant. These final signatures and
   lifetime clauses supersede the earlier table's incomplete folder-scratch
   description. They cover simultaneous old and new backing rather than
   refunding wire DATA when traversal alone ends. Concrete preflight must still
   implement these promises before each allocation, including malformed
   collection lengths, relocation peaks, nested values and verification scratch.

2. **P2, resolved by the final bound correction — A directory pair was not a
   constant amount of verification work.** `history_fold.rs::verify_directory`
   currently derives bases by scanning member rows. Repeating this for distinct
   directory allocations could rescan a large shared decoded member even with
   pair deduplication. The final revision carries the funded, once-derived
   direct-base set with the member and names total compared base entries V.
   Its general bound `O(input + output + H + V)` plus ordered navigation replaces
   the earlier incorrect `O(input + output + H + D)` pair-count expression.
   Request charging explicitly includes visits, copies, hashes, comparisons,
   cache hits and navigation. Deep acyclic directories remain legal and pay
   their actual work; neither one decode nor acyclicity asserts a depth bound.

3. **P2, resolved by the final bound correction — The Home proof now cites the
   actual producer.** `region_capture/complete.rs` refuses any entity or relation
   prefix and emits an empty history directory. `checkpoint_complete/regions/
   covered.rs` carries a selected region only when its history has no direct
   bases; a layered region must take the DATA projection path. Thus the selected
   Complete base is flat, and postselection journal mutations retain actual
   flat row bases plus their suffixes. With member bases derived once, unrelated
   rows are not rescanned per directory check. This supports the stated Home
   `O(S + T)` body/materialization bound with S counting complete output state
   across changed selected regions, and the ordered-container term kept explicit.
   The ordinary export's eight-layer consolidation is a separate producer
   policy. It is neither general loader admission nor the primary Home proof.
   The existing discarded-canonical-history court proves its own history axis;
   the planned cold Home evidence must still observe zero covered canonical
   bodies and exercise the region reconstruction/funding axes.

4. **No blocking local Query finding — Zero-publication preparation preserves
   the recovered binding boundary.** The inspected `existing.rs` change skips
   binding only when both the actual Native commit catalog and recovered index
   definition inventory are empty. Any catalog commit or any definition retains
   `RequireRecovered`. The real `root/index_installation.rs` owner matches each
   required name/kind/scope and binds its owner-issued numeric ID; this posture
   does not register replacements. Native family readmission still runs.
   Existing preparation has no initializer/seed-publisher capability and still
   requires the actual World root before service. The unchanged
   `initializer_refusal_reopens_at_missing_world_origin_without_replaying_initializer`
   court checks the intended later `OwnerUnavailable` refusal, callback/contact
   counts and retained mutation lease. This is source-level approval of the
   local change; the court was inspected, not executed in this review.

Design verdict: **APPROVE**, with the final settled lifetime and bound sections
governing implementation. The same-cell counted funded Full DATA lease may be
checkpoint-backed; serving guards, the original Full rule, prefix-free emitter
and independently funded output remain intact. No maintenance side door,
canonical replay, format change or borrowed-prefix emitter is required.

Local Query source verdict: **APPROVE** by inspection, subject to the planned
focused evidence and required checks.

The four proposed private funding/test files remain **pending permission**.
This review grants no additional source scope and does not approve a funding
implementation or declare the whole funding path green. Concrete allocation
quotes, retained/refund controls, work-denial controls, retry behavior and the
named integration evidence remain implementation obligations.
