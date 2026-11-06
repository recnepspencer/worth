# Wave 4 backlog

Collected while wave 3 runs; the wave-4 packets are written from this list, the merged wave-3
notes and W3-12's host-order designs once wave 3 is integrated and measured.

1. **Attempt record, rulings 1-3** (`rulings.md`): `pg::application_commit` with the commit
   receipt; the `CommitDeferred` side channel and its three preconditions; the observed fact
   with the realized-footprint contract lowered whole. Depends on W3-07's merge (two of its
   files) and on W3-05 (source-facts allocation). Design: `packet-notes/W3-09.md`.
2. **The retention envelope**, in the six-step order of `packet-notes/W3-08.md`, after item 1's
   observed-fact step. Its blockers: `SealedComputationFacts.content` -> `FactContent` -> the
   observed fact; `NativeCallCarriage` via `NativeCarriageReads::rebase_changes` ->
   `output_lineage::ComputationChanges`; `CarriedPartitions` -> contribution's partitioned denial.
3. **ComputationPrior design step** (`rulings.md` §5): write the two directions with a
   recommendation for the owner; no code until the ruling.
4. **PendingProducerPublication, option B** (`rulings.md` §4), once its two upward field types
   are lowered by their owners.
5. **`application_checkpoint` -> `application_output_demand`**: the port design in
   `packet-notes/W3-10.md` §4, with W3-12's list.
6. **W3-12's host-order designs** (`host-order-designs.md`), one packet per designed seam.
7. **Court placement** for moved sources: W2-02's `bridge_denial/cause_contracts` and W3-08's two
   sequence courts (integrator step at each merge).
