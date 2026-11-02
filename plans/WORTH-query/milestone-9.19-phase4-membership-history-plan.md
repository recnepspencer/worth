# Query9.19 Phase4: Native membership authored-history plan

Read the [boundary review](milestone-9.19-phase4-membership-history-boundary-review.md)
first. These artifacts precede Rust edits for this slice.

## Intended result

The actual Native runtime can issue a descriptive authored-history selector for
one registered membership-only Source program and full-generation scope:

```rust,ignore
let selector = runtime.authored_managed_source_selector(execution, &program, scope)?;
let mut history = runtime.authored_managed_source_history_cursor(
    execution, &original_basis, selected_target, &[selector],
)?;
let page = history.next_page(&runtime, execution, limits)?;
```

Use the existing cursor/page lifecycle; declarations above sketch the new Native
metadata entrance, not Query READ or correction DX. Nonempty fields/routes refuse
explicitly. Foreign actual programs/rosters refuse despite equal descriptions.

## Module responsibilities

- validation/custom_registry plus a narrow Source history roster module: actual
  immutable catalog, profile eligibility, deterministic local slots and selector
  issuance. No address ordering or inventory digest.
- validation/candidate_source/authored_history/: enumerate original lifecycle and
  relation intents (including no-ops), observe actual before/after incidence,
  select installed programs through existing postings and reuse scope_closure.
  Retain only paid coordinate arrays. Do not change normal provider-maintenance
  source_changed or invoke providers for no-op history recording.
- history/data/canonical_commit_envelope/live_managed_source_history.rs: private
  live packet, exact final envelope Arc association, deep-clone absence, explicit
  wire/Eq/digest exclusion. Native source interpretation retains the prepared
  packet until the actual artifact handoff; it is not a public proof constructor.
- Existing artifact/publication phases: move/share prepared live metadata through
  their actual source-interpretation custody; freeze only the final assembled
  envelope. Check ordinary single-parent/physical/schema posture; unsupported
  cases leave a typed historical gap rather than fabricated empty coverage.
- Existing authored_writes generation plus relevant_writes selector/cursor/page:
  derive optional local-slot/scope postings, retain exact roster interval coverage,
  verify canonical packet association at consumption, and preserve original
  paging/continuation/admission behavior. Missing acceleration stays visible.
- Existing relational managed_source court: actual transaction/receipt/basis
  tests over its existing factory and Source owner, mounted in its current target.

## Ordered implementation and evidence

1. Introduce the catalog-local sealed selector vocabulary and runtime issuer.
   Verify profile and actual program/roster affinity; keep ordinary selector key
   behavior unchanged. No portable reconstruction of the live selector.
2. Capture membership-authored incidence from merged intents and resolved record
   allocations at the actual Native candidate. Use actual metadata before/after,
   original paid work and complete inverse Source discovery. No-op endpoints must
   be included despite absent semantic effects; unrelated field writes omitted.
3. Attach live metadata to the actual canonical artifact and final Arc. Deep Clone
   and wire readers produce absence. Preserve exhaustive envelope field
   classification and canonical bytes/digest. Check owned errors and lifetime.
4. Extend the existing authored generation and cursor/page with exact interval
   roster checks and managed-scope postings. Generic rebuild retains typed gaps;
   ordinary reads never rebuild silently. Page refusal cannot advance position.
5. Add real court twins with exact emitted commit IDs: original interval empty,
   actual no-op endpoint authored occurrence, transient add/remove, shared row and
   A/B rewiring, irrelevant other scope/field omission, foreign equal program,
   unsupported profile/interval, discard/rebuild/missing or parsed metadata and
   original interruption/continuation where changed. Use existing canonical
   envelope/codec courts for deep-clone/roundtrip anti-authority checks as needed;
   do not substitute hand-issued Source packets for real runtime success.
6. Update milestone/capability/schema docs with the precise Native membership-only
   history contract and explicit missing full-field/Query correction integration.

## Verification and independent gate

Use Rust1.99.1, configured sccache, C:/worth-build-cache/target and default
flags/features/profiles. Build the existing Native court once after code and
independent early QA settle:

```text
cargo test -p worth-court-relational-certification --test worth_court_relational_certification --no-run
```

Run the new child and existing relational::managed_source:: tests directly from
its cached binary. Run focused existing authored-history/envelope owner tests
only when necessary for changed protocol boundaries; share one owner build.
Run affected Query consumer regressions through the existing application build
wrapper once for the final Native change, plus the relevant maintained/correction
lanes. Preserve scoped compilation and documented unrelated baseline exclusions;
no new geometry-scale claim without its actual court.

Run required Cargo boundary-check and agent-context commands, dirty formatting,
line caps, function scrutiny, UTF8/C0/added-mojibake and diff checks. Split touched
files before exceeding400 lines. Save actual evidence and obtain unconditional
independent qa-loop, qa-tests and code-quality-qa clearance before committing.

## Explicit follow-ons

Full Source scalar/whole-aspect and projected-endpoint authored support requires
its own intent-based classification and real profile courts. Query correction
then must filter relevant canonical occurrences, require existing cancellation
and compare actual paired support. This slice cannot remove its existing managed
historical refusal or close Phase4. No governed producer API or private pin here.

## Early independent placement refinement

Keep the ordinary public RelationalAuthoredWriteSelector Eq/Ord surface unchanged.
The managed entrance issues a separate opaque live selector without Ord and uses
its named admission method above. Both entrances validate then lower once into a
single internal PostingKey (ordinary descriptor or local program slot/scope),
sharing the existing generation/cursor/page lifecycle. Roster authority is never
ordered or equated with local posting coordinates. This replaces the temporary
public-enum managed variant scaffold before any build.

## Conservative impact-candidate precision

The existing Native pullback intentionally inverse-pulls affected row endpoints
through both Source images. Thus touching A->x can include shared B even when only
A's direct edge was authored. This slice records a complete conservative Source
impact-candidate superset, not exact per-scope relevant-write history. The new
managed entrance's Rustdoc must distinguish that contract from ordinary exact
selectors. Its court includes shared B as appropriate and omits disjoint C.
Original authored lifecycle/incidence coordinates remain available for later
Query/support filtering. Candidate occurrences alone never prove relevance or
cancellation. Do not add a competing precision traversal to relabel this stage.

At final publication the actual Source interpretation is already retained. Move
its prepared coordinate packet into the final canonical-envelope freeze there,
using a separate bound-header allocation paid from the original request. Only
that final envelope owns the self-Weak; prepared metadata carries no self-Weak.
Deep Clone clears the live wrapper. This avoids intermediate artifact plumbing,
full-plan copying and accidental weak custody in discarded Native interpretations.

The immutable single-parent generation records its greatest missing managed
metadata ordinal rather than adding a per-gap tree: a nonempty interval refuses
exactly when that greatest gap is at/after its start (or roster continuity fails).
Packet coordinate backing uses original-funded immutable ordered maps; their cells
retain their own grants, and no unrelated Source rows are enumerated.


## Journal court placement correction before fixture implementation

BC9002 refused a new runtime court under Native src. The existing source baseline
is migration debt, not permission to add another runtime case there. Remove that
uncommitted fixture and keep runtime/journal execution in the existing external
managed_source court. A narrow byte-backed fixture implements the public durable
medium/cold-source ports: actual canonical and transition bytes, strict next
position/identical retry, bounded reads, no checkpoints or fake READ authority.
Reuse the existing Source registry/Reader/SourceOwner. Exercise a genuine scope
deletion, then move the selected head past it, release outcome/snapshot holders
and use public root reclamation. Actual bytes must be reacquired by Native before
the same continuation and explicit rebuild refuse missing live metadata.
Deep Clone clearing is a simple private representation rule reviewed statically;
actual decoded gap behavior is the public anti-reconstruction evidence. No new
source runtime court, testkit migration, target, feature or public test door.
