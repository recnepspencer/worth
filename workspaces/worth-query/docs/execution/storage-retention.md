# Reclaim retired storage

Use `WorthQueryPrimaryGraphApplicationRuntime::try_reclaim_retired_storage`
inside an advancement borrowed from that same installed application. The pass
uses its installed retention backend, automatic-reclaim setting and batch size,
plus the original request's memory, work, cancellation and deadline controls.
It creates no logical Product commit and grants no record-selection authority.
When the effective original context has accounting, scan buffers and mutation
peaks use that same loan, including limits inherited from an accounted parent.
An absent ceiling within an accounted policy keeps accounting enabled. A
genuinely unaccounted context uses the original native execution controls
without manufacturing accounting or a budget.

Save the pass result outside the callback. A request stop also refuses the
advancement's final handoff, while reclamation already performed remains done.
This example returns both observations so the host can report partial progress.

```rust
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_contribution::WorthQueryAdvancementDenial,
    domain::ApplicationSchema,
    primary_graph::{
        WorthQueryPrimaryGraphApplicationRuntime, WorthQueryStorageRetentionDenial,
        WorthQueryStorageRetentionProgress,
    },
};

type Pass = Result<WorthQueryStorageRetentionProgress, WorthQueryStorageRetentionDenial>;

fn reclaim<S: ApplicationSchema + 'static>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    request: &WorthQueryRequestScope,
) -> (Result<(), WorthQueryAdvancementDenial>, Option<Pass>) {
    let mut pass = None;
    let advancement = runtime.with_application_advancement(request, |phase| {
        pass = Some(runtime.try_reclaim_retired_storage(&phase));
    });
    (advancement, pass)
}
```

Success reports actual scanned chunks, slots, lifecycle refreshes and reclaimed
records. A denial carries the completed prefix and its typed stop. Counts do
not promise an amount of CPU work or bytes. A disabled automatic-reclaim policy
can refresh retention state while reclaiming no records.

`ForeignPhase` refuses before storage contact. `StorageObserved` means a live
reader holds the authoritative storage edition; release that reader normally
before retrying. Historical Product observations own independent retained roots
and keep their existing visibility contract across physical slot reuse.

For execution refusal, retain the completed prefix and retry under a fresh
lawful request when appropriate. In an accounted context, each scan buffer and
mutation peak is admitted before allocation against the original request, then
released. Adopted storage continues under its existing storage owner. No cleanup
handle escapes this pass.
