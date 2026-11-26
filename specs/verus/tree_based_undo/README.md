# Verus: tree-based undo's jump-pointer lift

`jump_pointer_lift.rs` is a Verus model of the history journal's skew-binary jump pointers (public #56, Phase 2). It transliterates, line by line:
- `LinePosition`;
- `child_position`, which reserve calls;
- `is_ancestor_or_self`;
- `position`.

They live in `workspaces/worth-query/crates/worth-query-execution/.../document_history/journal/sources.rs`.

## What it proves

- The initial one-node tree is well formed.
- `child_position` never panics, underflows or overflows. Inserting its result keeps the tree well formed, including the skew-binary jump invariant.
- `is_ancestor_or_self` never panics and terminates. It returns exactly parent-chain ancestry.
- That lift takes `qsteps(depth(node), depth(ancestor))` iterations and at most `2 * qsteps + 2` node probes. The logarithmic bound on `qsteps` is proved, unbounded, in `specs/lean/tree_based_undo/JumpLift.lean`.

## Run

```sh
python C:/worth-orchestration/tools/verus/run_verus.py check specs/verus/tree_based_undo/jump_pointer_lift.rs
```

It must report `25 verified, 0 errors`. The runner's gate rejects `assume`, `admit`, `external_body` and `unsafe`.

## Drift guard

Each mirrored source range has a header line `proof-mirror: <repo-path>:<start>-<end> sha256:<hash>`. Run:

```sh
python specs/proof_mirror_check.py
```

It exits 1 and names any range whose lines changed. When it does, re-mirror the proof, update the line numbers and hash, and run Verus again.
