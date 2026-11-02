# Query 9.19 Phase 5: portable public maintenance and Query handoff plan

Status: complete before Rust edits. Baseline `61b17083ed`.
[Boundary brief](milestone-9.19-phase5-query-maintenance-boundary-review.md).
Independent review selected the additive geometry/public and refusing-prefix twins.

## Public DX and unchanged contracts

```rust
impl ApplicationManagedIndexBinding<MySchema> for MyIndex {
    // Existing SourceBinding, ProbeBinding, Entity and provider contracts.
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
}
// Same contribution setup.managed_index / provider(...).install entrance.
// The real Source is prepared before publication even without an invariant.
```

Default WhenRequired requests no additional public maintenance; real invariant
requirements still apply, and refusing audiences cannot borrow another admission.
Existing default unrefined/refined typed definitions remain v3/v4 respectively.
No public Parts fields or old bytes change. Add actual
ApplicationQueryBasisSupport::current_only(), transported by its existing flags,
for the new one-shot Parent and later distinct Bank current-only adoption.

## Ordered implementation and proof

1. **Freeze the old format before changing production.** In the existing archive
   managed-index owner fixture, capture full member wire bytes for valid v1, v2,
   v3 and refined v4 from the baseline implementation, through one narrowly run
   pure owner test. Preserve immutable literal hex goldens in its final test;
   no normal test writes files and no new runtime/root family. This baseline run
   precedes all production meaning/codec edits. Keep the fixture's explicit
   portable identities and full Probe/Source payload stable.
2. **Portable meaning.** Add a pure Foundational ManagedIndexMaintenance enum
   (WhenRequired default, Native with existing Cold posture), reexport through
   Decl, and defaulted binding MAINTENANCE. Erased definition gets a private
   optional explicit field, effective getter and explicit untrusted constructor.
   Absence is the only legal v1-v4 representation; explicit metadata only v5;
   v5 absence refuses. Keep Parts literals unchanged and full Eq/Ord/typed
   descriptor matching. Typed default selects old v3/v4; explicit Native selects
   v5. Retain required v4 refinement and valid optional v5 refinement, rejecting
   unsupported candidate/refinement combinations and future versions.
3. **Archive/canonical projection.** Keep v1-v4 branches byte-for-byte intact.
   V5 encodes explicit mode/cold and optional refinement with checked tags;
   unknown/truncated inputs refuse before borrowing absent payload. Canonical
   append adds maintenance/cold/presence only for v5; keep the existing global
   schema canonical version. Test full old goldens, v5 both cold postures/default
   explicit mode, absent/unknown/truncated tags, optional/refined twins and
   canonical mode/cold/refinement separation. Old constructors cannot invent v5.
4. **Actual basis support.** Add current_only at the existing Decl owner. Decode
   current=true/pinned=false with the same preview flag; keep old true/true bytes
   exactly and reject current=false. Preserve preview and continuation validation;
   continuation without pinned remains denied. Add focused owner/codec evidence.
5. **One lowering roster.** In schema_layout/managed_source derive the combined
   required index/cold set once from installed declaration members (explicit
   Native requests plus actual invariant requirements). Refuse wins. Each lowered
   entry carries exact definition, effective cold and actual schema program.
   SourceInstallation binds factories from this same map, once per actual program,
   instead of independently reconstructing the rule-only set. Full definition
   comparison includes new metadata. Remove the displaced required-set iterator
   from invariant_factories.lower; retain its real factory inventory, callbacks,
   access/work and program-selection gates. Unsupported shapes remain typed
   refusals; no ordinary-cache fallback is admitted.
6. **Native bootstrap handoff.** Pass only the genuinely explicit public programs
   to install_with_managed_sources, and all compiled programs to the receipt's
   descriptive exact-set check. Keep actual real-rule registration transport and
   full Native recovery receipt behavior. Preserve initialization issuer and
   effective cold policy across both audiences; introduce no fresh budget.
7. **Actual existing public consumer.** Add a public-only geometry index/Parent/
   binding in the fixed maintained Catalog, over its existing SourceBinding,
   CoordinatesBinding and Edge. Its distinct index-name Native program has no
   invariant requirement. Parent is explicitly one-shot/current-only, v5 without
   refinement. Generalize only the existing provider base impl and bounds /
   row_bounds / summary_fault helpers under exact associated-type equality.
   Keep provider identity/version fixed to the actual implementation; keep old
   specific predicate/refinement impls and numerical/storage mechanics unchanged.
   Separate new Calls in existing Configuration. Declare/configure through that
   same contribution; no new schema, root family, provider engine or fallback.
8. **Adversarial receiving proof.** New actual journeys assert no invariant targets
   this index, independently ready Source before any manual rebuild, actual new
   Calls, full-scan rows after real Move/Share/Edge edits, unopened shared scopes,
   warm positive updates with stable build count, retained old rows, current-only
   pinned refusal before provider callbacks, cancellation with unchanged World
   and genuine fresh retry. Use current geometry authoring/owned Source rows and
   direct independent closed-bound comparisons, not provider traversal as oracle.
   Extend narrow helper visibility only where needed inside this composition.
9. **Joined cold proof.** Add the shared prefix fixture MAINTENANCE profile
   constant to its existing four compositions. Only refusing opts explicit public
   Native/AdmitRebuild; its genuine rule remains Refuse. Existing explicit
   initialization courts must still refuse automatic cold acquisition and admit
   the lawful initialization twin. Original/candidate Completed Box data, eight
   tests and lifecycle assertions remain unchanged. Default maintained profiles
   retain old v4/v3 meaning unless the new index explicitly requests maintenance.
10. **Qualify changed roster.** After small/focused gates pass, run existing real
    geometry 1000/2000 growth evidence with unchanged original guards. The added
    independent Source program changes actual maintenance cost, so historical
    invoices are not a current acceptance receipt. Preserve honest whole-source
    setup and all declared scopes; do not borrow the invariant index's root to
    evade that cost. This is not 50k topology discovery qualification.
11. Update capability/guide/examples with actual public authoring, current-only
    posture, original controls, effective cold join, unsupported shapes and
    remaining Bank/topology/private work. Run independent qa-loop, qa-tests and
    code-quality-qa gates, save measured evidence and commit this slice.

## Checks and compile discipline

Rust 1.99.1/configured sccache, original flags/features/profiles,
C:/worth-build-cache/target; runtime stack 67108864. No clean/kill or new test-only
feature. Static review/format/caps before first changed-production build. Capture
legacy goldens with the existing pure archive owner before production edits.

Use the existing Query workspace owner targets for declaration and archive tests;
focus on managed-index meaning/canonical, basis/control, malformed transport and
legacy goldens. Build the existing consumer entry once through application_build.py
(no --lib), run refusing join and new actual public-only geometry first, then
existing managed/refined/correction/ordinary receiver groups. Run changed-roster
geometry growth only after quick gates, once; include it in distinct counts if a
broader consumer run skips that already-run heavy case. Keep the two established
unrelated cleanup failures explicit; do not claim an unfiltered green run.

No second heavy Query owner runtime build or Native owner rebuild unless actual
production changes/failures require it. Source metadata/lowering/protocol proof
is supplied by pure owners plus actual consumer, not new unit runtime courts.
Run actual Cargo boundary-check/agent-context, cached final rescans, dirty rustfmt,
400-line cap/function scrutiny, strict UTF8/C0/added-mojibake/diff checks. Review
all touched >400-line files causally; no unrelated cleanup or allowance changes.

## Deferred scope

Actual Bank current-only adoption preserving executable v1 pinned contract,
guarded pending-payment/activity/continuation meanings, real public topology
spatial discovery and 50k acceptance, final public documentation closure, and the
single final private Phase6 pin/adoption. No public pass supplies those claims.

The whole-zero-rule Native court plus the real Query no-rule-associated index
are composed evidence, not a whole-Query-zero-rule runtime claim. The new index's
own ready roots/counters/results must convict missing public installation even
while a different invariant index consumes an equal-looking physical Source.

Implementation sizing: the added exact Source receipt check takes bootstrap's
preparation coordinator over400 lines. Extract the one-way Native install and
both exact inventory checks into bootstrap/initial_installation.rs; keep existing
admission/order in preparation. Add generic Source-shape/Source-receipt diagnostic
kinds for these now shared boundaries while retaining invariant-specific failures
at real invariant loan/factory boundaries. This is causal owner placement, not an
extra runtime/authority lane.

## Resolved actual current-selection handoff

Runtime controls previously classify every selected snapshot as pinned. Declaration
and transport alone would therefore leave current_only unusable. The independent
review traced the actual issuer: ProductEntry::select_with_request and
select_for_managed_index call admit_product_occurrence. Carry private selection
origin from those routes through the selected operation and shared progression.
Raw on_product, temporal/adoption/retained/continuation controls retain Selected
origin. Publication capability, pointer equality and head identity are insufficient.
This describes current at selection; it does not refresh or promise current at
execution. Fresh security and Native same-product checks remain separate.

Implementation: use an opaque runtime selection-origin value whose current
constructor is private to the actual context owner. Default controls to retained
Selected; ordinary/governed direct one-shot admission and actual shared-read
handoffs carry the value with their existing snapshot leases. Same-selection
retention preserves origin without reselecting. Validate current support only for
that carried origin, pinned support otherwise. Preserve public SelectedProduct
snapshot receipts and executor/governance dispatch. Prove current success and
same-unchanged-head explicit .at(...) BasisUnsupported before callbacks.

The shared prefix adapter reaches402 lines after the actual profile requirement.
Extract its typed index declaration into prefix/definition.rs (same inherited
composition module; preserve public PrefixIndex and compiled-example markers).
The provider adapter retains all actual mechanics and calls; no new facade or
authority is introduced.

Concrete consumer authoring uses the existing staged declaration entrance with
the shared genuine Source result shape, selecting current_only at its actual
basis stage before lanes/public/managed_index. The completed base builder has no
basis override; no platform convenience setter is added. Schema inspection uses
the actual declaration.erased().members() public door.

## Blocking large-growth finding and revised attribution step

The unchanged full geometry test completed1104.36s and failed its difference-of-
invoices refusal guard:1000=(3250687,2925262),2000=(3370454,3094510),
2000+50000=(4624211,4309867). Local refusal delta119767 permits958136;
remote delta1253757 exceeds it. Commit,25% and50% guards pass. This is an
acceptance blocker, not proof of linear work. Original provider callbacks remain
bounded; current SDK observation labels only that provider. Public Calls and its
real maintenance execute inside unattributed gaps. Local doubling grows A while
remote edges genuinely grow both A and B; preserve that full authored topology.

Before choosing a production optimization, extend the existing test-only geometry
work_attribution owner to label BOTH real provider Calls inside ONE original SDK
observer/invoice. Keep raw required-prefix intervals distinct from accepted work;
do not sum overlapping spans or add a second payer. Retain/restore both observers
on the real Calls owner, including unwind. Record both providers' actual build,
update, Source-bound and summary-source deltas; require no hidden warm rebuild and
local changed-Source work for the public provider too. Run existing small/fixed
growth gates first. Then use attribution to select a principled repeated-work
optimization and repeat the unchanged large gate. No guard/denominator, Source
roster, remote sharing or dataset reduction is authorized to clear this failure.
The current slice remains uncommitted until required evidence and independent QA
are clear. Source authorization/Bank work is deferred behind this blocker.

## Reviewed production optimization after dual-provider attribution

Dual small16/32 and fixed64+remote0/256/1024 tests pass. Both actual providers
report build0/update2/bounds4/summary4 per measured edit and identical labeled
update spans. There is no measured hidden warm rebuild. Reviewer selected the
smallest demonstrated repeated-work owner: pure box_tree/edit.rs.

At each insert/extract unwind, write_branch reads both actual children and builds
the parent, then balance rereads that parent and the same children immediately.
Fuse that handoff by returning the just-read Copy child nodes with the branch
write and passing them to balance. Preserve actual storage.replace (including
NodeWriter liveness checks), key ordering, checked heights, exact integer union,
all rotations/depth controls and Source occurrence identity. No cache, heap,
new grant or alternative tree algorithm is introduced. Bounded stack copies are
mechanical data within the exclusive storage borrow. The numerical owner remains
consumer_values Box2/order: exact i64 closed bounds in its declared length/XY
frame and exact65-bit centroid order; no rounding/tolerance/frame conversion.

Review the existing value_edits/boxes pure real-storage/interpreter cases for
rotation/rekey/removal/retained-snapshot coverage; add only a proportionate strong
read-count/value oracle if needed. Build the existing consumer once, run pure
edit/parity and affected public geometry/correction groups, then dual small/fixed
invoices before deciding on another full50k run. Smaller absolute invoices alone
do not satisfy the unchanged8x ratio: its local denominator may also fall.
Generic summary traversal and broader shared-Source redesign remain deferred.

## Exact-equal geometry backing writes

Fused pure reads pass194 selected consumers and four edit/parity tests; dual small
invoices16=(2500261,2565428),32=(2559394,2609140) save only6-7k units. Do not
repeat the1104s gate on that evidence alone. Reviewer traced the next honest
owner: geometry/provider/storage.rs::Edit::replace. Its full Node is fixed-size
exact data. Derive Eq/PartialEq for BoxNode and BoxNodeLinks; compare ALL metadata
(full child/source IDs, bounds,last_source,last_bounds,height) after actual
self.read(id) owner/liveness/control validation. Equal values poll original
control and omit replacement/observer; unequal values retain the existing
NodeWriter.replace path and post-write observer. Count the actual comparison read
in provider navigation. Do not alter generic NodeWriter or BoxEditStorage Id
bounds, infer liveness from equality, cache data or fire observers for no write.

This frontier tracks provider backing effects, not authored intent. Actual Native
SourceChanges independently seed before/after Source nodes and complete final
ancestors in summary_frontier::affected, including with zero node edits. Native
Source row/history publication and exact summary recomputation remain unchanged.
Retain actual-write cancellation and no-op/summary-staleness courts. Compare new
small/fixed original invoices before selecting the unchanged large rerun.

## Independent QA correction to the scaling workload

QA-tests identified a false premise in the old8x calibration, independently of
the red receipt. The1000→2000 samples grow A alone; B retains two shared chain
edges. Remote50000 grows both A and B, making B's added height about14-15 levels,
not the comment's fewer-than-six. That derivative cannot independently model both
changing population axes. Preserve the old failed1104.36s receipt explicitly.

Correct the LARGE scaling court to author the initial chain into BOTH A and B
for1000,2000 and2000+50000. All remote edges remain authored into BOTH scopes;
all25%/50%/8x thresholds, two touched occurrences per scope, genuine refusal/commit,
full census/independent rows, public/rule provider observations and Source-work
guards remain. This adds realistic B pressure to the local baseline; it does not
remove remote work. Keep the sparse-B shape in existing16/32 and fixed64 with
remote0/256/1024 courts via one explicit SharedChain population input. Move the
independent authored-universe/closed-comparison interpreter into growth/population.rs
to stay below400 lines; it consumes declared inputs, never provider output.

This is corrected paired-scope qualification, NOT an unchanged original test or
a production fix of its failed receipt. The new full original invoices must pass
afresh. Do not change the thresholds if the corrected workload still fails.
The two production optimizations are reported separately by actual correctness
and measured savings; neither is claimed to have repaired the old full guard.
