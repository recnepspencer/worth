# Independent cold Full funding design review

Reviewer: `/root/cold_region_funding_design_review`.
Scope: `cold-region-funding-design.md`, coordinator Ruling 1, M3 D12, the six
explicitly granted residency files, `cell/demand.rs`, `cell_load_cache.rs`,
`history_capture_bound.rs`, `history_member_directory.rs`, existing concrete
allocation quotes and materialization-quote analogues. Read-only inspection;
no builds, tests or source edits. This note is the sole reviewer write.

1. **P1 — The reservation-lifetime contract is incomplete at the actual owner
   transitions.** The design says scratch releases with HistoryFolder while
   only final backing charge transfers. But `HistoryFolder::read` returns an
   `Arc<RegionHistoryImage>`, and `region_load::load_full` retains that Arc after
   explicitly dropping the folder; it uses the image again for historical kind
   reconstruction and bucket verification. Folded result Vecs also escape the
   folder. A reservation stored only on the folder would refund their storage
   while those values remain alive. In addition, a Cold Full demand first
   calls `load_head`, publishes/caches the Head, and only then calls load_full
   (`cell/demand.rs`). The raw Head cache remains a real owner. Funding only
   the final Full partition would leave that prerequisite allocation and its
   retained cache unaccounted.

   Required design correction: identify the funding owner for every escaping
   decoded image and folded row result, and the exact transfer/drop boundary.
   For example, return an owner carrying both decoded DATA and its ticket, or
   retain an enclosing admitted funding owner until all temporary owners die;
   do not release scratch merely because traversal ended. Explicitly fund
   load_head's decode/rebuild/retained cache as part of the ordinary owner path,
   before allocation, and keep its ticket with that Head ResidentRegion. Keep
   the final Full ticket with its ResidentRegion, and fund simultaneous Head,
   decoded cache, folded wire rows, reconstructed native rows and verification
   scratch. The implementation may choose the smallest concrete type shape;
   the design must settle those actual lifetimes before coding. Perf allocation
   lifecycle/peak-vs-retained laws, arch 9/16, coordinator Ruling 1.

2. **P1 — The stated O(S+T) argument does not yet establish the claimed work
   bound.** One decode per member does not imply one traversal per member.
   `fold_row` follows a prefix chain again for each row; cache hits still do
   ordered-map lookup and directory checks. `CachedHistory.directories` is a
   linearly scanned Vec, and a new directory identity re-runs
   `verify_directory`, which rebuilds/sorts direct bases. `fold_rows` also
   traverses each row. The design invokes existing bounded restart layers, but
   `RelationalHistoryMemberDirectory::readmit` proves acyclicity, not a depth
   ceiling: its own value court admits a 16,384-deep graph. The producer's
   eight-layer consolidation rule is not a universal loader admission rule.

   Required design correction: state the bound using separately named
   physical-input bytes, output state/row occurrences, row-prefix hops and
   ordered cache/directory navigation; show which are bounded by S+T for a
   legally produced selected Home. Name the exact provenance/producer invariant
   that bounds depth there. General Full materialization must still charge all
   row hops and directory checks, even for a deeper admitted directory; do not
   silently reject such a graph or claim producer policy is codec validation.
   If repeated empty-prefix/directory work is not bounded by the selected-Home
   provenance, eliminate or memoize it in the owner rather than concealing it
   in a serialized-length charge. The existing discarded-canonical-history
   proof remains valid for its own axis, but it cannot establish this separate
   region-layer/row-alias axis. Perf locality/amplification/proof laws and D12.

The core architecture is sound: fund both ordinary Full loaders at the owner;
keep request failures retryable; use the exact counted same-cell Full DATA
lease; preserve original Full and Head serving rules; keep one prefix-free
emitter and independently funded output. No covered canonical replay or
maintenance-only accounting route is proposed. The existing cache correctly
avoids storing request stops as damage, so the design should preserve that
behavior.

The four proposed private files are a reasonable and necessary responsibility
split for concrete preflight and its direct funding controls. `region_load.rs`
and `history_fold.rs` already have 340 and 307 physical lines; existing index
and canonical decode quote families use dedicated owner-specific leaves.
`region_image.rs` here means the proposed quote child, not permission to edit
`storage/residency/region_image.rs`. This review grants no scope: the four new
files remain pending the coordinator's explicit handoff. No public API,
facade, format or broad boundary pattern is needed for their proposed private
placement. Additional concrete quote families, if needed to meet caps, require
the exact additional grant before edits as the user specified.

REJECT
