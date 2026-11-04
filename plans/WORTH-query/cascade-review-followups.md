# Source-scope cascade review follow-ups

Issue #44 / D1075, following certified #42. Base: replayed master fb5bf38626;
the identical patch must apply to pin 7a8569b3b8.

## Boundary review

Native's recorded publication owns its source identity. The shared closure scan
currently discards that identity when route validation or installed ownership
fails; authored readmission then substitutes the cascade anchor. Materialization
payload validation also knows the associated original publication before it
checks the record. Preserve that context through a private history-failure type;
request stops remain request stops, and unassociated history failures may still
name the anchor. Existing non-authored consumers keep their original stop kind
and detail. No new public authority, control, or serialized field is needed.

The exact candidate roster owns the hidden-deletion guard. Query's real producer
can delete an unrelated ordinary entity without declaring a Retire output role;
that is distinct from an extra declared role, which the existing courts cover.
The distinguishing court must reach Native and assert ExtraEntity for that exact
ordinary entity, with no publication or graph effects.

Producer ownership is a binding constant, not runtime configuration. Add an
independent generator binding to the existing public fixture and an additional
program output graph using the same child and leaf producers. This preserves all
existing opted-in scenarios and tests the documented independent-parent case.

## Implementation plan

Topology stays under Native `history/data/codec_retirement_closure/`:
`publication_failure.rs` carries the subject, `source_scope_closure.rs` attaches
route/ownership failures, and `materialization/payload.rs` attaches record
failures. The Native court is `authored_retirement/corruption_subjects.rs` in
`worth-court-relational-certification`. Query courts remain in the existing
decl-host `producer_succession/` harness: `refusal_courts.rs` covers the hidden
deletion and `independent_generator_courts.rs` owns the independent scenario.
The independent output graph has distinct parent, child and leaf input ports;
the original two graphs and opted-in producer bindings remain available.

1. Carry optional publication source plus the original stop under the Native
   codec-retirement owner. Attach it where route/ownership or materialization
   record validation identifies the original publication. Authored readmission
   emits typed CorruptPublication for that source. Add a real Native authored
   readmission court using authentic history and a mismatching installed
   ownership declaration; its child source must differ from its anchor.
2. Extend the existing Query fault fixture with an undeclared deletion of an
   ordinary unrelated realization. Assert the exact typed refusal and unchanged
   live records. Disable only the hidden-deletion guard, run that court, record
   the failure, restore exact source, invalidate its mtime, and rerun green.
3. Add an independent generator family and program graph; shrink through the
   real Query producer path, retire the owned descendant closure, and preserve
   sibling identity. Assert the binding really declares Independent ownership.
4. Run the focused Native and Query courts, existing authored and publication
   continuity courts, formatting, dirty line caps, boundary-check and
   agent-context. Commit on master, cherry-pick that commit without adaptation
   onto 7a8569b3b8, and verify there with unshimmed Cargo. Report exact commands,
   outcomes, mutation restoration and commit SHAs. Do not push.

## QA considerations

Failure subjects must identify an associated corrupted publication rather than
an unrelated anchor. Resource stops must not be recast as corruption. Hidden
entity deletion must be observed independently of declared roles; the mutation
must expose acceptance or a changed refusal. The independent generator court
must actually install and select its independent binding, with opted-in child
publications. Existing authority, budget, atomicity and history rules remain.
No unrelated repository debt or resource-accounting redesign is in scope.
