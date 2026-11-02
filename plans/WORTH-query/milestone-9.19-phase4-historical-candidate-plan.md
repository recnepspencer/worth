# Query9.19 Phase4: Native historical weak material foundation

Read the [boundary review](milestone-9.19-phase4-historical-candidate-boundary-review.md)
first. The initial boundary review/plan preceded Rust. Lifecycle review then
rejected strong canonical payload ownership before further implementation; this
final plan describes the repaired Native prerequisite. Query filtering remains
a follow-on. The earlier conservative integration is committed as56bfbd8639.

## Result and limits

An actual Native Source observation can obtain sealed historical metadata from
an actual canonical authored occurrence:

```rust,ignore
let impact = observed.authored_history_impact(runtime, execution, &occurrence)?;
// Leave Native/runtime/attempt guards before acquiring opaque data.
if let RelationalAuthoredManagedSourceImpact::Retained(metadata) = impact {
    let Some(payloads) = metadata.acquire_payloads(execution)? else {
        // Expired actual data means conservative selection, not empty Source.
        return conservative();
    };
    inspect_owned_payloads(&payloads);
    drop(payloads); // Same outside-guard boundary, including failure paths.
}
```

The view retains actual program/full-generation scope, closed affected rows and
funded weak receipts. It admits no READ, narrowing proof or performed correction.
Both actual prepared sides must be present/available; every missing side stays
Conservative. Wrong runtime/catalog/branch/cell correspondence is a typed refusal.
Only the actual original controls pay new lookup/work and metadata memory.

Query's existing correction remains conservative. No irrelevant-edit success,
new governed/box/refined correction, field tracing or Phase4 closure is claimed
by this Native prerequisite.

## Module responsibilities and implementation

1. Existing managed_source/history owner retains only the actual cell affinity
   metadata; no interpretation, physical basis, branch root or whole image is
   stored in canonical history. Its receiver creates selected weak receipts while
   borrowing under the actual directory guard, without a transient strong payload.
   Existing exact program/key validation remains one shared lookup authority.
2. candidate_source/opaque_custody owns shared internal funded opaque Arc/Weak
   mechanics. Existing public context-share/receipt wrappers keep their API.
   Original constructor refusal returns T. Retaining an existing borrowed Arc
   quotes only its exact dynamic inline allocation and the sealed funding Arc:
   Layout([usize;2]).extend(Layout::for_value(payload)).pad_to_align(). Admit before
   Weak creation. Inner heaps stay with strong owners; receipts/shares keep sealed
   funding through the last weak allocation. History has separate public names.
3. authored_history/material walks the completed checked preparation roster once.
   Actual root entries with missing sides are inserted Conservative and never
   fall through to cache reuse. Eligible warm before custody uses the actual
   selected payload; unavailable selected material may use the actual borrowed
   prepared side, whose later expiration remains conservative. A second impact
   pass shares selected receipts only for scopes excluded by the completed exact
   semantic root roster. Avoid per-impact linear roster scans and Source rebuilds.
4. Final canonical freeze transiently borrows the actual interpretation, then
   binds packet plus weak material map under the same actual envelope self-Weak.
   Deep cloning, serialization and cold decoding cannot recreate either. Existing
   canonical authored cursor/generation/gap and ordinary codec semantics remain.
5. The observation-owned impact API validates original work, actual branch bytes,
   runtime, frozen catalog, live envelope and actual affinity before returning
   Unaffected/Conservative/Retained. Acquisition returns OWNED metadata only and
   performs no opaque upgrade inside its work callback or Native runtime loan.
   The explicit outside-guard payload method pays fixed work before upgrades,
   retains funding and final-polls the original request. Partial/full/cancelled
   strong custody is released at that caller's outside boundary.
6. Extend the SAME external Native owner with actual stable managed-image cells,
   branch-specific admitted/pending lookups, real fork handoff/rollback cleanup and
   existing Native membership-derived roots. Add actual before/after/row-set,
   unrelated scope, scope birth, foreign runtime and actual foreign-cell evidence.
   Reuse the byte-only journal gap/recovery courts. A real plain/over-aligned
   opaque payload court holds acquired shares across cache eviction, verifies
   destruction only on the outside carrier drop, reenters the registration owner,
   requires weak expiration despite retained canonical metadata, and compares
   actual inline weak-quote deltas. No new SDK/root family or runtime src court.

## Verification and commit

Use Rust1.99.1/default flags/features/profiles, configured sccache and
C:/worth-build-cache/target. Never kill builds or clean caches. Settle shape and
independent static review before costly builds. Build the existing external
Native court and run its managed_source cases from the cached binary. Build the
Native owner once for authored history, envelope residency, canonical codecs,
cold-envelope lifetime and opaque cache-retirement controls.

Query Rust draft was never compiled and has been removed; no heavy Query owner
rebuild is needed. Its6 tracked/3 new file draft is preserved outside the repo at
C:/forge_workspace/query919-historical-candidate-query-draft.json. Build the real
consumer entry through application_build.py, then run its existing broad cases
with two already baseline-reproduced cleanup exclusions and the unrun large
geometry case explicit. Include the unchanged prefix25%/50% growth guard once.
Do not repeat broad/owner builds just to restate unchanged receipts.

Run actual Cargo boundary-check/agent-context checks, final cached rescans,
rustfmt, dirty400-line caps, function scrutiny, strict UTF8/whole-text C0/added
mojibake and diff checks. Record genuine current counts and diagnostic failures.
Update Native API/capability/milestone, obtain unconditional independent qa-loop,
qa-tests and code-quality-qa gates, and commit this foundation.

## Next Query boundary

Only after this Native commit, resolve actual outside-runtime/attempt custody for
Query upgrades, matching and strong-share release, including every error path.
Do not mistake release of one inner mutex for that boundary. Then consume actual
historical before/after affected entries through the SAME installed forward
matches_source predicate under admitted Public Parent proof/parameters. Preserve
exact rejects, no-ops, transient entrants, conservative expiry and the independent
ordinary occurrence union. Keep cancellation and strict-original/complete-target
checks unchanged. This and remaining profiles/field precision keep Phase4 open;
Phases5/6 follow its honest closure.
