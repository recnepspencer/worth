# WORTH Signal DX Phase 5 Review

## Outcome

Phase 5 is complete.

The main goal was to make runtime policy feel like one guided story instead of a
pile of equally respectable knobs.

That is now the shape:

- pick a stock runtime preset first
- use grouped edit points when you need more control
- keep lower-level and bridge-facing controls available without teaching them as
  the normal path

Just as important, the docs now lean harder toward plain language. This pass
explicitly trimmed wording that sounded more academic or infrastructure-heavy
than it needed to.

## What Changed

### 1. Runtime posture now has a clearer owner

We kept the normal path centered on:

- `SignalRuntime::build_for::<Ctx>(graph)`
- named runtime constructors
- `SignalRuntimePolicy` presets

We also added named builder helpers so the builder path can still read like the
same product story:

- `development_policy()`
- `operational_policy()`
- `forensic_policy()`
- `web_development_policy()`
- `fintech_policy()`
- `kernel_policy()`
- `game_engine_policy()`

### 2. Diagnostics tier reset is now clearly a reset, not a rival owner

The main ownership rule is now explicit:

- `set_runtime_policy(...)` owns full runtime policy changes
- `reset_runtime_policy_to_tier(...)` is the stock reset back to a named tier
- Phase 8 removed the deprecated `set_diagnostics_profile(...)` transition
  path; stock changes use `reset_runtime_policy_to_tier(...)` and full policy
  changes use `set_runtime_policy(...)`.

This removes the old ambiguity where a caller could think they were only
changing diagnostics detail while actually replacing the full policy bundle.

### 3. Advanced tuning is grouped instead of scattered

We added grouped adjustment entry points so advanced tuning can stay powerful
without feeling messy:

- builder:
  `adjust_runtime_policy(...)`, `adjust_checkpoints(...)`,
  `adjust_fallback_comparator(...)`
- runtime:
  `adjust_runtime_policy(...)`, `adjust_tier_policy(...)`,
  `adjust_checkpoint_policy(...)`, `adjust_fallback_comparator(...)`,
  `set_domain_checkpoint_barrier(...)`

This keeps the advanced story practical:

- runtime policy owns runtime posture
- `TierPolicy` owns tier behavior
- `CheckpointPolicy` owns checkpoint behavior
- comparator layering stays recipe -> tier -> runtime fallback

### 4. Product docs now teach the simpler path first

The runtime, diagnostics, comparator, and history docs now teach:

- start with presets
- use the guided runtime/history surfaces first
- only reach for deeper controls when you really mean it

The wording also moved toward general-audience language where we could do that
without making the API descriptions fuzzy.

## Verification

Ran:

- `cargo check -p worth-signal`
- `cargo test -p worth-signal`

Result:

- `482 passed`
- `0 failed`
- `23 ignored`
- doc tests passed

## Intentional Non-Goals

Phase 5 did not try to delete every advanced or bridge-facing type.

That was intentional.

The point of this phase was:

- clearer ownership
- better guided paths
- plainer docs
- less duplicate-looking policy control

Not:

- flatten every advanced capability
- hide real power from people who need it

## Follow-Through

If we do a future cleanup pass, the best next move is not more runtime-policy
work. The next move would be broader naming and docs cleanup across older
specialist surfaces that still use heavier internal wording.
