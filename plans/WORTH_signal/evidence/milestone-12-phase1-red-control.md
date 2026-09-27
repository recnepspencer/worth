# Milestone 12 Phase 1 Red Control

This record preserves the required pre-repair control against inherited
revision `6b51e9c77`.

## Reproduction

From the repository that contains the revision:

```powershell
git worktree add --detach C:\forge_workspace\m12_red_control 6b51e9c77
Set-Location C:\forge_workspace\m12_red_control
git apply C:\forge_workspace\worktree_3\plans\WORTH_signal\evidence\milestone-12-phase1-red-control.patch
cargo test -q -p worth-signal phase1_red_control_translated_aspect_must_recompute_matched_leaf --lib
```

The retained patch creates a source whose `A` output is translated by its
immediate consumer into `B`, then places a `B`-filtered leaf downstream. After
the source changes `A` and the translator changes `B`, the matched leaf must
run a second time.

## Captured Failure

The command was rerun against detached baseline `6b51e9c77` during closeout
and exited with status `1`:

```text
running 1 test
tests::node_conditions::condition_admission::phase1_red_control_translated_aspect_must_recompute_matched_leaf --- FAILED

assertion `left == right` failed: RISK/B change must pass the matched leaf filter
  left: 1
 right: 2

test result: FAILED. 0 passed; 1 failed; 1041 filtered out
```

The baseline routing source also passes its one root `aspect` into
`transition_node_maybe_stale` for every transitive descendant. The behavioral
failure therefore convicts the copied-root-aspect implementation rather than
only documenting its shape.

The current runtime removes aspect and scope payloads from structural
transitive summaries. The financial
`quote_to_risk_aspect_translation_matches_fresh_truth_and_necessity` scenario
and the sealed certification run are the post-cutover green authority.
