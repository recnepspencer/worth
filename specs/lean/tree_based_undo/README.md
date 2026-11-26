# Lean: tree-based undo's lift bound and supersession rule

Two Lean 4 developments, unbounded, for public #56, Phase 2:

- `JumpLift.lean` mirrors `child_position` and `is_ancestor_or_self` (`document_history/journal/sources.rs`). It proves:
  - every jump is a level ancestor;
  - the lift is exactly parent-chain ancestry;
  - the lift reaches the level ancestor in `qsteps <= 4 * log2(x + 1) + 2` steps.
- `Supersession.lean` mirrors restored support's navigation rule (`correction_source/restored_support/navigation.rs`): `OccurrenceKind`, `superseded_through`, `needs_witness`, and the filter in `without_vouched_navigation`, plus the journal's `restores_descendant_of`. It proves:
  - only the latest vouched navigation supersedes;
  - unvouched means everything is kept;
  - a cancelled effect is the restored state, and never reaches the target;
  - a non-descending checkout does not vouch.

  It also proves the counterexample that makes `eligibility::compare` mandatory for facts outside the owned closure.

## Run

```sh
python C:/worth-orchestration/tools/lean/run_lean.py check specs/lean/tree_based_undo/JumpLift.lean \
  --theorem JumpLift.steps_log_bound --theorem JumpLift.jump_is_level_ancestor \
  --theorem JumpLift.isAncestorOrSelf_iff --theorem JumpLift.lift_reaches_level_ancestor_in_log_steps \
  --theorem JumpLift.J_succ
python C:/worth-orchestration/tools/lean/run_lean.py check specs/lean/tree_based_undo/Supersession.lean \
  --theorem Supersession.superseded_split --theorem Supersession.unvouched_keeps_everything \
  --theorem Supersession.cancelled_effect_is_restored_state --theorem Supersession.target_fact \
  --theorem Supersession.cancelled_never_reaches_target --theorem Supersession.restored_differs_only_by_line_writes \
  --theorem Supersession.fresh_fact_forces_descent --theorem Supersession.nondescending_checkout_does_not_vouch \
  --theorem Supersession.outside_closure_counterexample
```

Each must end with `Lean kernel audit PASSED`. The audit rejects `sorryAx` and any axiom beyond `propext`, `Classical.choice` and `Quot.sound`.

## Drift guard

Run `python specs/proof_mirror_check.py`. Each mirrored range carries `proof-mirror: <repo-path>:<start>-<end> sha256:<hash>`, and the guard names any range whose source lines changed. When one does, re-mirror the proof and update the line numbers and hash, then run Lean again.
