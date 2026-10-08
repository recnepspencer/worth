# Restored publication-head acceptance

## Boundary and implementation

The public baseline `901f52ef312609a3d1e9de80cd39556a6c0aa167` already
contains `64aae52872`'s native family-head checkpoint cutover. Checkpoint
capture selects heads from the native publication index before canonical
serialization loses their live order. Its `native_priors::merge` filters both
current accepted payloads and recovered payloads against that selection.
Restoration readmits installed producers and compares recorded source facts
and native output evidence; a digest or checkpoint row sort is not authority
to select between competing bindings.

This increment adds acceptance coverage and explains that existing contract.
It does not create a second production selection path or invent publication
order for older archives. It is independent of package-identity changes.

The first working journey uses the existing reusable `CheckpointSchema`,
`CheckpointProgram`, and distinct final-output Initial and Preserve producers.
It creates Initial through the ordinary demand, captures opaque bytes, drops
the runtime, and freshly installs those bytes. A real source edit then causes
Preserve to publish value 11 over the original output entity. A second capture
must remove the genuine recovered Initial row from the exported family head.
After another fresh installation, an ordinary mutation handler's
`DecisionReader::current_output` reads the final family before any demand.
Both root and final-provider callback counters start before installation and
must stay zero. An ordinary edit of the generated Length then requires a typed
`StaleSource` denial from native output verification. The existing root-source
current-output regression independently protects changed tracked source facts.

## Artifact responsibilities

- `consumer_values/src/planar_operation.rs` adds the fixture's final-family
  current-output read intent without importing Query into pure values.
- `consumer_entry/topology_entry/src/handler.rs` routes that intent through the
  existing real DecisionReader and preserves typed selection/denial outcomes.
- `checkpoint_recovery/current_family_output.rs` owns the two-reopen journey
  and generated-output negative control.
- `topology_entry/src/final_output.rs` adds test-only callback observations
  for the actual provider used by both Initial and delegated Preserve.
- `worth-query-host/README.md` explains capture-head selection and the limit
  on reconstructing order absent from an older archive.

No production authority type, wire format, budget, provider implementation,
or source-fact equivalence contract changes. Runtime publication selection
continues to precede source-fact validation; older Initial cannot be revived
merely because its facts happen to match.

## Acceptance and limits

The meaningful sensitivity control removed only native-head filtering from
checkpoint capture while preserving real native capture and reconstruction.
The final two-reopen case failed at the cold current-output read with typed
`OutputUnavailable`, rather than from setup, compilation, or resource denial.
The production source was restored byte-for-byte, its modification time was
invalidated, and the clean artifact was rebuilt before final testing. The
complete affected court passed all 34 tests and all 26 doctests on that clean
build, including both the new native-output negative and the existing tracked
root-source negative. Formatting, dirty line caps, generated context and
whitespace checks passed; boundary checking reproduced exactly six untouched
baseline UI source-reachability diagnostics with no scoped diagnostic.

Earlier single-reopen variants stayed green with that filter disabled. They
did not retain a competing recovered Initial payload and are not accepted as
evidence for this seam. The retained test uses real earlier-checkpoint custody
instead of synthetic rows. An initial negative that edited source PositionY
did not deny the final family because its handler did not track that field;
the retained negative edits the actual generated output witness instead.

Qualification uses Rust 1.94 with the fixture's dependency optimization
override and finite installed budgets unchanged. The broader CI court has a
demonstrated pre-existing Rust 1.98 compatibility gate: clean baseline and the
independent package-identity candidate both pass 16 tests and fail the exact
same 17 tests. Both pass the original 33 tests and 26 doctests on Rust 1.94;
the package candidate also passes the fixture's default optimization profile
on 1.94. The specific layout or accounting mechanism is not qualified by this
comparison. This increment qualifies only Rust 1.94 and does not repair that
separate compiler lane. Known untouched UI source-reachability failures remain
separate.

Fresh native application authoring, save, new-process reopen, edit and actual
post-edit pixels remain a downstream acceptance gate. This public fixture
does not certify that product journey. An archive that already lost native
publication order receives no retrospective repair guarantee from this
capture-side rule.
