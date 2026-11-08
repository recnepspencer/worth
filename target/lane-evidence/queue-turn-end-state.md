# Queue turn-end state

IN PROGRESS. Review parent1f9f39b138 plus frozen5 source files in queue4-source-state.json. This commit records the reviewed first public-court migration; it does not complete D13, named digests or BC9002. Nothing is parked; no integration merge or push.

## Commits already landed this turn

- d0061f1327: exact direct same-directory reopen reproduction passes1/0/0 with RUST_MIN_STACK unset; no product edit. Independent review APPROVE.
- 5e3baae5eb: D13 required-owner handoff, independently approved prerequisite stop.
- 1f9f39b138: named-digest prerequisite at integration-owned Foundation encoding, independently approved stop.

## Current partial migration

The source name recovered_nonhead_envelope_custody_ends_with_the_last_inflight_read moved exactly once into the external Native court. It uses actual public replay shares, their same-body pointer identity and intrinsic read funding, source failure/read counts, exact receipt, unchanged frontier/offers and retry. Payload release at final share drop is observed independently of fixed control-allocation retention, whose full refund is checked after runtime drop. Producer/recovery and read ledgers are separate; original unlimited ceiling is preserved. The other4 original lifetime tests remain unchanged. Native/public owner checks and new external named court are green. Detailed13-lane logs/counts are in queue-resume-lanes.md.

## Conditional outside-area files changed

All5 pass git diff --quiet7d0e5189bc codex/m3-p3-lifecycle --file (exit0):

- crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/cold_envelope_custody.rs
- crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/mod.rs
- crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs
- crates/worth-testkit-relational/src/cold_recovery.rs
- crates/worth-testkit-relational/src/lib.rs

No production Native API, new observer, authority conversion, macro, feature, Cargo profile, budget ceiling or boundary pattern changed. The runtime-root baseline is unchanged.

## HAPPY open items and precise owners

- D13 / Lifecycle: application_installation/construction.rs under worth-query-execution differs from7d on Lifecycle (exit1). It owns actual checkpoint decode/publication. A wrapper/alias would leave the interim path names; shared owner cutover requires coordination. Full path and proof: queue2-d13-owner-block.md. No D13 door or frozen callers list is claimed.
- Named digests / integration: current Foundation CanonicalDigestId lacks Serde, whereas integration owns category_ids/canonical_digest_wire.rs. Those definition files are outside item3 grant. No parallel codec/cherry-pick/unnamed merge was made. Raw owned digest fields still await named-type cutover; transferred original_image.rs and branches.rs untouched. Proof: queue3-named-digest-prerequisite.md and exact search.
- BC9002 / protected root owner: branch/root_tests.rs and branch/root_authoritative_payload_tests.rs (plus transitive root_tests/request.rs) still instantiate the local provider. User forbids edits to branch/root_*. Deleting the local attachment without migrating those consumers would need another local trait bridge, false memory fixture or skipped/changed tests, all forbidden. Exact names/lines and the independent consumer map: queue4-forbidden-root-owner-handoff.md, queue4-consumer-migration-audit.md/.json. The audit distinguishes actual durable consumers from mixed memory/value helpers;262 direct paths are not a migration count.
- BC9002 / P-A after that handoff: migrate every remaining provider-dependent court, split meaningful pure value units where appropriate, extend only external Native testkit provider behavior, remove local adapters/hooks, shrink baseline and require zero findings. Current gate still has all3 findings; no completion claim.
- Clippy / integration after master merge: scoped10 sites/7paths stay unchanged under standing rule3. Their exact path/lint list is in queue-resume-lanes.md and queue4-clippy-diagnostics.json. The previously accepted full-Root29-site list is retained in funding-landing-turn-end-state.md.
- Integration merge / coordinator: no head named this turn, so FUNDING MERGED is not reported. Merge at a clean commit point when named, then requalify affected seams.

No ignored tests, query-scale, exhaustive Home oracle or Store scenario suite ran. No disk cleanup or other worktree edits occurred.

Final fresh independent review: APPROVE for the5-file partial migration. Findings2(P1 protected-root dependency) and3(P1 zeroBC acceptance unmet) remain explicit OPEN owner items; finding4 is the permitted master-changed lint skip. Resolution: queue4-partial-review-resolution.md. No post-review source bytes changed. All13 current lanes retain their actual counts and exits.
