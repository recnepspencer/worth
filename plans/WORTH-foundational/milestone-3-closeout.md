# Milestone 3 Closeout: Profile And Policy Vocabulary

Date: 2026-05-13

## Status

Milestone 3 is implementation-complete for `worth-foundational`.

The crate now owns the shared profile and profile-driven policy vocabulary for
typed richness, support posture, compatibility posture, admission readiness,
retention/delivery, certification posture, requested/admitted/materialized
progression, canonical profile identity, target-aware attachment,
materialization/elision planning, proof-bearing certification strengthening,
and production-test readiness evidence.

This milestone is ready for production-shaped testing through `worth-harness`
or adopting-crate migration work. It does not claim that any adopting crate has
already lowered its real runtime policy into the foundational profile language
correctly.

## Completed Surface

- Typed profile families now exist for diagnostic richness, support posture,
  compatibility posture, admission readiness, retention/delivery, and
  certification posture.
- One sealed total `FoundationalProfileSet` now carries exactly one explicit
  slot per required family and rejects incoherent composed meaning.
- Requested, admitted, and materialized profile meaning are mechanically
  distinct progression surfaces rather than one mutable "effective profile"
  record.
- Requested-to-admitted and admitted-to-materialized progression use explicit
  narrowing records, explicit narrowing kinds, and structured
  `TransitionOutcome` categories instead of implicit fallback behavior.
- Boundary, support, and proof-bearing attachment targets are distinct typed
  lanes with explicit legality rules and target-preserving profiled wrappers.
- Admitted profile meaning now lowers through Milestone 2 canonical basis law
  and derives canonical profile identity and structural compatibility
  classification from that basis.
- Profile identity remains basis-derived semantic meaning rather than digest
  folklore or materialization-plan output.
- Optional descriptive surfaces are centrally enumerated through closed
  target-scoped inventories for history, replay, lineage, provenance, and
  forensic diagnostics.
- Materialization planning exposes exhaustive, selected, and elision-profile
  entrypoints with explicit materialization cost and typed absence-cause law.
- Support and proof-bearing surfaces can attach reduced-richness profile
  meaning without changing authoritative payload truth.
- Proof-bearing certification strengthening now has explicit evidence-backed
  and production-certified lanes, plus trust-boundary weakening and readmission
  law.
- Milestone 3 production-test readiness now exists as a proof-bearing artifact
  with certified-surface inventory, hostile-pressure inventory, compile-fail
  inventory, runtime assumptions, runtime non-assumptions, residual debt,
  concrete certified-surface evidence rows, and a named `worth-proof` API
  appendix.

## Final QA Fixes

- Strengthened Phase 7 readiness from enum-only summary inventory into a
  concrete readiness artifact with exact certified-surface evidence rows and a
  named standardized `worth-proof` API appendix.
- Corrected a semantic identity leak so canonicalization cost counters no
  longer participate in `FoundationalProfileIdentity` equality.
- Tightened composed-profile law so stronger certification posture cannot be
  claimed without the required support, readiness, and retention commitments.
- Reworked the profile compile-fail gate into responsibility-shaped proof
  groups for family boundaries, set construction, attachment, identity,
  materialization, and certification/readiness instead of one catch-all test.
- Lifted repeated certification profile fixture construction into a narrow
  domain support module so the certification tests read like proof rather than
  repeated tuple assembly.
- Updated one stale canonical export trybuild snapshot encountered during the
  full-suite milestone closeout so the workspace proof surface matches the
  current `worth_proof::AuthorityWitness` path.

## Proof Evidence

- Certification tests cover profile composition, progression and attachment,
  canonical identity and difference, materialization and absence-cause law,
  proof-bearing certification strengthening, and Phase 7 readiness.
- Compile-fail tests prove raw strings cannot satisfy profile-family or
  descriptive-surface APIs, raw collections and unnamed defaults cannot satisfy
  composed profile-set APIs, plain payloads cannot satisfy profiled attachment
  APIs, raw digests cannot satisfy profile identity APIs, illegal target
  inventories cannot be WORTHd, and wrong-strength certified artifacts cannot
  satisfy stronger APIs.
- Blind-consumer style certification tests prove attached profiles, profile
  identities, materialization decisions, and readiness artifacts remain
  interpretable without producer-private state.
- Reduced-richness hostility tests prove optional descriptive suppression
  changes only named descriptive surfaces and does not change authoritative
  payload truth.
- Readiness certification now binds every certified surface to one hostile
  pressure class, one compile-fail boundary, one owning certification test, and
  one concrete trybuild evidence path.
- Topology checks show all touched production and test files remain under the
  400-line cap, and the profile source/test directories remain under the
  10-direct-file cap with responsibility-owned substructure.

## Verification

The final QA pass ran:

```powershell
cargo fmt -p worth-foundational
cargo test -p worth-foundational
git diff --check
```

All passed.

Result counts:

- `4` unit tests passed.
- `136` certification tests passed.
- `23` compile-time boundary test groups passed.
- `0` doc tests ran.

Additional final gates passed:

- profile source/test line caps
- profile source/test direct-directory caps
- readiness artifact evidence integrity
- full-suite compile-fail snapshot integrity

## Explicit Deferrals

Milestone 3 does not implement:

- real adopting-crate profile lowering parity
- runtime policy execution inside `worth-foundational`
- diagnostics ontology
- lineage, provenance, or receipt ontology beyond profile-governed descriptive
  surface vocabulary
- artifact/report/summary/receipt taxonomy
- branch, merge, and commit authority-transition vocabulary
- a universal profile registry, executor, scheduler, storage model, or support
  matrix engine
- Milestone 4 artifact taxonomy or any later roadmap milestone

Those remain downstream roadmap work. Milestone 3 closes the shared profile,
attachment, materialization, certification, and readiness language those later
surfaces must consume.
