# Query 9.19 Phase 5: public-only Native Source registration plan

Status: plan complete before Rust. Boundary:
[milestone-9.19-phase5-public-source-boundary-review.md](milestone-9.19-phase5-public-source-boundary-review.md).

## DX and scope

Native compiler callers register an actual schema-bound program:

```rust
let runtime = RelationalRuntimeApi::builder()
    .schema_registry(registry)
    .managed_index_source(program)
    .build()?;
```

Initial application installation uses the same owner:

```rust
runtime.prepare_initial_schema_installation()?
    .install_with_managed_sources(additions, actual_rules, source_programs)?;
```

No rule is required. This Native substrate is not yet a Decl/Host application
authoring contract. Existing builder/custom-invariant/initial install APIs remain
valid and feed one complete roster. No portable bytes or Bank semantics change.

## Ordered implementation

1. Extend private `candidate_source/preparation_group.rs` with explicit provenance,
   complete value equality, and one actual public-group compiler. Validate every
   membership/route against the receiving registry and current field declarations;
   derive canonical structural read/affected access from program kinds, with no
   extra installed work ceiling. Refuse duplicate exact public registrations.
2. Existing Frozen registry accepts validated explicit groups after rule-derived
   groups. Expose crate-private full-roster snapshot/explicit-program derivation
   for transport; no second roster authority or caller-chosen access exists.
3. Thread exact programs through RuntimeBuilder/RuntimeExtensions. Construction
   errors retain `DerivedRelationDenial` as a typed ManagedIndexSource cause.
4. Extend initial installation atomically: preserve existing explicit programs,
   compile additions against merged registry before mutation, rebuild all groups
   once, and retain the full roster in its receipt. Old entry points delegate.
   Extract receipt vocabulary/accessors to a dedicated `receipt.rs` if needed
   for the 400-line cap. Add exact-program comparison for later Query bootstrap.
5. Recovery readmission compares complete ordered group values. Carry the same
   snapshot through recovered codec receipt construction. Fork/checkpoint restore
   already clone the full contract owner; verify rather than add another path.
6. Extend existing codec consequence admission to Source group access. Pass its
   actual frozen registration metadata rather than an entire runtime (that owner
   uses no other runtime state). Add root-free affected/unaffected consequence
   tests; preserve the existing outer typed unsupported error.
7. Extend existing external `managed_source` family (same schema/SourceOwner) with
   builder and initial-install journeys, actual empty/populated/unopened/shared
   maintenance/history, branch-fork affinity, checkpoint restore followed by real
   maintenance, foreign/missing-roster/sealed-install refusals. Reuse existing
   actual observation helpers with narrow sibling visibility. No dummy invariant.
8. Extend original work-account fixture with an explicit public-only choice and
   one compact test exercising actual success, original exhaustion and inherited
   cancellation; keep rule-ceiling evidence unchanged. Update capability index
   narrowly as Native substrate, explicitly leaving Query adoption unfinished.
9. Format/static-review before build, run measured scoped evidence, independent
   qa-loop/qa-tests/code-quality-qa final gates, save evidence, commit.

## Verification and compile fences

Use Rust 1.99.1/configured sccache, original flags/features/profiles,
`C:/worth-build-cache/target`, runtime stack 67108864. Do not clean/kill builds.
First build existing Native external target once, execute inventoried public-only
new tests and all existing managed Source, original-invoice/support-growth,
compound publication and checkpoint/initial-install relevant courts. Build Native
owner once for registry/program/history/initial-install/consequence owner tests.
A Query owner compile may be necessary because a new construction-denial variant
and receipt layout cross Native/Query; inspect actual exhaustiveness before
building, and use existing consumer target only if that is the needed public
receiving proof. Do not run unrelated Bank/scale builds before Query integration.

Run actual Cargo boundary-check/agent-context, final cached rescans, rustfmt,
dirty line caps/function scrutiny and strict UTF8/C0/added-mojibake/diff checks.
Use exact test inventory and count distinct passes. Existing measured evidence
must not be presented as a new Query/Bank acceptance pass.

## Remaining milestone work

Query public maintenance policy and portable v5 preserving v1-v4 bytes; real
bootstrap Source-roster comparison and lowering; Bank compatibility plus actual
provider/ordinary receiver adoption; topology box/contact cutover and public
scale/lifecycle qualification; stable facades/docs; final one-pin private Phase6.
Native Source-affected codec migration and generic low-level recovery preservation
remain unsupported, not hidden fallbacks.

Prebuild boundary integration: the new Native initial-denial variant requires the
actual Query `primary_graph/initial_schema_denial.rs` exhaustive mapping to existing
RelationalSchemaRejected with its original detail. Both actual codec-consequence
callers (restore admission and transition admission) must consume frozen metadata.
The former keeps its existing SchemaMismatch refusal; the latter keeps
UnsupportedCodecConsequence. Cached boundary validation admits the new external
courts in their existing family without configuration exemptions. Compile the
actual consumer entry once to prove the receiving Query/facade/bootstrap path,
then run its relevant managed initialization/correction/recovery cases; a second
heavy Query owner build adds no necessary proof for this signature mapping.

Runtime boundary confirmation: the existing history owner propagates an absent
parent authored generation after native-checkpoint restore. A successful live
Source mutation does not reconstruct that optional history accelerator. The
checkpoint court must prove new root content/revision and actual selector
admission, then assert exact GenerationUnavailable. Fresh builder/initial tests
prove complete live history separately. No production history change or hidden
rebuild is part of this slice; independent reviewer confirmed the owner path.

Independent review added one mixed-association obligation: register the same exact
program publicly and through a real rule in the existing work-account sample.
The one-work installed rule allowance must retain typed RuleLimit and zero provider
contacts; the same-data twin without that limit must succeed. This proves
end-to-end no bypass, not which of the two preparation owners refuses first.
No new hook, ceiling or fixture family is required.
