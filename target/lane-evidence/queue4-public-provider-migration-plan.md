# BC9002 public-provider migration

Boundary review: queue-boundary-independent-review.md, item4. The external
ImmutableCheckpointMemberSource already preserves lazy selected protection
without copying/funding all members. Explicit test-owned TestDurableMedium
handles replace runtime-private attachment lookups. No new observer, authority
conversion, alias or test feature is needed.

Inventory: queue4-provider-holder-inventory.json records104 direct provider /
attachment references; queue4-all-medium-holder-inventory.json additionally
tracks all262 Native files mentioning the medium or durable-runtime fixture
builders. All recorded owners pass the conditional Lifecycle comparison.
These inventories are scopes to trace, not assertions that all262 files need
to move or that every private assertion has already been audited.

Implementation order:
1. Introduce one externally constructed cold-checkpoint fixture in the existing
   relational testkit. It owns the original medium, immutable member source,
   real Native checkpoint/recovery input and receiving runtime. Publications,
   schema, frame readmission and recovery use the existing Native facade.
2. Migrate recovered-envelope custody through public replay shares. Preserve
   exact receipt and same-body identity via shared as_ref pointers, request
   funding unchanged while one of two shares survives, payload refund at final
   share drop, injected source refusal, unchanged frontier/offers and retry.
   A separate read ledger uses the SAME original unlimited-memory policy;
   it observes allocations without raising or padding any ceiling. Fixed
   control allocation legitimately remains while the runtime retains Weak;
   complete refund is required after dropping that owner too.
3. Continue the cold source family, retaining actual member locators from the
   captured checkpoint. Split pure index-copy assertions into root-free owner
   units where public courts cannot observe them, using the actual subsystem
   fork and value rows. Keep real page/source/funding work in external courts.
4. Migrate the remaining attachment-dependent durable tests with explicit
   medium handles; delete the three local adapters and cfg(test) hooks only
   when no consumer remains. Shrink the baseline; never hide root signatures.

Preserve a before/after test-name multiset. No ignored/exhaustive campaign is
selected. Compile each coherent migration batch before its named focused tests;
run required format/line/boundary/context gates at phase completion. All new
outside-area files receive the exact base-to-Lifecycle eligibility probe. If
an owner is Lifecycle-held or a claim has no honest existing public or pure
value proof, report that precise part rather than silently discard it.

The existing certification court is22,776 physical Rust lines; the current
testkit is9,747. Before a broader addition exceeds governed source capacity,
derive necessary court splits from actual module ownership/dependency frontier
and add only exact workspace/config entries. Do not import unrelated helper
families or move private tests merely because they are nearby.

Acceptance remains zero BC9002, source baseline shrinking only, every migrated
claim mapped to public observations or a root-free unit, fresh independent
approval and a separate queue-item commit. This plan is not completion evidence.
