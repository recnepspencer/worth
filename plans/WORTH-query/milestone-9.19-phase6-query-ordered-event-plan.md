# Phase6 Query ordered-event recipe implementation plan

The [boundary review](milestone-9.19-phase6-query-ordered-event-boundary-review.md)
precedes implementation. Deliver the real typed schema->portable->installation->
exact Native binding path for the already qualified Native event operator. Full
root-path membership/currentness is a subsequent required receiver, not implied
by this prerequisite.

Caller DX uses existing named derived relations:

```rust
schema.derived_relation(
    LiveRows::reference(),
    ApplicationDerivedRelation::ordered_events(
        Previous::reference(), Introduces::reference(), Retires::reference(),
    ),
    ApplicationRelationGraphLaws::unbounded(),
)
```

1. Extend application_schema/derived_relation expression with OrderedEvents and
   a typed stored-reference constructor in its recipe owner. Keep physical
   Event/Row endpoint types aligned. Update append offset remapping, prior-node
   and literal/namespace validation, reached-node traversal and legacy-shape
   classification. Reuse the existing physical source closure and named DAG.
2. Extend the schema canonical owner with an explicit ordered-event operator and
   all three role indices. Extend archive member/derived_relation/extended with
   version2 and fresh node tag8 only when OrderedEvents occurs; preserve old
   tag41 and tag47v1 output exactly and reject cross-version/alternate forms.
3. Preserve Native InvalidEventChain as an explicit typed invariant access-denial
   kind, with a focused translation check. Lower the closed syntax to Native::OrderedEvents through the existing
   schema_layout/derived_relation/expression owner. Use existing exact registry
   registration and compiled physical audience; no new evaluator/authority path.
4. Add focused typed/hostile/canonical/portable tests in existing declaration,
   archive and installation owner families, plus an existing Execution owner
   binding test for the actual compiled Native roles and complete source roster.
   Cover composition offset, role swaps, nonliteral/Derived/projected inputs,
   missing source closure, forward/self indices and stale/unknown wire versions.
   Keep literal legacy wire fences and a positive new roundtrip/fresh readmission.
   Do not claim an installed RootPath/Source runtime from metadata tests; Native
   runtime event/refusal/funding courts are the separately accepted substrate.
5. Format and run cheap caps/topology checks first. Compile the narrow existing
   owner targets, run selected new and affected compatibility cases, and reuse
   prior accepted Native runtime evidence unless changes invalidate it. Run real
   boundary/context tools, strict UTF8/LF/C0/added-mojibake/new-link/default diff
   checks and independent qa-loop/qa-tests/code-quality-qa gates. Record actual
   results/limits, commit the coherent slice, then build the root-path receiver.

Destination topology stays in the existing descriptive grammar, codec, canonical
and Native-lowering owners. A subordinate ordered_events authoring/validation
module is appropriate if its semantic steps need a separate file to preserve
400-line caps. No catch-all helper, root/profile/feature/checker exception or
public CAD fixture. All file reads/writes explicitly use UTF8 and LF.

Unqualified work stays explicit: root path shared derived observation/currentness,
private scalar admission/reference paths and persistent sharing, one final SDK
adoption, actual CAD contacts/topology/face scale, Acceptance8 and final clean
private loop gate. No SDK pin changes in this prerequisite.
