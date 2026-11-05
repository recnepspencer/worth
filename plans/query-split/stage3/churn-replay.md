# Churn replay

Commits: 400 (mass-edit threshold: more than 150 files).

| Candidate | Valid | Crates | Mean reusable lines | Median | Mean reusable crates | Mean recompiled crates | Parallel width |
|---|---|---:|---:|---:|---:|---:|---:|
| today | yes | 1 | 0.0 | 0.0 | 0.0 | 1.0 | 1 |
| vocabulary-below | yes | 2 | 3657.5 | 3891.0 | 0.94 | 1.06 | 1 |
| vocabulary-below-facade-above | yes | 3 | 5903.6 | 3891.0 | 0.94 | 2.06 | 1 |
| layered-3-hypothetical | hypothetical | 3 | 87438.3 | 0.0 | 0.67 | 2.33 | 1 |
| layered-5-hypothetical | hypothetical | 5 | 99233.4 | 48681.0 | 1.33 | 3.67 | 1 |
| layered-8-hypothetical | hypothetical | 7 | 106085.0 | 72886.0 | 2.28 | 4.72 | 1 |

## today

The single crate as it stands.

| Crate | Production lines | Transitive dependents | Fan-in x size | Touched by commits |
|---|---:|---:|---:|---:|
| worth-query-execution | 454268 | 0 | 0 | 400 |

## vocabulary-below

Modules that cannot reach the largest cycle form one lower crate.

| Crate | Production lines | Transitive dependents | Fan-in x size | Touched by commits |
|---|---:|---:|---:|---:|
| worth-query-execution | 450377 | 0 | 0 | 400 |
| worth-query-execution-vocabulary | 3891 | 1 | 3891 | 24 |

## vocabulary-below-facade-above

The vocabulary crate below, and the modules above the cycle on top.

| Crate | Production lines | Transitive dependents | Fan-in x size | Touched by commits |
|---|---:|---:|---:|---:|
| worth-query-execution | 1175 | 0 | 0 | 85 |
| worth-query-execution-core | 449202 | 1 | 449202 | 396 |
| worth-query-execution-vocabulary | 3891 | 2 | 7782 | 24 |

## layered-3-hypothetical

The greedy order of 106 module nodes cut into 3 line-balanced crates, top first.

Hypothetical: assumes 2045 upward references are removed first.

| Crate | Production lines | Transitive dependents | Fan-in x size | Touched by commits |
|---|---:|---:|---:|---:|
| worth-query-execution-layer-0 | 170149 | 0 | 0 | 276 |
| worth-query-execution-layer-1 | 167561 | 1 | 167561 | 264 |
| worth-query-execution-layer-2 | 116558 | 2 | 233116 | 202 |

## layered-5-hypothetical

The greedy order of 106 module nodes cut into 5 line-balanced crates, top first.

Hypothetical: assumes 2516 upward references are removed first.

| Crate | Production lines | Transitive dependents | Fan-in x size | Touched by commits |
|---|---:|---:|---:|---:|
| worth-query-execution-layer-0 | 106560 | 0 | 0 | 252 |
| worth-query-execution-layer-1 | 98685 | 1 | 98685 | 134 |
| worth-query-execution-layer-2 | 95093 | 2 | 190186 | 161 |
| worth-query-execution-layer-3 | 105249 | 3 | 315747 | 233 |
| worth-query-execution-layer-4 | 48681 | 4 | 194724 | 135 |

## layered-8-hypothetical

The greedy order of 106 module nodes cut into 8 line-balanced crates, top first.

Hypothetical: assumes 2708 upward references are removed first.

| Crate | Production lines | Transitive dependents | Fan-in x size | Touched by commits |
|---|---:|---:|---:|---:|
| worth-query-execution-layer-0 | 74160 | 0 | 0 | 223 |
| worth-query-execution-layer-1 | 95989 | 1 | 95989 | 164 |
| worth-query-execution-layer-2 | 83735 | 2 | 167470 | 114 |
| worth-query-execution-layer-3 | 66942 | 3 | 200826 | 139 |
| worth-query-execution-layer-4 | 60556 | 4 | 242224 | 191 |
| worth-query-execution-layer-5 | 61175 | 5 | 305875 | 138 |
| worth-query-execution-layer-6 | 11711 | 6 | 70266 | 64 |
