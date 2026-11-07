# D13 cold Head work accounting correction

## Evidence and invariant

The unchanged fixed/discovered terminal restore courts have about 4.395M of their
8M original work allocation after Native admission. Cold Head loading then charges
3,612,650 units from memory bounds (wire 1,518,252; native 2,094,398) for a
25,877-byte source member. The allocation bounds include spare capacity,
relocation coexistence, and largest-catalog-contract bounds. Reconstruction owners
subsequently charge work again. This is a work invoice defect, not permission to
relax the request or bypass authenticated Head admission.

## Owner changes

1. `storage/residency/region_load.rs`: retain the complete allocation preflight and
   all reservations. Replace the wire+native capacity work invoice with a separate
   payload-length charge before Serde decoding. Existing source read/hash,
   preflight traversal, schema, kind, symbol, content and history-opening work
   remain. No Full/Head/root selection or validation changes.
2. `storage/substrate/shared_column/bulk_bootstrap.rs`: at the dense-construction
   owner, precharge actual page initialization (32 Option<T> cells plus the node),
   actual incoming carrier moves, level-vector pointer writes, and actual branch
   initialization/moves. Reuse the original request and preserve typed stops.
   Charge before construction, without changing the general node allocator or
   ordinary COW mutation accounting. Empty construction has no backing/work bill.
3. `durability/checkpoints/aspect_state_images/readmission_funding.rs`: keep the
   existing retained and transient memory bounds unchanged. Replace charging their
   sum as work with a distinct work preflight for actual entry/value validation,
   entry moves and key/contract cloning. Resolve the actual matching contract by
   borrowed lookup; preserve Foundation's exact-then-key fallback and all its
   typed semantic validation. Two contract clones really execute (catalog lookup
   and validated value construction); do not charge the catalog maximum as work.
   Fund struct field comparisons and variable scalar scans before validation.
4. `durability/checkpoints/images/arena_images/readmission.rs`: keep diagnostic
   backing bounds/reservations, but charge only row visits and moving the actual
   BTreeMap into its StorageAllocation. The already-decoded map and String backing
   are transferred, not reinitialized at their capacity sizes. Existing scratch
   vector charges remain actual initialized/moved element-layout work.

All changes stay in granted storage/ and durability/checkpoints/ trees, outside
Lifecycle's exclusions. No new authority, observer, budget, wire format or caller
surface. Memory lifetimes and conservative memory preflight are unchanged.

## Bound and proofs

Work is payload decode plus actual materialized rows, initialized pages, carrier
moves, matching contract clones and validation comparisons, plus existing ordered
index and commitment work. It has no charge proportional to spare allocation
capacity or unrelated largest contracts. Memory bounds retain their old safety
margin. Explicit reconstruction remains request-funded and tiny/cancelled work
requests still refuse before the affected operations.

Run the two failed terminal courts unchanged, the pending 19-name Query packet,
the Store selected recovery court, and the existing Native column custody and
cold materialization controls reached by these changes. Reuse green evidence for
unchanged Native/World authentication controls. Fresh implementation review must
verify every removed work invoice has its actual executing work covered.

## Review correction: complete owner coverage

The first design review REJECTED two omissions, both accepted:
- Kind hash validation and projection grouping were cancellation-only, and the
  request projection driver discarded its work count. Add a request-aware Head
  validation leaf: stream the existing kind hash with charged writer callbacks;
  compare bucket memberships directly through canonical borrowed iterators,
  preserving exact row/group/order equality while eliminating temporary Vecs;
  charge grouping per claim and ordered navigation. The projection request
  driver must charge its existing logical-work argument, never the accompanying
  memory bound. Fund symbol-closure traversal at its existing owner too.
- Charge fixed Extra construction from owner_bytes even for absent aspect state,
  and fixed metadata construction at the generic arena conversion before its
  constructor. Pointer-carrier column work does not cover those pointees.

The actual matching-contract work preflight uses borrowed exact-then-reverse-key
lookup and pays its own comparisons before reading keys. It pays two successful
contract clones, two canonical-map key clones, entry moves through the known
vectors/map/boxed slice, ordered comparison bounds, required-field binary search,
value-field linear declaration search and canonical scalar scans. Retained and
scratch capacity bounds remain memory-only. No Foundation behavior changes.

Budget exception: existing repeated rounds isolated authority/readmission bugs;
the passive two-name runs now establish this separate accounting defect. One
coherent owner correction, then a batched focused verification, replaces further
probe builds. Current cumulative counts before that batch:8 check/Clippy calls,
10 prebuild invocations,8 selected test invocations;1 selection-only refusal.

Final design clarifications: the work preflight includes the catalog exact-query
temporary basis-key clone, including lookup misses, and a denial-key clone bound,
as well as both canonical-map key clones. Installation's final current-versus-
indexed set equality keeps its exact semantics and pays source visits, actual
member insertions/navigation and comparison through its existing callback.
The capacity/work separation claim applies to the three replaced invoices;
existing history-opening accounting is unchanged and not claimed rewritten.
