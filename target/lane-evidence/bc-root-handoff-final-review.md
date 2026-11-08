# Independent final review: root consumer owner handoff

Reviewed source HEAD: `5692704eb7bf41cc9c182c1506067b94a7836eb5`.
Verdict: **APPROVE the documentation/evidence handoff and exact production-owner stop.**
This does not approve completed consumer migration, adapter retirement, baseline
shrink, zero BC9002, D13 or named-digest readiness.

Reading boundary: the six new handoff/preflight/boundary-review artifacts and
`p3-reopen-progress.md`; grant 12 and its exact five-assertion handoff; the
three newly granted test paths; their private root subjects, public basis and
branch facade, replay access, artifact readmission, publication cost recorder,
and local/external medium attachments. Reviewed AGENTS.md, the QA-loop review
instructions and relevant architectural, mentality, performance, iteration and
testing laws. Formed the source conclusions independently of the prior audit.
No build, test, Clippy, enforcement run or source edit was performed.

## Numbered findings and disposition

1. **P1, existing task blocker correctly reported; no handoff defect.**
   `request_readmission_matches_real_root_and_keeps_only_actual_retained_owners`
   does more than compare public recovered data: `root_tests/request.rs:21,38`
   invokes both private DATA constructors over a genuinely published root;
   line 49 compares the exact private `regions` Arcs; lines 50-59 require the
   original alias/drop sequence to retain and finally release the invoking
   request's backing. `root_readmission/request.rs:8,87` declares both
   constructors `pub(crate)`, while `root.rs:138` keeps the actual root
   crate-private and its regions private. Public admitted basis retains that
   root behind `basis.rs:166`'s crate-private field; its public descriptors do
   not return the region Arc. Public recovery plus ledger observations cannot
   distinguish identical root content backed by different region allocations,
   or establish the same exact DATA owner lifetimes. A root-free local test
   cannot invoke these subjects without `ExecutionRequest`, itself a forbidden
   source-court root; a hand-built previous root loses the authentic original
   oracle. External providers can supply bytes and physical fates, not obtain
   Native's private region owners. No permitted tests-only route preserving
   this complete claim was found. Root cause: the existing assertion boundary
   and public court boundary expose different facts. Required owner action:
   decide an honest production separation or explicitly revise these exact
   claims; do not add a test port or local trait bridge. Required later proof:
   reviewed assertion equivalence and genuine public courts, followed by
   adapter/hook deletion, baseline shrink and the required gates. Applies:
   iteration law 2, testing laws 2, 3, 6 and 20; MENTALITY 11 and 18; the
   user's explicit production-root stop condition.

2. **P1, existing task blocker correctly reported; no handoff defect.**
   `request_root_refusal_preserves_original_and_releases_partial_backing`
   deliberately calibrates the native controller, then grants exactly
   `controller + size_of::<RelationalBranchRoot>() - 1`
   (`root_tests/request.rs:77`) before invoking the private unchanged-root
   constructor. It observes typed memory exhaustion, zero retained partial
   charge, later cancellation, and unchanged original axes. That constructor
   immediately reserves the actual root (`root_readmission/request.rs:96`).
   Whole recovery has other reservations and owners; its earlier exhaustion
   would not prove this boundary. Neither the actual private root type/size nor
   this constructor is on the public facade. Replacing the test with a public
   refusal/retry test loses the disputed constructor claim; retaining it in
   `src` still constructs request roots. This independently suffices for the
   stop even if other hash/value portions are migrated. Owner action and later
   verification are as in finding 1. Applies: iteration law 2, testing laws 2,
   3 and 20; architectural law 23; the explicit stop condition.

3. **INFO, remaining assertion map is accurate.** The ten named tests match
   the inspected source. The new/reused-schema counters and schema allocation
   ID are private; the public recorder aggregates authoritative bytes and
   region metrics. The schema/visibility, strategy and transition assertions
   submit mutants to the retained authentic root's private relink gate;
   completeness mutates its private committed visibility. Artifact admission
   and normal recovery may refuse earlier and do not preserve those exact
   claims. Request-funded visibility additionally calls the private branch
   operation. None is one of grant 12's five retired assertions. Pure hash and
   buffered-preimage portions can be split independently, as the handoff
   explicitly states; they do not resolve findings 1-2. The existing merge
   test already uses a memory runtime and is correctly excluded from the
   provider-dependency claim. No durable scenario was converted to memory.

4. **P2 evidence wording issue corrected during review; closed.** Initially
   the handoff associated `queue4-boundary.log` with unchanged source at the
   current HEAD. The log actually records HEAD `1f9f39b138` and a dirty tree.
   Both final artifacts now explicitly call it historical precommit evidence,
   not a fresh `5692704eb7` run. Independently verified that the three named
   adapter files have no diff between those commits. The saved log contains
   exactly three BC9002 findings and exit 1, with 0/0/0 test counts. No current
   boundary pass is claimed. Correction verified by rereading the revised
   paragraphs and the original log. Applies: testing laws 3 and 24;
   MENTALITY 18. No remaining issue.

## Evidence and limits

Independently reran all three exact `7d0e5189bc..codex/m3-p3-lifecycle`
quiet-diffs and the three Lifecycle worktree status probes: every command
returned 0 and every status probe was empty. Lifecycle resolved to
`d40751936a8b9299f72a8e75fed5ae6d6bfd4828`, matching the saved preflight.
Current HEAD matches the handoff. Source/baseline status is empty; the dirty
set contains only the stated evidence/progress files. The saved preflight
script, JSON and log agree on all 13 command results, including the expected
facade search exit 1 and empty output. Its exit 0 and 0 passed / 0 failed /
0 ignored are a source-boundary probe, not runtime qualification.

The final artifacts keep all unrun gates explicit, retain BC9002 as open,
and leave D13/digests waiting for the named combined integration head.
The stop is an actual excluded-owner boundary, not an inference from
difficulty. No supported material defect remains in this handoff candidate.
