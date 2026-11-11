# D13 qualifier boundary: private storage courts (not a product workaround)

Source: eb875cdf04 plus d13-source-head-work-packet.json. Evidence:
`d13-head-work-root.log`, Root21/4/0. Independent boundary review rejects
extending CountedSource or its local adapters. No such extension was made.

## Actual refusal

`region_load/materialization_funding/history_custody.rs::capture` correctly calls
`RelationalDurableMemberCustody::capture` under the original request. The
CountedSource fixture implements only legacy `retain_members`; the funded trait
method refuses UnsupportedProvider, projected as SourceUnavailable. Cancellation
and damage guards must stay. External immutable_members already implements the
actual funded custody door; the private fixture must not counterfeit that proof.

## Exact claims and closest governed observations

| Exact test | Private claim not currently observable externally | Closest real public flow |
|---|---|---|
| head_prefixes_rebuild_owned_history_after_endpoint_edits_and_generation_reuse | Resetting physical lifecycle to Reusable, rewriting the same slot with a new generation while retaining old metadata; exact partial/full canonical_history and kind commitments; exact one history read. | External peer_history court reads historical/current outgoing neighbors after checkpoint/reopen. Public retention/maintenance_slot_reuse proves actual reclamation and a higher generation, but can retire precisely the history that the private unit retains. |
| adopting_a_different_head_carries_its_owned_historical_projection_with_the_record | merge_slots_from_owned takes a Head prefix from a second physical source Arc while other prefixes retain their first issuer; later Full promotion reads both original issuers and equals the full oracle digest. | Public managed_source/cold_merges use authentic fork/merge and public Complete capture can demand Full. The inspected cold_merges recipe has one recovered MemberSource; branch identity does not prove distinct provider identity. |
| cancellation_during_readmission_preserves_the_request_and_does_not_damage_the_cell | exact private cell.leases == 0 after cancellation following member read; same cell retries; less than four physical reads. | Existing external SelectedSource/recover/read_head, actual locator reads, exact Request(Cancelled), request-memory refund, successful fresh-request retry and a delimited read bound. Its Cancel control currently fires only in read_member_owned; Head uses read_member, so cancellation must be extended at that actual external seam. Memory refund is not claimed to be the private lease counter. |

Source searches: `rg -n 'pub fn .*residen|pub fn .*lease|active_.*lease|leased_regions'`
under Native runtime/, facade/ and storage/residency/ found no public residency
lease-count observation. The owner review inspected actual public managed_source,
cold_merges, peer_history, maintenance_slot_reuse and immutable_members paths.

## Honest migration boundary

Source-free private units can protect exact projected rows, slot sets, current
and retired generation data, endpoint edits, kind commitment equality and
selected-slot value adoption. They must not construct a runtime, request,
provider or authenticated history opening. Companion external courts must prove
the real checkpoint/read/merge/cancellation journeys.

The physically distinct original-provider adoption claim and exact private
lease counter remain unproved by those inspected public flows. They cannot be
quietly dropped or replaced with approximate public claims. No new observer door,
local provider bridge, baseline expansion or test deletion is authorized here.
An owner decision or a genuine existing public journey is needed for that
irreducible remainder before these three named courts can be declared migrated.

The fourth Root failure is separate and repairable: the old node test quote uses
Arc<T> layout, omitting StorageAllocation's own memory-ticket backing. Actual
2496 minus expected2208 =6 nodes x48 bytes. The prepared correction retains exact
equality and every alias/drop assertion and uses the actual owner layout.
