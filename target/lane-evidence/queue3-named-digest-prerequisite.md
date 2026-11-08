# Named digest cutover: missing integration-owned type encoding

Status: BLOCKED on the integration-owned Foundation type prerequisite, not
implemented with a parallel codec. No product source changed for this item;
focused compile/codec tests are unrun.

Current branch has CanonicalDigestId in
`crates/worth-foundational/src/identities/category_ids.rs:54`, with Debug,
Clone, Copy, Eq, Ord and Hash only. It has no Serde implementation/module.
Search: `rg -n 'CanonicalDigestId' crates/worth-foundational/src --glob '*.rs'`,
plus the complete type definition. The working nearest analogue is integration's
same definition: `git show codex/m3-host-home:crates/worth-foundational/src/identities/category_ids.rs`
declares `mod canonical_digest_wire`, and that private leaf implements the
strict fixed-width binary Serde encoding at the named type.

Required integration-owned files:
- `crates/worth-foundational/src/identities/category_ids.rs`;
- `crates/worth-foundational/src/identities/category_ids/canonical_digest_wire.rs`.

These are outside the Native area grant, and the conditional outside-area
grant explicitly applies to items1,2,4. Integration owns digest definitions.
The prior exact Foundation grant was clone_allocation_bytes only. No merge
head has been supplied. I did not edit these files or copy the codec into a
new Native compatibility adapter.

Remaining raw digest fields were inventoried with:
`rg -n '\[u8;\s*(16|20|32|48|64)\]' crates/worth-relational/src/durability/canonical_commit_artifact crates/worth-relational/src/durability/checkpoint_admission crates/worth-relational/src/durability/authority/checkpoint_complete --glob '*.rs'`.
Examples in owned producers/consumers are CapturedRelationalCheckpointAdmissionImage
checkpoint/source/initial digest fields, projected Coordinates/Schema/Visibility
fields in admission_projection.rs and admitted_commit.rs, reference/root-directory
projections, and selected-source digest/map keys. Raw-array binary-encoding tests
also remain. The existing BinaryFixedArrays serializer requires its T to
implement Serialize; wrapping the named type cannot supply its absent Serde.

The intended cutover uses integration's named type directly, preserving the
existing strict binary wire policy and typed predecessor refusal. Transferred
original_image.rs and branches.rs, branch/root_* and the ten exclusions are
untouched. The next action is the coordinator-named integration merge (or an
explicit owner handoff for its type definition), then the owned consumer
cutover and focused wire/refusal tests. Proceeding to BC9002 does not claim
this item complete.
