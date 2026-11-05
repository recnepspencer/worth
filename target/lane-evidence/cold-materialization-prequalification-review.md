# Cold materialization prequalification review

Verdict: **not ready for expensive qualification**. Five source-level blockers
remain in the reviewed candidate. This is an independent accounting and boundary
review before qualification, not final full FUNDING certification.

The review read AGENTS.md, every coding-guideline document, and the approved
`cold-region-funding-design.md`. It inspected the six granted residency owners,
the four new funding files, and narrowly necessary producer, allocation,
extraction, root-serving, and history-directory analogues. No build, test, or
source edit was performed. No coordinator report, failure-map, or progress draft
was read. Line references describe the candidate inspected on 2026-11-04.

## Findings

1. **High — the broadened physical Full lease bypasses root serving admission.**

   Evidence: `cell/resident_leases.rs:87-98` now returns a Full lease for every
   `ResidencyState::Full`, removing the cold-backing restriction. This method
   already serves `retention.rs:113-121::try_admitted_partition` and
   `retention.rs:178-193::pin_lease`, not just Complete capture. The former
   installs the returned lease into a second retention's `full` OnceLock without
   its root authority's `require_cell` check. That check is otherwise performed
   by `cell/demand.rs::demand` before loading or admitting a view. After any first
   demand or DATA capture materializes the shared cell, another root can now
   acquire a serving partition through these accessors without the root demand.
   `root_view_tests.rs::a_shared_full_load_requires_a_separate_admitted_view_for_each_root`
   explicitly asserts the removed restriction and will reject this behavior.

   Root cause: physical DATA custody and root-qualified serving custody still
   share this existing lease door, so broadening it for capture broadens every
   consumer. Request funding does not prove root membership.

   Correction: preserve the serving callers' admission requirement and expose
   the required counted physical Full DATA borrow at a boundary that cannot
   silently install a root-serving witness. Keep same-cell capture and ordinary
   materialization; do not restore a separate reconstruction route.

   Required verification: existing separate-root test; a negative twin where a
   shared cell is already Full but the second retention has a foreign/missing
   root binding; positive same-cell Complete emission and unchanged original
   resident Full behavior.

   Basis: Architecture 10 (representation must not promote authority), 16
   (required phase proof), and the approved design's unchanged root/kind serving
   admission requirement.

2. **High — InlineBytes payloads have no raw decode allocation quote.**

   Evidence: `region_load/materialization_funding/aspect_state.rs:78-102::scalar`
   quotes Decimal, BigInt, Rational and raw String payloads, then treats every
   remaining variant as inline/fixed-size. `worth-foundational/src/values/aspect_value.rs`
   includes `AspectValue::InlineBytes`. Its implementation in
   `values/scalar_wrappers/inline_bytes.rs` owns a `Vec<u8>`, permits up to
   4,294,967,290 bytes, and its Deserialize visitor grows that Vec. Both raw Head
   and history preflight reach this scalar function through portable aspect
   state, including struct fields. Increasing only an InlineBytes payload does
   not increase either nested `wire` or `native` quote. The separate encoded
   read-buffer ticket funds the still-live source bytes, not this new decoded
   Vec or its native lifetime.

   Root cause: scalar representation coverage is incomplete and the wildcard
   asserts an ownership property that this public scalar family does not have.

   Correction: preflight this sequence's concrete Vec growth/relocation and
   retained payload ownership before deserialization and native readmission.
   Audit the full actual scalar family when making that match exhaustive in
   meaning. Preserve the existing InlineBytes length admission.

   Required verification: raw Head and historical metadata InlineBytes payloads
   under a memory ceiling that admits the read buffer but denies the additional
   decoded backing; retry succeeds with sufficient ordinary request funding;
   corresponding retained-memory growth and release observation. Include a
   struct field payload, since it follows the same scalar path.

   Basis: Architecture 16 and 22; Performance I's rejection-before-construction
   and distinct transient/retained memory contracts; approved concrete aspect
   allocation preflight requirement.

3. **High — Head still reconstructs the complete history custody roster outside
   the request's accounting.**

   Evidence: `region_load.rs:103-110` evaluates
   `cold.history_members.members(cold.history).into_iter().collect::<Vec<_>>()`
   before calling `retain_members`. `durability/data/history_member_directory.rs:80-94::members`
   allocates a pending Vec and a BTreeSet while traversing the entire admitted
   dependency closure. It accepts no request. The Head quote depends on Head
   payload and schema, not this independent graph. Immediately preceding
   `RecordHistoryCustody::reserve` quotes only that custody object's fixed Arc
   layout (`storage/substrate/record_arena/history_prefix/custody.rs:16-28`). A
   small Head with a valid deep or wide dependency graph can therefore perform
   arbitrary closure work and allocate its roster before any Full folder input
   charging. Flat Complete Home histories do not establish the ordinary general
   Head requirement.

   Root cause: the old transitive custody construction remains outside the new
   materialization plan. Funding the final custody header does not fund the
   graph walk, temporary roster, or any new provider token backing.

   Correction: either carry/share an already admitted original custody basis
   where provenance allows that, or pre-admit and charge the actual closure
   owners and traversal under this same request. Preserve arbitrary admitted
   acyclic graph legality; no new depth refusal or maintenance exception.

   Required verification: fixed Head payload with independently growing valid
   dependency closure, denial before roster allocation or traversal exceeds the
   request, and retry; confirm actual provider custody owners are included.

   Basis: Architecture 16 and 22; Performance I's named cost lane, allocation
   lifecycle and bounded request requirements; approved ordinary Head/history
   accounting and carried-proof requirements.

4. **High — the retained ticket can be discarded while reconstructed partition
   backing escapes through existing owned extraction and edit paths.**

   Evidence: the loaders place the aggregate native ticket solely in
   `ResidentRegion._memory` (`region_load.rs:136,238`,
   `resident_history.rs:153`). Existing `lease.rs:29-37::into_owned_contents`
   drops cell custody, then returns `region.partition` if the region Arc is
   unique or a partition clone otherwise. Neither branch transfers or shares
   `_memory`. `Full::into_owned_partition`, `full_edit.rs::into_edit/freeze`,
   `current_edit.rs::into_edit`, and expanded checkpoint extraction consume this
   path. With the caller's cell/root references released, dropping custody also
   releases the cell's raw caches; the last region then drops/refunds `_memory`
   while the returned partition still owns its reconstructed pages and aspect
   backing. If shared, the cloned pages can similarly outlive the original
   region. `partition_from_image` uses ordinary native constructors without a
   request, so these owners do not independently retain the aggregate ticket.

   Root cause: the new funding lifetime is attached to a view wrapper, but the
   pre-existing ownership API permits its underlying shared backing to escape.

   Correction: preserve funding with the escaping backing or transfer it into
   the next actual owning lifecycle before releasing the original reservation.
   An independently funded adoption can also satisfy that transition if it is
   required and occurs before the old ticket is dropped. A retained-until-last
   ResidentRegion-Arc assertion alone cannot close this path.

   Required verification: funded cold Head and Full extraction/edit, release all
   original root/cell/lease/cache owners, retain only the resulting partition or
   frozen edit, and observe that charged memory remains until that actual
   backing's last owner releases it. Cover both unique move and shared clone.

   Basis: Architecture 19 (framework lifecycle ownership); Performance I's
   working-set and retained-state contract; approved ticket-with-actual-backing
   requirement. This is a causal integration issue in existing consumers,
   exposed by the new aggregate reservation, not unrelated worktree debt.

5. **High — Full funding omits the independent peer historical-membership axis.**

   Evidence: `resident_history.rs:357-381::fund_native_image` budgets historical
   kind-cache reconstruction using only twice the count of local relation
   metadata rows. Both Full loaders subsequently replace adjacency with the
   Head's shared tables and rebuild historical kind buckets
   (`region_load.rs:209-217`, `resident_history.rs:126-139`). Reconstruction
   preserves authenticated peer-owned historical claims as well as locally
   derived claims (`kind_projection/reconstruction.rs::rebuild`). A region with
   one local endpoint entity, no local relation metadata, and N retired
   peer-owned relations has zero local `incidence` but requires N historical
   membership nodes. `adjacency.rs::remove` removes a relation from current
   membership while preserving historical membership. The ordinary wire
   adjacency exports current ids only. `funding::current` also omits
   `inventory.optional_cache_bytes`, which is where these kind maps are
   inventoried. Thus the final Full retained quote does not grow with N. The
   temporary history image's ticket drops at function exit, and the still-live
   Head owns the old maps: neither funds the newly rebuilt Full historical maps.
   The owned-Head path additionally captures a temporary peer projection without
   a peer-axis scratch quote.

   Root cause: local relation rows are an invalid upper bound for all region
   kind incidences. Peer historical memberships are authoritative projection
   inputs to reconstruction even though their caches are derived state.

   Correction: quote the actual authenticated projection/membership volume and
   simultaneous capture/rebuild scratch, including peers, before construction;
   retain the final native funding with the Full backing. Reuse existing funded
   projection machinery where it satisfies these exact owner lifetimes.

   Required verification: grow retired peer relation history with local arena
   size and current adjacency held fixed; observe retained and peak growth and
   denial/retry on both backed and owned-Head Full paths. Preserve independent
   root/kind verification and current-format interpretation.

   Basis: Architecture 16 and 22; Performance I's execution breadth, physical
   memory and retained-state contracts; approved adjacency/verification scratch
   and S-inclusive materialization requirement.

## Readiness evidence and limits

The actual same-cell counted-lease emitter extraction is visible. Complete
emission still checks prefix-free Full rows, emits REG6, and produces an empty
history directory. Covered selected reuse accepts only flat histories.
HistoryFolder now carries its decoded image ticket with the escaping Arc,
retains folded-row tickets through native conversion, caches decoded members in
an ExecutionOrderedMap, caches each member's actual direct bases once, and
deduplicates directory checks using retained directory allocation identity.
These address the previously identified folder-lifetime and repeated whole-image
basis-validation defects. General prefix hops H and compared bases V remain
distinct from production flat Home S+T; no universal graph-linear claim is made.
Both Full loaders call the existing `capture_full_for_request` commitment owner,
and the outer native quote does not deliberately add its commitment storage a
second time. No format constant, schema authority, or request-budget increase
was found in the reviewed implementation owners. Finding 1 is the material root
admission change.

| Claim | Distinguishing remaining proof |
| --- | --- |
| Raw decode rejects before additional backing | InlineBytes denial with admitted source bytes |
| Every Head prerequisite is funded | Independent dependency-roster growth and retry |
| Native retained funding follows backing | Unique/shared extraction after original owners drop |
| Full covers all relation memberships | Retired peer history with fixed local/current state |
| Physical capture preserves serving authority | Already-Full cell plus unadmitted/foreign second root |

Existing `funded-cold-owner-check-corrected.log` ends at 2026-11-04
21:22:13 UTC with exit 101: `history_fold_refusals.rs:147` still calls `rows.len()`
on the new FundedRows wrapper (E0599). It records zero tests run. The coordinator
was already correcting compilation, so this is a current evidence limitation,
not a claim that no correction is underway. The new private quote tests cover
arithmetic/value properties only, consistent with placement constraints; they
do not establish the runtime accounting lifetimes above.

Resolve the five source findings, obtain the focused owner compilation, and
inspect the corrected seams before spending the expensive qualification lanes.
Required repository gates and full functional/cost qualification remain owned
by the coordinator and were intentionally not run by this review.
