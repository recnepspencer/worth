# Query execution split scripts

Requires Python 3.11+ and Git. `structural_measures.py` also runs
`cargo metadata --offline`; nothing here builds, times or mutates the tree.

```text
python scripts/query_split/structural_measures.py --out plans/query-split/measures/master
python scripts/query_split/module_coupling.py report workspaces/worth-query/crates/worth-query-execution/src/lib.rs domain_computation::primary_graph --out plans/query-split/stage2/primary_graph
python scripts/query_split/module_coupling.py baseline workspaces/worth-query/crates/worth-query-execution/src/lib.rs domain_computation::primary_graph --out scripts/query_split/primary_graph_layering.json
python scripts/query_split/module_coupling.py check workspaces/worth-query/crates/worth-query-execution/src/lib.rs scripts/query_split/primary_graph_layering.json
python scripts/query_split/churn_replay.py commits --ref master --count 400 --threshold 150 --out plans/query-split/stage3/churn-commits.txt
python scripts/query_split/split_candidates.py --out plans/query-split/stage3/candidates
python scripts/query_split/churn_replay.py score plans/query-split/stage3/churn-commits.txt plans/query-split/stage3/candidates/*.json --out plans/query-split/stage3
python -m unittest discover -s scripts/query_split -p test_query_split.py -v
```

- `module_graph.py` reads the library once through `scripts/test_topology`'s module tree,
  resolves every `use` and rooted path (including glob and named re-export chains) to the
  module that defines the item, and drops test-gated families and test-gated items on both
  ends. A reference is a distinct (source file, defining item) pair.
- `module_coupling.py report` gives strongly connected components, a greedy
  feedback-minimizing layering (`coupling_graph.py`) and the upward edges with their items.
  `baseline` freezes that layering; `check` is the ratchet (no new upward edge, every new
  module placed, stale baseline edges removed in the same change). It runs in about half a
  minute as plain Python (law 21).
- `churn_replay.py` maps each listed commit's crate paths onto a candidate partition and
  counts production lines outside the touched crates' reverse-dependency closure: what a
  compiler cache could reuse. The commit list is committed so the score is reproducible.
- `split_candidates.py` derives partitions from structure only. Candidates named
  `-hypothetical` drop their upward edges and report how many references that assumes.
