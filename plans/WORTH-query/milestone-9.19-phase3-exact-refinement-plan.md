# 9.19 Phase3: tracked row-local exact refinement

## Boundary brief (completed before implementation plan)

Clean authoritative HEAD is c1ffedd9d1. Complete candidate traversal, verified
visited/pruned cut, inverse watch routing, same-image registration and fresh
governed disclosure are implemented. They explicitly do not establish domain
exact refinement or Phase4 footprint narrowing.

The shared executable seam is TypedIndexProvider::candidates. Public cache,
maintained Parent and Native invariant reads all call it with an actual
CoverageFrame, then receive before projection. Refinement in Row::project or
only the public receiver would bypass Native correctness and could filter after
counts/cardinality/limits. Candidate frontier Matched dispositions currently
biject to genuine candidate slots; deleting/relabeling exact rejects there would
forge candidate coverage. CoverageVerifier.complete already verifies the actual
cut and candidate bijection before creating CandidateSelection. Keep that proof,
then complete a separate framework-owned refinement pass over every candidate.

SourceRow is a row-local loan: its projected root/related endpoint fields belong
to the genuine Source/probe/session. A refiner may borrow those fields and the
same typed probe under original controls; it gets no Source roster, other-row
lookup or external SDK/Query entrance. Full conservative Source dependencies
remain. The first profile therefore needs no new dependency pullback. The
existing exact Source/Parent projection correspondence bounds every input.
The canonical governed influence owner must cover every such input for ordinary
refined one-shot Parents as well as Live. Coverage/refinement does not grant READ.

Current geometry's AtMostTwoProbeEnclosures and prefix's reserved-name prefix
rule are real candidate business meanings. Do not silently change them. Both
candidate and refined access must coexist on one existing Index/Source/root and
one P allocation. Explicit requested mode belongs to declaration/installed access,
canonical archive/obligation identity and CoverageFrame receiver correspondence.
No undeclared/unconfigured or candidate-only source can claim exact completion.
The old descriptor/selector shapes and archive versions retain candidate meaning.

A new optional versioned row-local refinement meaning on the Index describes the
domain contract separately from candidate membership; a Parent explicitly asks
for it. Runtime configuration binds the matching refinement adapter to the same
P Arc as forward and inverse mechanics, without a backedge to TypedIndexProvider.
Native gets an explicit refined query entrance; its default remains candidates.
The callback returns only one row's bool/typed stop; Query owns complete iteration,
accepted/rejected dispositions and accepted slots. Empty Sources still validate
the refinement probe. Stops/refusals never become false or complete-empty.

Inverse tree enclosures remain candidate-conservative. At each surviving actual
record, the common candidate/exact evaluator reads genuine before/after row and
probe in that record's requested mode. Thus an input edit that keeps candidate
truth but changes exact truth reaches the right watch. Equal probes in the two
modes remain separate globally issued slots. Ordinary projection changes of
matching rows still notify; no dynamic refined-query scan is introduced.

Numerical qualification stays at the existing query-agnostic consumer_values
fixture owner, never Query orchestration. Authored coordinates are exact i64 in
Box2's one declared fixture length unit/Cartesian XY frame. Refinement interprets
Source-projected point endpoints as a closed linear segment; non-point endpoint
inputs refuse rather than silently choose a corner or invent tolerance. This is
fixture linear geometry, not a proprietary CAD kernel/adoption claim. CAD laws
were read from worth-proprietary/docs/house/cad-laws.md; no proprietary code or
production numerical authority is changed. Exact authored values, no floating
projection/epsilon, bounded arithmetic, candidate/contact distinction and no
topology authority govern this path. Each i64 difference magnitude fits u64,
each product fits u128; compare signed product magnitudes to obtain orientation
sign without forming a potentially overflowing signed sum. Independent small-grid
rational clipping plus literal full-width/degenerate cases qualify that claim.
The domain enclosure law exact-match implies candidate-match must be qualified;
complete candidate coverage cannot prove it alone.

## Implementation plan

Intended DX (concrete spelling may follow existing setup lifetime boundaries):

    setup.managed_index_provider::<Index, _>(provider)
        .with_predicate_routing().with_refinement().install()?;
    // Existing managed_index / managed_index_with_predicate_routing delegate
    // to the same installer and keep their existing return contracts.
    let query = ... .managed_index_refined(Index::reference(), probe()) ...;
    let exact = context.refined_managed_index(&source, probe_value)?;

The provider separately implements the row-local exact refinement trait, including
probe validation even for an empty candidate set. Its declaration names domain
identity/version. Existing .managed_index and Native managed_index remain complete
candidates when membership qualifies; they never claim refined/contact truth.

1. Extend portable declaration meaning with an optional refinement descriptor
   and explicit requested candidate/refined selector mode, preserving old Rust
   parts constructors and wire versions as candidate-only. New extension versions
   must carry descriptor/mode and fail closed when missing. Bind actual typed
   Source/probe/index/version at installation; update canonical identity, selection
   recipe/obligation and package archive round trips. Prove legacy/default,
   requested-but-undeclared, wrong contract and round-trip cases with small
   declaration/archive owner tests, no execution roots.
2. Add optional refiner execution and one provider registration builder/installer.
   Existing convenience entrances cut over to that one owner; flags select
   runtime optional adapters, not duplicated generic engines. Forward/inverse/
   refiner share the same P Arc and exact installed object. Missing declared or
   configured required refinement refuses before exact slots escape.
3. In coverage, retain verified candidate cut/bijection unchanged. The framework
   then invokes refinement once per genuine candidate, records accepted/rejected
   dispositions and tracked same-row Source support, and releases accepted slots
   only after complete successful validation/final controls. Retained evidence
   and ReceivedSelection distinguish candidate completeness from exact completion.
   Bind requested mode to actual CoverageFrame.same_invocation. No DTO stamp,
   portable witness or narrowed footprint authority is minted.
4. Thread the explicit mode through ordinary/maintained public selection and the
   existing Native committed/proposed source read owner. The same typed adapter
   runs the exact pass for all of them. Add an explicit Native refined entrance;
   preserve current candidate callers. Inverse routing calls the same row evaluator
   for actual before/after records. Full Parent filter/order/projection follows;
   complete original Source dependencies and original stop categories remain.
5. Extend canonical influence closure to every exact-refined Parent, including
   non-Live one-shot. The exact Source/Parent projection profile remains required.
   Reuse fresh governance and final disclosure; hidden inputs without permitted
   membership influence refuse before callbacks. Live additionally needs Live
   influence. Keep continuation unsupported until its separate qualification.
6. Implement and independently qualify prefix exact-name and closed segment/box
   fixture mechanics. Opt existing maintained providers into the optional contract
   while preserving their candidate/enclosure queries and rules. Add explicit
   refined Parents and an actual Native exact-name rule for 'sha', separate from
   the old reserved 'taken' prefix rule. Prove actual candidate false positives,
   candidate-true/input-edit exact transitions, closed boundary/degenerate cases,
   current/proposed/committed agreement, typed interruption/unavailable and genuine
   watch B/C delivery. Compare to independent authored/reference membership;
   qualify enclosure law. Reuse actual existing hosts and SDK roots.
7. Run pure numerical/declaration/archive feedback first, then one focused public
   refinement court build and affected existing candidate/governed/watch families.
   Use existing Rust1.99.1, sccache/shared target/default flags; never stop/restart
   a live build. No model-root unit fixtures or broad large growth reruns merely
   for this leaf. Run mandatory boundary/context,400-line dirty caps, function
   scrutiny, whitespace/UTF-8, compiled docs and independent qa-loop/qa-tests/
   code-quality-qa. Update guide/capability/spec evidence and commit only the
   integrated proof-bearing vertical, not unused scaffolding.

This slice does not close continuation qualification or Phase4. Full Source
support is deliberately conservative. Private CAD adoption and later phases stay
required; no epsilon, topology operation, private numerical parser or freshness
registry is introduced.

## Implemented slice and evidence

The optional contract and explicit selector mode are bound through the one
canonical declaration/selection/installed/obligation path and shared archive
codec. Older candidate layouts remain candidate-only. One registration builder
composes optional inverse/refinement callbacks over one actual provider Arc;
legacy convenience setup calls delegate to its single installer. Required
refinement executables and full Source/Parent profiles fail closed at actual
installation. No second provider registry, Source owner or generic engine was
introduced.

The coverage owner verifies the original cut/bijection first, retains the full
candidate array and all exact bool dispositions, then allocates/seals the exact
accepted count. The first real false-positive court caught an IncompleteSeal
from allocating candidate capacity for accepted output; this was repaired by
materializing from already sealed decisions, without repeating the evaluator.
Frame/receiver mode guards prevent mode relabeling. Domain/provider/decode and
original control stops retain their existing categories and custody.

Public and Native explicit refined reads use this same typed adapter. Inverse
routing reads each real retained Parent's mode and invokes the same callback on
candidate-matching before/after rows. Whole projected rows and complete original
Source dependencies are conservative support, not individual access traces or
realized/narrowed footprints. Prefix callback reuses the existing invocation's
with_text owner; fixture observers run after releasing their mutex.

The same maintained catalog adds explicit exact-name and segment-contact Parents,
non-Live governed exact/blocked influence twins, and the separate real Native
AtMostOneExactShaName law. Its original checked read loans yield committed and
proposed counts in one typed domain refusal with the actual scope; there are no
global cross-scope observation counters. Existing reserved-name and two-AABB
laws retain their candidate meanings. The geometry owner validates point
endpoint regions and implements fixed-width exact contact with no tolerance.
No private CAD code, pin or adoption claim is included.

Focused evidence on the default Rust1.99.1/sccache/shared target:

- Consumer exact-refinement courts:4/4,1.68s after21.00s final incremental build;
  initial integrated default test build2m46s. Public/Native same-scope predecessor
  and proposed truth, candidate false positives and stable-candidate exact
  transitions, equal-probe distinct mode watches, exact typed violation/unchanged
  selected commit, empty candidate domain validation, fired late cancellation and
  lawful retry, missing executable Contributions denial and lawful configured
  twin, and non-Live hidden influence rejection before provider use all pass.
- Domain numerical courts:3/3,0.00s after0.66s build. Independent rational slab
  oracle over finite endpoint/probe grid, reversal, exact implies enclosure,
  literal full-i64/closed/degenerate cases, non-point refusal and lawful twins.
- Declaration malformed/missing exact mode contract:1/1,0.00s after9.74s build.
- Real archive managed-index owner tests:7/7,0.00s; versioned member/selector
  round trips, missing/truncated/refused contracts and legacy conservative forms.
- Existing candidate/watch/regression receiver courts:6/6. Three public/governed
  watches10.82s; prefix Source publication1.05s; indirect geometry1.85s; complete
  candidate interruption0.14s. No large growth reruns or new SDK roots.
- Compiled consumer documentation:1/1,0.01s. Boundary/context checks pass.
- Dirty400-line caps:73 Rust files pass. Function scrutiny:38 advisories,0 scan
  errors. The candidates/refinement orchestration remains one cohesive original
  decoder/control/Source session; many-parameter advisories carry explicit actual
  frame/request/installation owners. Existing long fixture journeys are one
  finite actual registered root; no helper-specific runtime root was added.
  Remaining advisories are existing cut/installation/codec/session orchestration,
  reviewed within this causal scope; no unrelated dirty-worktree debt absorbed.
- Whitespace and strict UTF-8 audit pass. Final independent qa-loop,qa-tests and
  code-quality-qa gates PASS after the final evidence refresh.

No loop build-role tool is available in this session; actual default Cargo was
used. No live build was killed/restarted, no flags/features/profile/target/cache
path were changed, and request-root tests remain in existing consumer courts.
Phase3 continuation and Phases4-6 remain required.

Final reviewer clearance explicitly covers all three QA skills. Its guide
mojibake finding was corrected by restoring untouched guide text from HEAD and
reapplying only focused UTF-8 edits; added-line encoding audit reports zero
suspicious double encodings. Final guide diff43+/6- and compiled examples pass.
The last oracle strengthening requires a genuine sha-other entrant to deliver
to the candidate watch while the same-probe exact posting stays Pending. The
refreshed4/4 court run passes and the reviewer reconfirms all gates. Final
existing receiver regressions remain6/6 (watch family10.63s, Source1.05s,
geometry1.86s, interrupted complete candidate0.14s). All78 changed text files
pass strict UTF-8; dirty73 Rust caps and38 function advisories remain reviewed.
