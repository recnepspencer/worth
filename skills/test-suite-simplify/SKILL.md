---
name: test-suite-simplify
description: Use when asked to reduce the cost or fragility of existing tests,
  fixtures, snapshots or verification steps. Not for use during a production
  change.
---

# Test suite simplification

Code is cheap. Certification is expensive: builds, slow suites, snapshot
regeneration, review and evidence handling. Spend code to cut certification
time. Success is that the next production change certifies faster.

## Preproduction

WORTH is not in production. Do not preserve obsolete APIs, stored formats,
fixtures or golden bytes for backward compatibility. Within the change's
scope, delete displaced implementations, adapters, migrations and
compatibility-only tests; use one current representation. Recreate development
artifacts instead of migrating them. Preserve required current-version replay,
undo and recovery behavior. Add backward compatibility only when the user
explicitly requests it.

## How to work

- Act; do not propose. Report what you changed afterwards.
- Rewrite before you repair. If a fixture or test is costly to understand or
  keep valid, delete it and write the small direct version.
- Move each check to the cheapest place that can fail for it. Cover variants
  at the boundary that owns them; keep one expensive journey per distinct
  failure mechanism.
- Check with cheap tests as you go. Run expensive qualification only when the
  task or repository policy requires it, after all edits. If a later
  correction invalidates part of it, repeat only that part.
- New tests must be readable on their own.

## The one guard

For each removed test family, state the guarantee preserved and its
replacement, or why no current guarantee remained. The replacement must stay
sensitive to any real defect the family previously caught.

## Remove on sight

- A fixture that re-decides what production decides: policy, authority or
  semantics. Preparing inputs, including its own types, conversions and
  writers, is fine.
- Full-output snapshots. Assert what the guarantee names.
- Hardcoded opaque identifiers (IDs, cursors). Assert the relationship: an
  issued offer is accepted, a stale one is refused.
- Variants multiplied by lifecycle stages behind expensive setup.
- A test that selects a declared outcome and checks it comes back, without
  exercising the prohibited behaviour.
- A test that asserts removed code is absent. Test that the removed behaviour
  is refused, or nothing.
- A measurement whose name claims more than is observed.
- An evidence step with no named risk.
- An expectation kept only because it was asserted before.

## Keep

- Known-answer vectors for computed values (digests, encodings), established
  independently of the implementation. A check where both sides come from
  the same code proves nothing.
- Byte-exact checks where a contract requires the bytes. Each names its
  contract.
- Results, decisions and errors. Do not normalise them away.

## Do not

- Add a fixture layer, framework or helper library. Replacements are plain
  tests.
- Run the expensive suite to check a single edit.
