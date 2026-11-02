# 9.19: borrow-confined decoded Source text

## Boundary review (HEAD 67af5e7750)

The actual completed or maintained Source image owns occurrence identity and
governance. SourceRows is constructed only inside build, candidates, and update;
before and after update rows are distinct immutable images even when their slots
are equal. SummarySources admits the current region before SourceRows.get. Its
parent walks remain unchanged in this slice.

Projection owns required-field omission/disclosure and exact installed slot
contracts. Its projected field already carries the installed Arc slot key. The
declaration owner can compare a borrowed selector contract to that key without
allocating four new strings. Query/slot/value portable identities, presence,
scalar family and all locus/output names remain part of equality. TypeId is
only the installed decoder's nominal verification, never an ordering key.

Installation owns immutable binding descriptors; its catalog currently owns
descriptors directly under one Arc map. Share each actual entry with Arc while
preserving outer installation identity and borrowed APIs. ValueDecodeSession
continues to own installed codec selection, sticky first refusal and the
original cumulative parse reservation. A decoded String must not escape that
reservation through a public shared handle.

The current SourceRow alias cannot lend invocation-owned text. Replace it with
a privately constructed row receiver holding the actual projection and its
SourceRows memo. Ordinary projection methods stay available by Deref. Current
provider consumers use the public SourceRow spelling; no expanded type callback
requires a compatibility lane. Prefix build comparisons, summary comparisons,
bounds tests and matches_source all currently allocate owned Strings repeatedly.

## Implementation

1. Add canonical borrowed required-field equality at the declaration key owner.
   Projection uses the actual projected key on exact matches; the requested-key
   fallback preserves omission precedence and existing mismatch diagnostics.
   Factor required-field admission so ordinary decode and managed lending share
   one semantic path. Keep optional fields and relation behavior unchanged.
2. Retain actual installed descriptors as Arc entries. Add admitted shared
   entity-field lookup by cloning the existing entry only after real admission;
   keep borrowed lookups, iteration and descriptor const methods. Expose narrow
   private session operations for shared binding acquisition and registered
   decode; preserve unsupported legacy paths and first-refusal behavior.
3. SourceRows owns invocation/image-local SDK maps: canonical projected key to
   installed descriptor, and (issued slot, canonical key) to one-element shared
   SDK String array. All map operations, inline backing and parse storage use
   the original execution request/session. No snapshots/Provider state/global
   cache, borrowed-descriptor RefCell, names-only key or uncharged deep clone.
4. Add SourceRow.with_text(selector, |text: &str| ...). Each use checkpoints,
   admits the actual field/disclosure, and verifies the nominal binding even on
   a warm hit. Clone only the SDK array handle out of the RefCell before invoking
   the callback. Poll again afterward. The callback cannot return its borrowed
   text. Receiver construction remains closed; no cache handle is public.
   Propagate a narrow public text-read denial (projection or controlled stop).
5. Cut over every current prefix text path to nested lending, including the
   candidate predicate. Keep the pure comparator and actual SourceSlot tie.

Example: `row.with_text(query::name(), |name| name.starts_with("al"))`.
For two-key comparisons, keep the first borrow alive through a nested read of
the second row. Never copy text into an adapter-owned cache.

## Evidence and gate

Start with public small consumer courts, then the existing managed family and
Candidates profile. Add nested same-slot nonempty backing identity and nested
two-key reads, a warm nominal mismatch negative, cancellation/refusal after
population with original-payer cleanup and fresh lawful retry, and a real
public compile-fail lifetime escape with compiling lending twin. Preserve the
existing UTF-8/equal-name oracle, shared-X before/after rename, and genuine B2
out-of-region negative; exercise the new lending read in that hostile path.
Check catalog shared entry identity and independent installation behavior in
the nearest existing real installation court. No fabricated Source loans.

Use default Rust 1.99.1/sccache/shared target, one build at a time, never restart
a build. Run formatting, scoped courts, boundary-check, agent-context, dirty
Rust caps/function scrutiny, diff whitespace, and compiled-document drift.
Independent reviewer applies qa-loop, qa-tests and code-quality-qa before commit.

Out of scope: ancestry memoization, large prefix qualification, page batching,
spatial grouping and churn repair. Audit item 4 remains open until ancestry and
the promised scale evidence are also complete. No whole-query linear claim.

### Verified storage correction before continuing

The first compiler check exposed an incorrect boundary assumption: ExecutionArray
is move-only for every T, and explicitly documents Arc<ExecutionArray<T>> for
sharing. Do not change SDK semantics or clone String payloads. Use a private
Arc<TextBacking> containing the one-element array and its separately retained
original-request reservation. Before Arc allocation, quote Layout([usize;2])
extended by the actual TextBacking layout (including the reservation), just as
the existing ProviderNodeBacking owner does. Reserve through the original
ExecutionRequest.reserve_retained_memory. Array and parse grants stay separate;
only the private Arc handle is cloned. No public handle can outlive decode scope.

## Implemented evidence

The former SourceRow alias is now the closed lending receiver. Build,
summary-bound comparisons and matches_source use nested with_text calls; ordinary
ProjectionRow field reads and managed reads share required-field admission.
Canonical borrowed equality lives at the declaration key owner. Catalog entries
share their actual Arc descriptors; no copied recipe or secondary catalog was
introduced. SourceRows owns separate before/after image memos. Each decoded String
is moved into a one-element SDK array under a private, separately paid Arc header;
its parse reservation remains with the original ValueDecodeState. Per-use original
checkpoints, disclosure/field contract and nominal Rust binding checks remain.
Controlled text errors propagate as Controlled through all prefix adapters.

Accepted evidence (default Rust 1.99.1, sccache/shared target, no build overrides):

- Pure declaration key equality: 1/1, varying every field-key axis, Required,
  Optional, OptionalAspect and non-Field rejection against owned-key equality.
- Existing installation compilation family: 8/8. Shared lookup yields the actual
  borrowed entry, repeated lookups each admit, original lookup refusal survives,
  wrong binding and independent installation identities retain prior behavior.
- Focused real maintained court: nested simultaneously live nonempty text loans
  have identical backing; a different output selector is refused and a lawful
  same-invocation read still succeeds. A lookalike canonical key with a different
  Rust binding is refused after warming the actual value. Original cancellation
  after population is swallowed locally but still denies final handoff. Both
  denials release the original payer to its baseline; the same actual rename,
  runtime/head and idempotency key commit with a fresh request. A/B before/after
  values remain exact and no cold rebuild supplies the successful retry.
- Final public managed-index family: 49/49, 777.16 seconds. This deliberately
  records the accidentally included long unchanged geometry case, rather than
  restarting it. Existing UTF-8/equal-name and actual B2 out-of-region controls
  exercise the new lending path. Separate Candidates family: 9/9, .81 seconds.
- The included public SourceRow documentation was tested with rustdoc against
  the exact current consumer-built execution/declaration rlibs: owned-result
  twin 1/1 and caller-row-lifetime escape compile-fail 1/1. A standalone rustc
  metadata check of the extracted negative reports only `lifetime may not live
  long enough` at `row.with_text(selector, |text| text)`. The fence does not merely
  demand a static borrow and requires no Source factory or private-owner build.
- Generic live-grant disclosure smoke: 1/1 with the repository's Cargo-configured
  runtime environment. An initial direct-executable run omitted the configured
  64 MiB test stack and overflowed; the accepted rerun applies the actual config,
  not a build/profile override.
- Mandatory Cargo boundary-check and agent-context, formatting, whitespace,
  dirty Rust caps (32 files), function scrutiny (21 advisories, no scan errors)
  and compiled document drift all pass. No new Source factory exception.

The broad run also reconfirmed the original geometry guards without editing them:
1,000 `(2194388, 2114916)`, 2,000 `(2329336, 2247073)`, 2,000+50,000
`(3393472, 3294818)` for refusal/commit accepted original SDK work. Its complete
Source census is outside that mutation invoice. These are geometry evidence,
not a prefix scale or text-only cost measurement. Logs are
`C:/forge_workspace/query919-text-{key,installation,small,managed,candidates,lending,escape,disclosure-configured,boundary,context,functions}.log`.

Ancestry reuse, small real repeated-extrema cost evidence and large prefix
qualification remain next; audit item 4 and Phase 3 remain open. No universal
sublinear initialization, constant empty query, or whole-query linear claim.

Independent reviewer gave unconditional qa-loop, qa-tests and code-quality-qa
clearance on this final code/evidence/docs set. The 21 function advisories are
coherent scoped owners or inherited functions; no unrelated refactoring or
additional build is required for this slice.
