# K1c slice 3: field-bound Decl and Host content door

Status: implemented and verified with documented untouched lint debt. The same
independent reviewer passed the complete slice under qa-loop, qa-tests and
code-quality-qa before commit. Slices 1 and 2 are committed at
c878b6928ea65b27dc62cc977e09441d0b109841 and
52f8df312bc702bd4af909cb86a53177bb09e06c. Before this slice, both sync branches
were fetched; master c1b656df11c7ebc4e429facb18ee3f73e2c4b2e3 merged cleanly at
3553dc320ee7665707db0860da2fe44bdb18dbd8. The provider branch was already current.
The same independent reviewer gates the completed slice before its commit.

## Boundary review from the actual code

Decl and Installation do not depend on Execution or Native. Their immutable
field recipes and portable schema records describe meaning and bind typed
provenance; neither can own byte custody or open a Native request. Keep that
direction. Add descriptive format/field declarations and controlled callback
interfaces there, with executable erasure retained only in typed provenance.
Query's existing schema_layout owner lowers those callbacks into the installed
Query-free Native RelationalContentCodec. The adapter is a concrete engine with
function pointers, rather than a generic engine instantiated per application.

Native already owns frozen codec handles, typed canonical frames, original-funded
attachments, per-root lookup, transactional closure and recovery. Query may retain
these actual Native values in an attempt, but cannot create a byte directory or
cache. Existing schema layout lowers actual installed aspect contracts. Content
fields need a full field-meaning codec and an explicit alias to their declared
format identity in that same Native registry; a bare ContentRef field cannot
silently publish without an installed format.

CandidateWriter currently delegates typed writes into effect_program, which seals
the read set plus effects and lowers them through provider_binding into staged
provider candidate material. provider/invariant_execution/relational_validation
then opens the genuine Native transaction and validates it. Carry actual Native
content attachments through these same owners, with the ordinary mutation, before
Native validation. Preserve the actual request and all existing operation, target,
field, creation and capability checks. No new effect permission follows from an id.

The existing pre-publication domain_value_write::admit runs after effect expansion
and before provider publication preparation. Its canonical-domain predicate must
also select content field contracts. Extend this one owner to ask Native whether
each present reference resolves against the actual basis plus staged attachments
under its installed format. The final Native transaction still closes all actual
fields before Source/invariants and publication. There is no parallel Query content
publication validator.

DecisionReader::field records actual operation-scoped field reads. The operation
projection reader retains the selected product/basis and original execution. Its
content extension first performs that declared field read, then asks Native to
resolve through that exact basis. None means genuine permitted absence; no missing
frame is an optional value. Required-field and parent-aspect obligations remain.
A typed admitted content view retains the real immutable Native frame and lends
its canonical payload, without introducing a second decoder or numerical owner.

The installed schema and package archive already preserve portable members and
canonical field meaning. Extend them with named format declarations and the
field's format reference, not executable callbacks. A recorded descriptor cannot
construct a typed callback or Native installed handle. Archive readmission checks
against the actual configured typed recipe. Preserve old bytes when the optional
content extension is absent; no old numerical ContentRef identity is resurrected.

## Plan and concrete developer result

The application declares one format and any number of fields using that format:

```text
impl ApplicationContentBinding for SampleGridFormat { ... }
let samples = content_field(GRID_SAMPLES, SampleGridFormat);
ApplicationSchemaDeclaration::builder(...)
    .content_format::<SampleGridFormat>()
    .content_field(GRID, samples)
    ...
reader.content(&identity, samples)?;
writer.initialize_content(&created, samples, &grid)?;
writer.write_content(&existing, samples, &changed_grid)?;
```

The format binding supplies immutable identity, ASCII name, nonzero version,
complete frame ceiling, payload length, encoding, canonical validation and an
independent input correspondence check. Encoding receives one bounded encoder
that also exposes the original checkpoint. This combines sink and control into
one borrowed interface so Rust never aliases two mutable borrows of Native's
original control. Validation/length/correspondence receive its read/control port.
The adapter restores the exact Native first-stop cause after callbacks, including
a callback that swallows its stop. A fabricated stopped outcome while the real
control is live is an internal inconsistency, never a guessed execution cause.

1. Add Decl content binding, typed field reference, recipe and builder support in
   named application_schema/content modules. Portable format declarations and field
   references enter canonical schema meaning and archive records. Refuse duplicate,
   missing, mismatched or untyped format installation. Prove these with focused
   declaration/installation/archive courts and constructor/type-boundary checks.
2. Lower actual typed recipes into one concrete schema_layout/content codec adapter
   and register field aliases through Native's existing registry. Retain the same
   actual format handles for writing and Native validation. Add no hash owner.
3. Extend CandidateWriter's two field-bound operations and the attempt's Native
   attachment custody. Stage encoding plus reference atomically; failure exposes
   neither a dangling authored field nor an attachment. Carry attachments into the
   actual provider Native transaction before validation, with no policy-only copy.
4. Extend DecisionReader and ordinary operation-projection acquisition through the
   actual Native basis. Preserve field observations and typed Native/read/authority
   denials. Extend the existing domain_value_write owner for content contracts and
   staged/basis resolution before provider effects. Reject missing, foreign-format
   and unsupported-version references with the selected commit unchanged.
5. Land a public Decl/Host sample-grid consuming court: exact field/value roundtrip,
   original cancellation and projection-budget refusals, bad encoder and correspondence
   fault, raw reference refusal, and optional/required/read/write authority fences.
   Native admission/custody courts provide original memory and frame-size refusal
   evidence in this slice. Public-boundary memory/size qualification remains slice 5.
   Expected
   dimensions, order, signed words and literal bytes come from an independent court
   oracle. Source/correction integration remains the accepted next slice.

No new AspectShape form, InlineBytes change, Store edge, graph constructor,
freestanding public put, generic engine body, or public cost parameter is planned.
The complete frame ceiling remains 16 MiB even for legal larger control counts;
admission refuses before attachment or effects rather than segmenting or converting.

## Verification

Use only affected packages, their focused court binaries and filters, and Query's
manifest. Set WORTH_IMPLEMENTATION_BUILD_ID to
11111111-1111-1111-1111-111111111111 for Query courts. Long cargo commands run in
hidden background processes with logs and completed exits. Run touched-package
fmt --check and clippy --all-targets, boundary-check, agent-context, dirty line caps,
and diff checks before commit. Measure the produced worth-query-execution rlib if
new typed facade wrappers grow generic code; keep actual engines concrete and
resolve synchronization conflicts in favour of the non-generic form. Do not run a
workspace command. The later Source slice supplies its own full consuming candidate
and correction/history courts; this slice does not claim that qualification.

## Placement and exactness refinements

The typed field binds only the platform's transparent ContentRef scalar binding;
a custom scalar encoder must not alter the identity after independent content
encoding. A runtime wrapper using another format still has to match the installed
field's exact typed format recipe. Portable field meaning carries an optional
format identity; absent extensions preserve previous canonical field bytes.
The existing full-field meaning owner derives the Native codec, and Native's
schema snapshot records its explicit alias to the format. The layout installs
codec adapters into Native; subsequent encoding and reads resolve the actual
selected Native basis, rather than retaining a second Query format authority.

Builder authoring remains physically in declaration/content.rs so no builder
storage visibility expands. The pure descriptor and callback vocabulary remains
in application_schema/content. Existing correction-output declarations that do
not carry content format meaning explicitly refuse a content field until the
accepted Source/correction slice extends that complete contract; they never
compare an incomplete old value shape as though it were complete.

### Handoff refinement found during independent review

Query's installed recipe and Native's retained registry each certify their own
contract. Neither alone proves that the two contracts are equal. A single Native
frame checker therefore compares the expected ASCII name, version, exact frozen
frame ceiling and actual complete-frame length under the original request. The
Query read wrapper, staging before attachment, and existing domain-write seal
consume that check. The Native directory and read closure use the same checker.
A genuine retained-frame court changes name, version and ceiling, including a
frame below both ceilings, so a size-only check cannot pass. Name or ceiling
contract disagreement is `FormatMismatch`; version is `UnsupportedVersion`;
actual oversize remains `SizeLimit`.

Installation behavior is exercised in its existing integration court because
Decl is a foreign owner. No new private Installation unit-test lane remains.

Raw ContentRef fields without a declared format also enter the existing domain
write seal. A present write refuses `FormatBindingMissing` there, before Native
publication preparation. The public court writes a genuine retained identity to
such an optional field and checks the exact denial and unchanged selected commit.

### Initial installation composition found by the public court

The first runnable public court refused `FormatMismatch`: Native's existing
`RelationalSchemaRegistry::extend` copied kinds/domains/sources but silently
omitted the new content registry. Repair that same composition owner. Native's
content registry composes actual frozen handles, refuses duplicate format ids,
name/version coordinates and field aliases, and regenerates its portable snapshot.
No callback or descriptive reconstruction substitutes for the actual handle.
The public create/read/write court is the end-to-end falsifier; a focused Native
composition court checks duplicate refusal and preservation of both registries.

The writer's acquisition/session/staging engine will also be concrete. Typed
facades supply an exact monomorphized binding-acquisition function pointer and
borrowed input. The concrete content staging owner runs the decode session and
Native handoff; the existing generic builder only retains its typed target and
sticky outcome. This follows the concurrently landed generic-shell/concrete-core
cutover rather than putting another engine inside `prepare_content<C>`.

## Readings I chose

ContentRef remains value plumbing when declared without a format, but that value
cannot become an authored domain content reference: the existing publication seal
returns `FormatBindingMissing`. No new compatibility decoder or fallback store is
introduced. Ordered numerical content always uses the content door, including
small payloads; the 16 MiB complete-frame ceiling is an admission refusal boundary.

Format contract equality includes the exact declared maximum even when both
ceilings admit the current bytes. Actual-size overflow retains its separate exact
`SizeLimit` denial. Metadata equality never substitutes for the actual installed
Rust binding/value provenance or for Native's byte admission and custody.

The temporary managed-Source content rejection is explicit until slice 4 lands
its typed callback and actual Native content dependency. This slice does not claim
Source invalidation, correction or consuming-geometry completion.

## Execution size telemetry

The final public9 build produced
`libworth_query_execution-b7016cf434752ce1.rlib`: **318,207,646 bytes**,
SHA-256 `ecba15068e1d81ce6f6a7a346a8cf4ab5a9b5130112282653747d5267626b9c5`.
This is the repository's unoptimized test-profile artifact, measured after the
concrete engine extraction and successful 12-test public court. There is no
controlled baseline claim. The new codec adapter, staging/session machinery,
observation engine, selected-frame read, Native contract checks and publication
seal are concrete. Typed permission, binding-predicate and wrapping shells remain
generic; cost is not part of a public type or signature.

## Courts and independent evidence

- The existing Native certification binary's `content::` filter passed **49**
  tests, **149** filtered, in **1.05 s**. Its frozen frame and Python/OpenSSL
  digest are independent of the production writer/hash owner. Its cold-read,
  corruption, missing closure, original cancellation/custody and durable-medium
  courts still pass after the handoff and registry composition changes.
- The existing Decl/Host binary's `content_door` filter passed **12** tests,
  **327** filtered, in **0.46 s**. It uses public Decl/Host doors and independently
  frozen exact payload words/digest. It reads back the actual created entity,
  requires the equal write to succeed, catches an encoder's canonical-but-wrong
  samples through independent numeric correspondence, and proves missing,
  foreign-format, foreign-version and unbound references leave the selected
  commit unchanged at the existing publication seal. Swallowed refusal, exact
  type provenance, optional clear/parent removal, original cancellation and
  projection-budget refusal are exercised through the real runtime.
- Installation's `content_` filter passed **4** tests, **19** filtered, in
  **0.01 s**. Its real package admission/build/bind path rejects foreign typed
  provenance and missing declarations; it verifies the complete field-meaning
  alias and the explicit temporary Source-content fence. The managed-parent
  profile filter separately passed **4**, **19** filtered, in **0.02 s** after
  the two existing lint corrections in those touched files.
- The existing package-archive schema-record filter passed **10** tests,
  **76** filtered, in **0.00 s**. Its independent 280-byte wire vector checks
  format/field metadata and fresh readmission; ASCII and missing format mutations
  refuse. Old frozen schema bytes remain covered by that same filter.
- Decl content rustdocs passed **1** positive (**27** filtered, **0.01 s**) and
  **2** negative (**68** filtered, **0.10 s**) contracts. Execution content
  rustdocs passed **2** negative (**100** filtered, **0.18 s**) contracts:
  undeclared read permission and forged admitted content fail compilation.

Native's required second all-target Clippy run reports exactly one error in
untouched `mvcc/retained_entity_selection/payload.rs:34`, `clippy::let_and_return`.
The same body is present in `origin/master`; no scoped dirty change touches it.
The earlier caused `clone_on_copy` finding in content recovery was corrected.
This follows AGENTS.md's explicit untouched-debt exception, without suppressing
or modifying that unrelated file. Full unsuccessful-run evidence remains in the
content evidence ledger.

Query's final all-target Clippy run completed with **64 errors**, all in untouched
Decl/Host history/workflow court files. The diagnostic path set has **zero**
intersection with the scoped dirty file set. They include disallowed thread
creation/spawning and existing assertion/loop/drop style findings. No scoped
suppression or unrelated source edit was made. Exact messages and locations are
retained in the evidence ledger. The production Execution library and its test
compilation reached completion without the earlier missing member initializer.

The direct Native court `--all-targets --no-deps` lint pass reached the court and
reported **6** existing errors in untouched managed-Source court files. No new
content court diagnostic was reported. Exact messages/locations are in the
ledger. Both its diagnostic paths and the Query lint diagnostic paths also have
zero intersection with this branch's committed diff against `origin/master`.

## Independent reviewer findings and closure

The same lane reviewer independently read the production path and courts using
qa-loop, qa-tests and code-quality-qa. Findings closed before the slice gate:

- Separate Query and Native certificates lacked equality at the typed handoff.
  One original-controlled Native contract checker now seals all handoffs; the
  same-id genuine registry court falsifies changed name, version and ceiling.
- Installation integration inputs initially sat in private unit tests. Those
  tests were removed and moved to the existing public Installation court.
- A successful create alone did not prove `initialize_content`, and unchanged
  identity alone did not prove the equal write succeeded. The court now reads
  the actual created entity through the typed reader and requires write success.
- An unbound reference test initially lacked the existing completed field
  observation. Its declaration and decision now record that ordinary read, so
  the typed refusal is proved at the intended publication seal.
- Acquisition/staging, field lookup and observation machinery initially lived
  partly in generic methods. Concrete runtime contexts now own these engines;
  typed shells pass exact predicates and borrowed facts. Final compile/runtime
  checks cover that extraction, including the corrected lifetime thunk.

The public court additionally exposed Native schema composition silently dropping
content registrations. The actual Native registry extension now retains the
frozen handles and aliases, with duplicate refusal; the separate composition
court and successful public runtime court close that integration defect.

No hard rule was bypassed. Source acquisition/correction and bounded-cost
qualification remain the explicitly accepted next slices; this commit does not
claim them complete. Untouched lint debt is reported under the repository's
explicit scoped exception, with no unrelated modification or suppression.

The review's final documentation finding was that plan step 5 had promised public
memory/size refusal coverage while this slice's public court proves cancellation
and projection budgets. The plan now states the actual split: existing Native
admission/custody courts prove memory/frame bounds here; public-boundary
memory/size and growth qualification remain slice 5. No public-boundary result
is inferred from the Native court.
