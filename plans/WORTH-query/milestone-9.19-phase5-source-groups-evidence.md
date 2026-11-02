# Query 9.19 Phase 5: Native Source preparation groups evidence

Status: complete prerequisite; independent qa-loop, qa-tests and code-quality-qa PASS.
Phases 5 and 6 remain unfinished. Baseline `9923e50002` is pushed master.

Boundary and plan:
- [Boundary review](milestone-9.19-phase5-source-groups-boundary-review.md)
- [Implementation plan](milestone-9.19-phase5-source-groups-plan.md)

## Actual change

Native freezes a single immutable Source preparation group roster from the
unchanged real invariant registrations. A group contains exact installed program
Arcs, declared access and maximum-work metadata, without a rule callback or
identity. Source-empty registrations are omitted without changing nonempty order.
Postings index group slots; history catalog, authored impact preparation and
pre-invariant staging consume that same roster. Real rule inventory/index/get/iter
and construction/recovery/fork behavior are unchanged. No new public registration,
portable meaning, budget, READ, provider cache or Bank query is introduced.

The two preparation algorithms change only metadata types/accessors/local names.
Independent static review confirmed original first affected group ownership,
sibling invocation-scope union, program affinity, require_access, original leased
work meter, empty completeness and participant/seal handoffs are preserved.
Existing external fixtures are single-program groups; passing them is not a new
runtime claim for multi-program or shared-program first-owner combinations.

## Measured verification

All logs below live under `C:/forge_workspace/` with prefix
`query919-phase5-source-groups-`.

- `native-build.log`: first build PASS, Cargo 25.52 seconds, existing two unused
  import warnings outside dirty scope; target inventory remains 93 tests.
- `native-tests-0.log`: actual substring `relational::managed_source` matches
  13 Source courts plus all three existing process/work-account/support-growth
  cases: **16/16 PASS**, harness 1.33 seconds.
- `native-tests-1.log` and `-2.log`: the two planned exact original-invoice and
  inherited-cancellation/installed-allowance courts pass again; these are repeats,
  not additional distinct tests.
- `native-tests-3.log`: genuine compound Source publication/failed-staging release
  **1/1 PASS**, 0.02 seconds. External total **17 distinct PASS**.
- `owner-build.log`: first build PASS, Cargo 43.10 seconds; existing owner warnings
  retained; actual inventory remains 2250 tests.
- `owner-tests-0.log`: actual rule registry duplicate/inventory/lookup **3/3 PASS**.
- `owner-tests-1.log`: actual schema-bound program interning/foreign recipe
  **2/2 PASS**.
- `owner-tests-2.log`: authored-write-history **19/19 PASS**, 0.74 seconds.
- Total **41 distinct scoped runtime passes**; repeated process tests not counted.
- Actual Cargo `boundary.log` PASS (Cargo 0.03 seconds) and `context.log` PASS
  (Cargo 0.09 seconds), using configured original target/profiles/features.
- Rustfmt and dirty 400-line guard PASS for seven Rust files. Function scrutiny
  has five pre-existing algorithm/parameter advisories, zero scan errors; no
  algorithm expansion or unrelated cleanup. Strict UTF8/C0/added-mojibake and
  `git diff --check` pass. Final text/cached scans repeated after this evidence.

No Query/Bank/geometry/50k build was run for this private derived-metadata cutover.
No tests, allowance ceilings, public facades or portable bytes changed. All work
uses Rust 1.99.1, configured sccache, `C:/worth-build-cache/target`, original
profiles/features and runtime stack 67108864; no build/cache was killed or cleaned.

## Next obligation

Explicit public maintenance still needs a genuine declaration policy/version,
Native Source registration and receiving-registry affinity, complete installation
and reconstruction transport, Source roster receipt/bootstrap/recovery binding,
then real Bank/topology receiver adoption. Old Bank v1 pinned behavior must remain
executable. This prerequisite does not close any public or private acceptance gap.

Independent reviewer verified all seven Rust files and three documents, build and
runtime logs, 41 distinct passes, unchanged scope/owner algorithms, no stale
Source-registration callers, five existing scrutiny advisories and strict text
checks across all ten dirty paths. No blocker remained to this slice commit.
