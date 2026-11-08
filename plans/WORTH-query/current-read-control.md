# D1347 read-only current control extraction

## Boundary review

HistoryProducerInvocation currently combines two independent truths: the exact
current product/request/principal borrowed from admission, and navigation's
revocable selection/callback seals and restored candidate. Its acquired read
locks selection then callback then occurrence commit lane then Native then World.
Its already-borrowed Native path only tries earlier seals and lane, returning Busy.
The current_head owner performs exact Native basis and World-head comparisons;
retention alone supplies its metadata effect, and the producer read carries none.
Current permission separately verifies that the query resolves the original
operation principal. SourceQueryReaderAccess retains control failures so handlers
cannot turn them into model denial. The ordinary producer already owns genuine
admission/current/principal and must not mint navigation lifetimes to read old truth.

The reusable runtime proof belongs at product_operation/current_read. Its fields
are immutable borrowed current lease, admitted request-currentness and principal
binding. It has no mutation/cutover API. One shared head owner performs exact
comparison and keeps refused-value cleanup outside guards. History wraps that
proof with its own seals and maps typed current-read causes back to existing
history refusals/subjects. No second head reader, permission grant or historical
truth selector is introduced by extraction. Native borrowed calls never wait on
an earlier lane; Busy remains typed. Foreign runtime, currentness, principal and
request mismatches continue refusing, never falling back.

## Plan

1. Move the existing head-confirmation implementation to current_read/head with
   a generic typed current-head denial. Keep history's exact refusal projections
   in its current_head facade, including its retention metadata transition.
2. Add sealed CurrentReadControl with current, admits_request, admits_principal,
   request_lapse, acquired and borrowed confirmation (and preserving internal
   handoff forms). Constructor accepts only original runtime admission types.
3. Replace HistoryProducerInvocation's three borrowed fields by that proof;
   preserve selection/callback lifetime guards, current-request deadline and
   publication cutover exclusively in navigation. Borrowed paths preserve try
   ordering and refusal ownership. Change permission principal matching to the
   reusable proof, with navigation call sites explicitly passing its read control.
4. Run unchanged document-history courts and boundary/context/caps/format, plus
   targeted borrowed-control/refusal tests already protecting these paths. Commit
   extraction separately and record EXTRACTION FOR #84. Then merge2860933b14 and
   plan/implement the actual predecessor observation at its real producer owner.

No new history capability, persistence format, permission source, fallback,
consumer policy or cost claim is part of this extraction.
