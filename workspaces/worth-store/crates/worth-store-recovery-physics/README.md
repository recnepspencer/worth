# worth-store-recovery-physics

Owns the narrow pure-law portion of physical recovery: WAL segments, LSNs,
pageLSNs, source precedence, operation-fate reconciliation, idempotent page
redo, and bounded redo planning.

This is physical recovery machinery. It must not become an alternate semantic
truth source.

It does not own filesystem discovery, runtime progression, Store coordination,
offline observation, report protocols, or physical effects. Those belong to
the recovery runtime, Store, offline verifier, Foundational protocol
vocabulary, and C.4 backend respectively.

The C.7 boundary and C.8 handoff are documented in
[_docs/worth-store/physical-durability-and-checkpoints.md](../../../../_docs/worth-store/physical-durability-and-checkpoints.md).

For C.8, this crate remains pure meaning only. Its public laws decide
current/previous source precedence, WAL-prefix continuity, checkpoint-covered
WAL ranges, pageLSN apply/skip eligibility, operation fates, and finite plan
cost. It owns no clock, filesystem walk, Store authority, process lifecycle,
observer report, runtime effect, replay surface, or reconstruction result.

The recovery runtime consumes these laws through its concrete owner facades;
the offline verifier interprets persisted bytes independently. Ordinary
runtime and observer lanes must not import replay or reconstruction. C.8 is a
fresh physical reopen, not backup/PITR, rollback, or semantic repair. The
runtime's fixed limits and the observer's four-axis limits are admitted by
their owning entry points rather than configured by this pure crate.

A selected page newer than a historical WAL target is not generically eligible
for a pageLSN skip. The private supersession calculation requires an exact
selected image/coordinate/digest/LSN anchor in admitted WAL, consecutive page
generations through a continuous root lineage, and byte-for-byte preservation
of earlier record identities and slots. It retains exact historical LSN/target
pairs; unproved generation mismatches still deny, and Apply retains its strict
successor rules. The calculation reuses already-admitted inline descriptor
ranges rather than introducing a second raw decoder.
The admission budget reserves a conservative 4 KiB per target, frame, and
placement for retained ranges and overlapping history/index scratch. This is
checked before retaining those facts, carried into the immutable plan, and
included in the runtime's aggregate peak-memory limit; it is a conservative
budget charge, not a measured allocator-byte count.
