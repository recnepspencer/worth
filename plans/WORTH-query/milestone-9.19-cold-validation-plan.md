# One cold DFS over retained Source parent links

## Boundary review

At7e8a0195c0, capture validates every described child against actual candidate
NodeIds; transition retains independent Region/parent columns, unique parents and
Source-slot bijection. Admission checks exact live counts, N-1 incoming postings,
root existence/parentlessness and incoming edges on retired nodes. It then walks
each new edge's parent ancestry. That is appropriate incremental induction for
warm edits of an already qualified immutable predecessor, but repeats cold
ancestries quadratically on skewed graphs.

Cold capture supplies a complete enumeration of actual live Edit ordinals. After
the same shape checks, one private functional-parent DFS can follow white nodes,
mark them gray, refuse a gray hit, stop at black/None, and mark that newly visited
path black on a second walk. Stop the second walk at the first non-gray node.
Earlier successful paths are entirely black, so gray belongs only to the current
walk. Each actual parent lookup is performed at most twice per live node. Visit
all supplied seeds to detect disconnected cycles as well. No visitation survives
this exact immutable capture; interruption drops the whole local map.

Use a sparse SDK ExecutionOrderedMap<usize,Color> and existing paid ordered
storage operations. This costs O(N log N) bookkeeping plus the real parent-column
access cost, not repeated O(N^2) parent walks or an H-sized bitmap. Do not claim
literal linear total SDK time or bounded cost independent of directory height.
The helper returns traversal success/refusal only. It creates no Source/tree/root
proof, row projection, identity or public read authority. Actual Step::Cold vs
Step::Warm dispatch in capture selects the path; warm new-edge induction stays
unchanged, including its existing ordering of retired-edge checks.

The independent reviewer approves a private numeric-owner test for this pure
traversal. Literal integer parent graphs, complete seed lists and measured parent
callback executions are descriptive algorithm input, not fake Source/Native
identity. Actual SourceTree/SourceSlot/native IDs will not be manufactured.
Existing real public structure/duplicate/cycle/Ready/head/lawful-twin courts cover
the real adapter, census, Source bijection and publication boundary. One targeted
private cfg(test) build is justified; enlarging the public Source fixture solely
for this algorithm would absorb unrelated prefix source/observer limits.

## Implementation plan

1. Factor the existing common live/source/parent/root shape census in admission.
   Preserve the warm admission's exact new-edge walk and retired-incoming check.
   Add a cold method that performs that census/retired check then calls one
   private cold.rs numeric DFS over actual live Edit keys and self.parent.
2. Select cold versus warm using the existing actual Step enum in capture. Leave
   transition, Source changes, summary closure and receiver authority unchanged.
3. Add exact private cold/tests.rs owner courts: empty/root,64/128 sparse-labelled
   chain, self-cycle and disconnected cycle. Count actual supplied parent callback
   executions with a predeclared<=2N success bound. Separately measure accepted
   original SDK work for a generous<3x doubling guard on paid map bookkeeping;
   that number is not reconstructed from callback counts.
4. Use actual SDK controls/owned map grants for finite-memory refusal and original
   cancellation. Assert scratch ownership releases and fresh funded uncancelled
   retry succeeds on the same immutable numeric graph. No state persists/caches.
5. Register only the exact private file/tests/actual SDK factory roots with the
   independently approved placement rationale. Run ONE scoped Query owner test
   build using normal Rust1.99.1/default sccache/target. Existing genuine public
   source structure and managed families verify Step/census integration; avoid
   repeating unchanged large50k qualification absent a real concern.
6. Run mandatory boundary/context, dirty size/function/fmt/whitespace checks,
   independent qa-loop/qa-tests/code-quality-qa gate, truthful milestone evidence,
   then commit. Prefix reuse, batched directories, spatial grouping, churn and
   later milestone requirements remain open.

## First owner build and test refinement

The single targeted private-owner build completes in4m30s; numeric chain/cycle
courts pass, accepted original SDK work64=959,490 and128=2,203,200. The1-byte
memory attempt refuses SDK root admission (392requested/1admitted) before DFS,
so it is NOT scratch-release evidence. The test now uses a predeclared4KiB budget
on128nodes and REQUIRES nonzero partial parent reads and an exact original SDK
MemoryExhausted refusal before counting cleanup/retry evidence. Actual
ExecutionWorkCeiling::run_with_checkpoint returns the callback's typed result and
report whenever entry ran, including cancellation; only root-entry refusal lacks
that tuple. The harness makes root-entry expectation explicit, keeps real reports
and removes redundant repeat64/128 calls. No denied work is synthesized as an
accepted invoice. Final rerun and public integration remain pending.

The final private-case strength pass also requires1..=2 actual parent callback
executions for EVERY supplied chain node, not merely an upper bound that an
early-return implementation could satisfy. The cycle family includes a rooted
64-node disconnected cycle in addition to self/two-node cycles, so a shallow
traversal cutoff cannot pass. These remain the same three approved numeric-owner
tests; no Source fixture, private factory root or authority surface is added.

Final reviewer correction keeps a fresh64-node retry immediately after the
64-node cancellation, and a fresh128-node retry after the128-node finite-memory
refusal. Each retries the SAME immutable numeric labels/parent function with a
fresh original request, and each original scratch grant must release. No smaller
substitute graph counts as retry evidence. The exact test/root placement is
unchanged; production/public integration code is unchanged by this correction.

## Final measured verification

The final exact owner target passes3/3 in0.01s. Empty/root and64/128 sparse-labelled
chains require every supplied node's actual parent lookup1..=2 times and total
reads<=2N. Self/two-node and rooted disconnected64-node cycles refuse. Accepted
original SDK bookkeeping work is959,490/2,203,200 under the unchanged<3x guard;
callback counts are not substituted for that invoice. The4KiB128-node case stops
after9 actual parent reads with original typed MemoryExhausted policy ancestor0
(requested120, admitted32), releases all actual original scratch and succeeds on
a fresh funded128-node retry. Cancellation at read8 independently releases and
retries its same64-node graph. The initial1-byte root-entry failure is not counted
as scratch evidence. One private-owner target was compiled(initial4m30s), with
incremental reruns only for concrete harness/reviewer evidence corrections; no
broader private crate test run, feature/profile/target/compiler override or build
restart was used.

Unchanged production after that owner test refinement passes the real public
managed_index family47/47 in18.65s and genuine Candidates profile9/9 in0.60s.
These include exact omitted/duplicate/disconnected Source topology, Failed/Ready,
unchanged selected head and lawful twin, checked summaries and actual warm edits.
The unchanged large50k qualification is not repeated or relabeled as cold evidence.
Boundary/context, dirty Rust400-line caps(5files), function scrutiny(5advisories,
zero scan errors), formatting and whitespace checks pass. Both mandatory Cargo
commands, compiled-document drift1/1 and independent unconditional final
qa-loop/qa-tests/code-quality-qa clearance pass on the final scoped files.

This repairs repeated cold ancestry walks. It does not eliminate capture-frontier
cost, shared census/retired checks, ordered-state bookkeeping or page depth, and
it does not close the separate churn axis or remaining Phase3 requirements.
