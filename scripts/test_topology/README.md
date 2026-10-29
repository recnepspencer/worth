# Court codemod

Requires Python 3.11+, Git, and the workspace's Cargo toolchain/cache. The only
Cargo subprocess in the codemod is `cargo metadata --offline --format-version 1`.

```powershell
python scripts/test_topology/court_codemod.py plan Cargo.toml --out target/court-plans/root
python scripts/test_topology/court_codemod.py plan workspaces/worth-store/Cargo.toml --out target/court-plans/store
python scripts/test_topology/court_codemod.py apply target/court-plans/root/court-plan.json --root <checkout>
python -m unittest discover -s scripts/test_topology -p test_court_codemod.py -v
```

`plan` never changes the input tree. Its JSON and Markdown are sorted, contain no
timestamps or checkout-specific absolute paths, and can be compared byte for byte.
`apply` checks every move/write precondition before mutation, uses `git mv` for
tracked files, refuses stale inputs, and accepts its exact final tree on replay.
An apply blocker is a refusal, not permission to omit a test or exceed a ceiling.

The graph adapter calls A2's existing `report_test_topology.measure`; it does not
implement another closure or maximal-frontier algorithm. A2's Rust `surface`
naming rule has an explicitly documented Python spelling, covered by its naming
examples. External dependencies and fixture/testkit identities do not create
frontiers. Source evidence narrows the direct compile inputs before that graph
calculation. Product source, examples and benches conservatively retain any
dev-dependency they name.

Source inventory and module resolution precede ownership. Compilation readers
own Rust source; inspecting another court's source as data does not change its
compilation owner. Shared support gets one testkit module tree, preserving its
ancestry and original visibility. Visibility, macro expansion, dynamic manifest
paths, package identity assertions, feature conditions, behaviour-name decisions
and cross-category support are handed to fixers by court. The codemod does not
expand macros, grant visibility, rename behaviours, or invent category evidence.
It preserves feature predicates for explicit semantic repair; a plan is not a
claim that those predicates are already meaningful in the new package.

Effects include the complete generated manifests/module roots, original/final
hash preconditions, and every module, filesystem, fixture-manifest and Rust path
rewrite. The source edits are module mounts and paths. Test function bodies and
existing ignore attributes are retained. Ancestor scaffolds exclude their tests,
so delegating a child cannot duplicate a parent's tests.

Fixture proofs cover v3 frontier separation and refinement, category mapping and
overrides, ambiguity reporting, one-owner shared support, asset relocation,
area-based ceiling registrations, deterministic plans, stale-tree refusal and
idempotent apply. Apply also compares test-name and test-body multisets and checks
the emitted manifests through real offline Cargo metadata.

The root/Store plans and trial diagnostics belong under `target/`, not source
control. A trial's compile errors are migration work, not passing tests. The real
tree must remain unmigrated during Track B step 1.
