# D13 retirement census: actual retained-root attribution

Evidence: d13-retirement-census-probe-normalized.log0/1/0 and
[d13-retirement-root-census.json](d13-retirement-root-census.json).
The diagnostic printed the actual decoded mirror and each decoded root,
without changing the original court or its equality assertions. Source
inspection.rs was restored byte-for-byte; its original SHA256 is recorded.

Before: mirror19 entities/10 relations; roots3..8.
After: mirror18 entities/6 relations; roots4..11.
The apparent net entity reduction1 includes three newly created entities in
partitions6,7,8. P0 entities6,12,13,17 and P0 relations2,6,7,9 are absent from
both after-mirror and current root11. Those are the exact expected4+4 removal.
Actual retained roots4..9 still contain retired P0 identities, so the required
union cannot shrink. The inspect operation has not fabricated those rows.

New origin fields were inspected: CaptureIdentity owns only immutable index DATA
and funding. Final Origin owns newly decoded original DATA, image, wrapper and
funding; original envelopes' live acceleration is not cloned by decoding. Staging
artifact ownership is not retained by Origin. Query encode_frame drops the Native
capture before sealing its final immutable buffer.

The old capture path calls run_index_generation_reclamation_pass before component
selection; the new interim freeze does not. This difference is a hypothesis,
not a demonstrated root cause. Read-only owner review found that pass cannot
reclaim a root strongly held by a captured live component basis, and index
generations have no direct basis/root/reader field. The final read-only review also traced
indirect provider payloads through a reclaimed root's managed_source_image. This
fixture installs no managed-index Source provider, so it found no concrete chain
that would release those registered bases. The maintenance hypothesis remains
unproved; a missing call alone does not establish causation. No such call
was added as a patch.
The remaining task is identifying the actual holder of each registered historical
basis and applying the legitimate release semantics at that owner. No unsupported
claim that the test is stale, no assertion reduction, no changed capture roster,
no fresh-index default and no history-only fallback is made here.

Final reviewer: /root/d13_retirement_census_review. No edits, builds or tests. No request-funded complete maintenance API was found; only requestless cold passes and funded retirement of an actual issued individual guard. No speculative maintenance or aggregate charge is added.
