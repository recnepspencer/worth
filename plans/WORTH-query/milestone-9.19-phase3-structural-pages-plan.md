# Phase 3: page actual ordinal structural metadata within each isolated edit

## Boundary review

HEAD 6cae3d9cf1 has exact checked Source topology and summary closure. Its
SourceTree retains three ordered maps: node ordinal to Region, child ordinal to
incoming parent, and genuine Source slot to node ordinal. Only the third needs
Source-key comparison. Repeated structural writes to nearby ordinals copy
ordered-map paths for both first columns. The existing nodes/pages owner already
has sparse immutable NodePages<T> and original-request PageEdit<T>: first touch
copies an actual 32-cell page once, staged tombstones override its predecessor,
seal consumes the same initialized allocation and installs once per page.

Reuse that concrete physical ordinal backing for TWO independent columns:
NodePages<Region> and NodePages<usize parent>. Do not merge them into one record:
removing a region must never erase a retained incoming parent. The explicit
retired-ordinal check must read the actual sealed parent column, independently
of live counts. Source slot correspondence remains the existing SDK ordered map.
The actual SourceTree/root/session and coverage frames continue to bind meaning;
physical page identity is descriptive, not Source/read/proof/publication authority.

SourceTree owns checked live region and incoming-parent counts. Update from
actual staged cell occupancy before each write, preserving Some-to-Some count;
compare against independent actual Nodes.len, Sources.len and root presence.
Counts alone do not prove connectivity, retirement or Source correspondence.
Keep all duplicate Source/parent, retired incoming, root, cycle, Native delta,
summary and executable coverage checks at their actual owners.

A consuming private topology transition opens one PageEdit per column on that
actual predecessor and original request. Remove all lost parent/Source/region
postings before adding all new unique ones, preserving staged None precedence.
Seal both columns before constructing the replacement. capture retains final
root/cardinality/retired/cycle/Native-delta/summary admission before returning the
actual qualified SourceTree/root pair. Refusal during either column leaves no
published mixed pair; ordinary immutable Arc custody preserves old snapshots.

Widen only the existing page module/types/methods to managed_index, keeping their
physical owner and private tests in place. Add empty Default backing with no
request/grant. No generic cache framework, SDK change, new public API or phase
marker. All converted capture, root, retirement, cycle, region, parent and summary
lookups must use the same paid page receivers, with one actual point-access
checkpoint plus paid directory traversal. Remove old ordered-key comparison
quotes for columns no longer represented as ordered maps; do not reprice the
remaining Source and transient unit maps or alter qualification guards.

Page memory has sparse-occupancy overhead and directory paths depend on actual
ordinal frontier, including nonrecycled history. Do not claim a bound in live
count alone or predict an improvement. First measure unchanged small geometry
axes and existing retirement/snapshot/hostile cases. Earlier original invoices
remain the oracle; raw callback windows are only diagnostic required-work data.

Independent reviewer approves this ownership/cutover, including preservation of
the explicit sealed retired-incoming check. Existing public structural adversaries,
retirement/old-snapshot cases, original-request cancellation/memory courts,
PageEdit physical custody evidence and the 46-family are proportionate; no new
synthetic/private fixture or Query cfg(test) rebuild is required just for this
representation change.

## Implementation plan

1. Make existing nodes/pages reachable only within managed_index and expose the
   minimum NodePages/PageEdit operations there. Implement zero-owner empty Default.
   Retain immutable arrays, original tickets, consuming seal and old private tests.
2. SourceTree stores the two concrete page columns plus actual occupancy counts,
   retaining the Source slot map. Its region/parent receivers perform actual paid
   ordinal reads. Capture obtains predecessor Regions through that same receiver.
3. Replace remove_edits/add_edits with one consuming apply_edits transition.
   Open both page editors, carry counts, remove all lost associations, add all
   unique associations and changed Regions, then seal both. A local mechanical
   write helper derives checked count changes from actual staged occupancy.
   Preserve explicit duplicate-child rejection even for retained edges.
4. Adapt admission to independent counts, actual root Region/parent and the exact
   sealed retired incoming-parent column; cycle walks use the ordinary parent
   receiver. Keep Native Source delta checks, affected dirty-map reuse/closure,
   Source reads and summary postorder unchanged. Remove all displaced ordinal
   ordered-map accesses/quotes; no competing reader or fallback remains.
5. Run decisive public Source structure and summary courts, then unchanged
   fixed64 remote0/256/1024 and all managed-index courts excluding the original
   large qualification. Preserve +8/+8, authored scan oracles, shared endpoint
   occurrences, original memory/cancellation, retired/sibling snapshots and no
   rebuild evidence. Use the registered Cargo target, pinned default compiler/
   cache/profile; the loop executable remains an explicit absent infrastructure.
6. Run current-input boundary/context, size/function, owned formatting and
   whitespace gates plus independent qa-loop, qa-tests and code-quality-qa review.
   Compare accepted original invoices and public correctness evidence before
   deciding whether a full unchanged 50k run is justified. Commit after clearance.

No Phase 3 or whole-milestone completion is claimed. The full large-scale guard,
retained predicate routing, exact refinement/disclosure/continuations and
Phases 4-6 remain open. There is no guard, quote or acceptance relaxation.

## Verified mechanics and running full qualification

The two decisive summary courts pass in 0.66s; production dependency compilation
through the actual consumer took 1m16s. The full public managed family passes
46/46 in 19.05s, excluding only the unchanged full large qualification. Existing
cold/warm malformed structure, duplicate retained edges, retirement/snapshots,
Source +8/+8, no-write/leaf-only summary, memory/cancellation, shared occurrences,
authored query oracles and no-rebuild assertions all pass.

| Fixed64 remote edges | Previous accepted refusal/commit | Paged accepted refusal/commit |
| --- | --- | --- |
| 0 | (2373349,2195267) | (1905451,1870404) |
| 256 | (3382068,3112454) | (2356630,2303250) |
| 1024 | (3528043,3238232) | (2419432,2387569) |

These are actual whole original SDK reports. Query invoices also decrease with
the new physical point-read paths. No raw callback intervals are summed into
accepted costs. Source, transient-unit-map and existing page-operation prices
are unchanged; the displaced ordinal AVL comparison/path-copy operations no
longer run. No qualification guard is changed.

Current-input boundary/context validation passes using unchanged built tools.
All 8 changed Rust files meet the 400-line cap; function review reports 9 coherent
advisories and zero scan errors. Owned formatting and whitespace pass. Independent
qa-loop, qa-tests and code-quality-qa review clears the mechanics without requiring
a new private owner build. Public runtime evidence covers the new columns;
existing page-owner custody mechanics are reused unchanged.

The accepted small-axis reduction justified one rerun of the unchanged full
1k/2k/50k qualification under the existing registered public target and pinned
default compiler/cache/profile. It passes 1/1 in 729.27s; authoritative log:
query919-structural-pages-large.log. No production code changed during the run.

| Population | Accepted refusal | Accepted commit |
| --- | ---: | ---: |
| 1,000 | 2,168,584 | 2,103,448 |
| 2,000 | 2,303,532 | 2,235,605 |
| 2,000 + 50,000 remote | 3,367,668 | 3,283,350 |

Both original 25% local-growth guards pass. Remote 50% limits are
3,455,298 / 3,353,407; the tighter eight-height-step limits are
3,383,116 / 3,292,861. Both final remote invoices pass both guards without
changing growth.rs. Source census and exact +8/+8, original-rule refusal and
lawful retry, retained rows and no-rebuild checks execute successfully. Actual
52,000-Source post-edit probes use 33/33/35/35/31/1 summary comparisons against
maximum 67; future-point membership changes from one to two authored matches.

Independent qa-loop, qa-tests and code-quality-qa review gives unconditional
clearance for this mechanics slice and its checked large-geometry qualification.
Earlier failed runs remain historical evidence, not current failures after this
repair. This qualifies the actual indirect geometry Source/provider composition,
not prefix-specific large workloads, retained predicate routing, tracked exact
refinement, protected disclosure/continuations, narrowed authority or Phases 4-6.
