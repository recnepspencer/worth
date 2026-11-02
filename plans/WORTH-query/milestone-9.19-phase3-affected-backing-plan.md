# Phase3: reuse actual dirty-node backing while completing ancestor closure

## Boundary review

HEAD80cbb2f8ca has a qualified final SourceTree and exact writer-owned physical
changed-node map. capture::qualify takes that map from the sealed actual Nodes,
enumerates its exact keys into immutable Edits, then admits all topology and Native
Source deltas. summary_frontier::affected currently allocates a new ordered unit
map and reinserts those same live dirty keys before requalifying summaries.
ExecutionOrderedMap already shares actual immutable cells and original grants;
there is no need for a global cache, key stamp, copied Source representation or
new payer. Consume the actual same writer map after capture/admission.

Raw root-qualified callback windows select this code for investigation but prove
no accepted phase sum. The accepted fixed64 remote0/256/1024 refusal/commit
invoices are (2440583,2243133), (3497420,3207412), (3650762,3338221). The full large
qualification remains failed/open. Keep every original guard and invoice oracle.

A dirty-map membership hit does not initially certify its ancestors. Let S be
all actual dirty ordinals and S_live their live Edits. Every S_live ordinal must
separately schedule its final parent walk. A walk may stop at another seed only
because that seed's parent is scheduled by the complete exact Edit pass. Added
ancestors terminate into an already completed chain or another scheduled seed.
After that entire pass the live mask is ancestor-closed. Final topology admission
already excludes incoming live edges to retired coordinates, so retired seed
keys cannot truncate a live parent chain. Retired keys stay transient; final
postorder visits only actual final-tree nodes.

Only after the whole live Edit-parent pass, preserve the independent Native
before/after Source-change walks. With no physical writes, the empty seed must
still grow the complete Source ancestor chain. No Summary equality, unchanged
Region or unchanged Source slot may suppress qualification. Provider/domain pure
summary callbacks, Source row admission, original controls, final graph authority
and Query-owned postorder/coverage remain with their existing owners.

Capture owns the actual map and ordered admission sequence. Move the checked-
summary profile gate and affected-mask preparation there, then lend that completed
mask to summary::qualify. Remove summary's competing mask preparation and its
step/Edits parameters. No public API, new proof, storage type or SDK change.
Independent reviewer approves this boundary and the scheduled-seed closure proof.

## Implementation plan

1. In capture.rs, finish exact key enumeration before moving changed backing.
   After topology and Source-delta admission and final root assignment, gate
   CheckedSummarySourceTree once, consume the actual warm changed map into
   SourceTree::affected and pass its borrowed mask to summary qualification.
   Cold summaries still visit the whole real tree; other profiles skip summaries.
2. In summary_frontier.rs accept the actual funded map as seed, schedule the
   final parent of every live Edit, then preserve all independent Source walks.
   Explain the scheduled-seed proof at the stop boundary. Keep postorder unchanged.
   In summary.rs consume only actual tree, Nodes, Source and borrowed mask; remove
   the displaced profile/mask coordinator path.
3. Add a scoped cfg(test) leaf-only provider fault at its existing Calls receiver.
   Use actual predecessor slot-to-node lookup and actual after Source bounds;
   write correct bounds/last_bounds only on those leaves and retain ancestors.
   No copied Sources, full scan, synthetic Node id or new test door.
4. Extend the existing public summary journey with a named leaf-only twin. Real
   setup shares e1 across A(one edge) and unopened B(three edges). Move v1 from10
   to-10: B's previous min0 ancestor enclosure is false, while business contacts
   stay lawful. Require exact SummaryMismatch before Rules, unchanged World/rows,
   released proposed grants and a fresh lawful twin commit. Existing no-write30
   case independently remains. Probe negative boundary equality after retry.
5. Run the two decisive summary courts first, then fixed64 remote0/256/1024 and
   the public managed family with only the unchanged full large court excluded.
   Preserve +8/+8, authored query oracles, old snapshots, shared occurrences,
   real canceled/refused work, retained-edge duplicate and no-rebuild guards.
   Compare accepted original invoices, not raw required-window sums.
6. Run boundary/context/size/function/owned-format/whitespace checks and independent
   qa-loop, qa-tests, code-quality-qa review. Commit the coherent verified slice.
   No Query cfg(test) rebuild is needed for public runtime evidence. Revalidate
   the absent loop executable/role limitation and retain actual registered Cargo
   targets/default cache/profile. Only rerun the full50k qualification if the
   cheap evidence supports it; never relax the gate or declare a partial pass.

Large-scale qualification, predicate reverse routing, exact refinement/disclosure,
continuations and Phases4-6 remain open. This slice claims only backed dirty-map
reuse with complete summary qualification, plus any actually measured invoice
change. It supplies no new Source/read/publication or narrowing authority.

## Verified result

The new leaf-only stale-ancestor court and existing no-write court pass together
(0.68s). Their production dependency compilation took45.65s; no Query cfg(test)
build was needed. The complete public managed-index family passes46/46 in19.10s,
excluding only the unchanged full large court. Existing shared endpoint and
scope behavior, retired coordinates, Source+8/+8, hostile retained edges,
canonical/recovery/cancellation, old observations and no-rebuild assertions pass.

Accepted whole original refusal/commit invoices improve at fixed64:

| Remote edges | Before | After |
| --- | --- | --- |
| 0 | (2440583,2243133) | (2373349,2195267) |
| 256 | (3497420,3207412) | (3382068,3112454) |
| 1024 | (3650762,3338221) | (3528043,3238232) |

This is a real2.13-3.36% reduction for these cases, measured by the original
accepted SDK reports, not sums of raw callback windows. It does not establish
the unresolved large-scale guard. The modest reduction does not justify another
17-minute50k attempt yet. Keep that gate failed/open and all guards unchanged.

Boundary and agent-context checks pass against current inputs with unchanged
built verifier binaries. All6 changed/new Rust files pass the400-line cap;
function review has6 coherent advisories and zero scan errors. Owned formatting
and whitespace pass. The loop CLI/alias/role entry remains absent, so the actual
registered public Cargo target and pinned default cache/profile were used.

The original writer dirty map is consumed after exact enumeration and admitted
final topology; the displaced new mask allocation/reinsertion path is removed.
Every live seed separately schedules its parent before Native Source walks, so
neither a seed membership hit nor equal summary/region can omit ancestor checks.
No public API, Source/read/narrowing authority, generic cache or SDK storage
surface was added. Predicate routing, other Phase3 work and Phases4-6 remain open.
