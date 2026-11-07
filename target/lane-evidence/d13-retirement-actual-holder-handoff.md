# D13 retirement: exact retained-root owner attribution

The unchanged
`checkpoint_keeps_exact_per_route_retirement_and_recovers_the_surviving_generator`
still fails its four-entity/four-relation shrink assertion. Full evidence:
`d13-codec-final-query.log` (packet 13/6/0). Its temporary owner probe is recorded
in `d13-retirement-probe-placement.json`; `capture.rs` was restored byte-for-byte.
No observer door, assertion edit, maintenance call or funding exemption was added.

The probe used only existing ordinary operations in the owned capture closure:
`history().immutable_commit_receipt`, `retained_application_commit_basis`, and
`observe_completed_application`. Each temporary share was dropped before capture.
It demands no canonical body. Exact rows are in
`d13-retirement-actual-holders.json`.

After retirement, the owner returns actual receipt bases for Native commits
4,5,6,7,8,9, respectively selecting root identities 4,5,6,7,8,9. The performed
World envelopes are also still present:

| Native receipt/root | World before→after | Native before→after |
|---|---|---|
| 4 | 0→1 | 3→4 |
| 5 | 1→2 | 4→5 |
| 6 | 2→3 | 5→6 |
| 7 | 3→4 | 6→7 |
| 8 | 4→5 | 7→8 |
| 9 | 5→6 | 8→9 |

This is actual custody attribution, not inference from a union-only census. The
earlier per-root census shows current root11/mirror removing the expected P0
records while these retained roots keep them in inspection's all-roots union.

Two concrete owner chains are established by source and this runtime probe:

1. Query receipt-basis store → retirable retained receipt basis → actual Native
   retention lease → observation → admitted basis inner → root.
2. Query completed/performed publication → shared terminal → consumed World
   publication → canonical envelope's before/after reference snapshots → World
   commits → Native component bases. Closed cached Ready answers can share this
   performed custody through `WorthQueryApplicationCommitReceipt::clone`.

The final D13 origins hold root-free canonical DATA/coordinates, weak receiver
affinity and schema/member custody. Source review found no root/basis field in
them. This does not establish which protected lifecycle operation should release
the live owner chains; changing a receipt clone to descriptive blindly would
risk its original-movement/follows-observation consumers.

## Ownership boundary

These exact owning files each return1 from
`git diff --quiet 7d0e5189bc codex/m3-p3-lifecycle -- <path>`:

- `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/provider/history_retirement.rs`
- `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/provider/session_commit/evidence.rs`
- `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/commit_receipt/mod.rs`
- `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/registry/lifecycle/release.rs`

They remain untouched. The needed decision/fix belongs to the lifecycle/history
owner: retire the actual completed/Ready performed custody and receipt bases at
their legitimate terminal, preserving replay provenance. No blanket reclamation
or weakened inspection/assertion is proposed. The named court remains red, so
this is not a D13 completion claim.
