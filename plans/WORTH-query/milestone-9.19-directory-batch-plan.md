# 9.19: seal the touched directory frontier once

## Boundary review

Audit item5 is real in nodes/pages/edit.rs: one PageEdit owns the actual predecessor,
original ExecutionRequest and SDK ordered touched-page map. replace copies each
initialized page once and preserves tombstones. seal currently consumes builders
one at a time and calls NodePages::install per page. pages.rs::install copies each
binary directory path again; neighboring pages therefore repeat common ancestors.
The retained directory is descriptive physical storage, with no Source, identity,
read, completion or publication authority. Genuine IDs and monotonically issued
coordinates remain in Nodes/Writer; actual Native/Query receivers still qualify
Source, summaries, completion and publication.

Directory leaves own move-only paid SDK arrays through private Arc headers.
Untouched branches/pages must share their original paid cells. Staged and sibling
edits are isolated; original request refusal must publish no partial predecessor
change. The SDK touched map is already ordered and uses a bounded inline iterator.
No second map, owned replacement Vec, allocator or authority registry is needed.
Sparse huge ordinals currently increase binary height; batching does not repair
that separate churn obligation. The maximum page key is ordinal>>5, so directory
height and all partition shifts fit usize.

## Implementation plan (saved before Rust edits)

1. Replace per-page install with one ordered, consuming traversal of the union of
   touched directory paths. A private edit/seal module owns this single semantic
   operation. Use the existing SDK ordered iterator with one peek; seal each
   WorkingPage at its leaf and construct each nonempty affected ancestor once,
   after both children. No intermediate root installation.
2. Traverse virtual missing growth levels without first allocating wrappers that
   would be copied again. Carry the actual old subtree height; untouched actual
   subtrees share their Arc. An untouched virtual left spine materializes only
   the required final wrappers. Empty leaves/branches prune, including insertion
   then retirement beyond the predecessor range. Empty edits preserve the base.
3. Keep every original control checkpoint, paid builder seal, retained Arc/header
   reservation and final poll. The entire consuming sealer returns one new root
   only after success; failure drops partial output and staged pages. Remove both
   displaced single-page install methods so one ordinary path remains.
4. Focused numeric owner evidence in existing edit/tests.rs, with exact placement
   approved by the independent reviewer: working_page_batch_seals_shared_directory_ancestors_once
   builds real dense/sparse predecessor pages, prepares multiple pages, verifies
   union-of-path ancestor counts, untouched Arc identity, tombstones, empty growth
   and old-root/sibling isolation. A receiver-owned cfg(test) passive allocation
   counter increments only at actual successful branch allocations, allowing
   detection of repeated temporary ancestor allocations; no global mutable hook.
   Whole original SDK seal invoices64/128 are reported separately.
5. working_page_batch_refusal_preserves_predecessor_and_releases_partial_seal uses
   original SDK work ceiling during multi-page seal after preparation, verifies
   typed refusal, no residual staged grants after drop, and funded same-data retry.
   Existing stopped writer, retained snapshot and genuine sibling-ID tests stay.
   No Source/qualified-root factory or semantic expectation from numeric fixtures.
6. One focused private owner build for nodes::pages, then affected managed49/49
   (exclude separately accepted expensive geometry/prefix growth), Candidates9/9,
   mandatory boundary/context, fmt/whitespace, dirty caps/functions and independent
   qa-loop/qa-tests/code-quality-qa gate before a dedicated commit. Default
   Rust1.99.1/sccache/shared target only; no aborted builds or broad runtime rerun.

The intended result is O(U) new affected directory cells for the union U of the
touched page paths, plus one sealed leaf per touched nonempty page. Iterator and
page processing remain paid. No global locality, logarithmic historical-coordinate
or universal spatial-selectivity claim. Churn, spatial grouping and unfinished
Phase3/4/5/6 work remain open. No public API/DX change.

### Empty growth boundary refinement

Before building, determine final height with one paid touched-page scan, considering
only nonempty replacement pages plus the predecessor height. Empty touched pages
beyond that range can be dropped with their unsealed builders and are excluded
from the ordered sealer. Otherwise a far insertion-then-retirement would allocate
needless virtual wrappers or force later collapse. This is O(P) bounded work over
the actual touched set, no full predecessor scan or second allocation. Existing
empty-page removals inside the predecessor range still traverse and prune normally.

## Focused owner result

The single default Rust1.99.1/sccache/shared-target nodes::pages build passes6/6
in0.02s. Dense64/128 touched pages of the same256-page actual predecessor allocate
65/128 branch cells, matching distinct required dyadic ancestors. Whole original
SDK seal invoices are324/641; preparation and predecessor construction are outside
those seal invoices. No fee was repriced. Adjacent512/513 sparse growth allocates11
branches including the untouched old-root embedding cell. Far insertion then
retirement allocates0 branches and shares the exact original root/height.

Complete page tombstones, old/sibling values and untouched page Arc identity pass.
An original work ceiling at half the actual funded same-data seal report stops
after at least one successful actual branch allocation, yields typed WorkCeiling,
releases every original staging/output grant and preserves predecessor/sibling.
The same input succeeds under a fresh funded request. The actual passive allocation
counter is per receiver and compiles only in owner tests; it grants no control.
Affected managed49/49 passes18.18s, excluding the separately accepted large
geometry and prefix cases; Candidates9/9 passes0.57s. Required cargo boundary and
context checks, rustfmt/whitespace, dirty caps4 files and function scrutiny3
advisories/0 scan errors pass. The existing large-world invoices were not rerun
for this physical allocation repair; the public small geometry locality and real
prefix guards are included in the49 passing cases. Independent qa-loop, qa-tests and code-quality-qa review clears this dedicated
slice without an outstanding scoped blocker. Churn and other milestone work remain open.
