# Iteration architecture migration

Status: planned. This document authorizes no claim that `loop`, generated
profiles, active sccache, court packages, or subsystem extraction already exist.
The first production owner selected for migration is `worth-relational`.

## Purpose and governing sources

Make iteration cost follow the responsibility being edited. Establish the
shared build foundation, then migrate one production owner and its subsystems
and courts completely before selecting the next owner. A narrow passing
scenario does not close a crate migration.

The governing [Iteration Laws](https://github.com/recnepspencer/worth/blob/f1fc6d368a/docs/coding-guidelines/iteration_laws.md)
were introduced on public master at `f1fc6d368a`. Use them with
[MENTALITY](../../docs/coding-guidelines/MENTALITY.md),
[Architecture](../../docs/coding-guidelines/arch_laws.md),
[Domain Structure](../../docs/coding-guidelines/domain_structure_laws.md),
[Testing](../../docs/coding-guidelines/testing_laws.md), and
[Performance](../../docs/coding-guidelines/perf_laws.md).
Bring that governing commit into the implementation branch through its normal
integration path before implementation; do not replace active branch work to
obtain a documentation file.

This cross-runtime plan owns the migration process and shared infrastructure.
The [Relational cutover](../WORTH-relational/iteration-cutover.md) owns the first
concrete destination. The [Windows setup guide](../../docs/development/windows.md)
owns contributor setup. Query milestones retain their product requirements;
this work does not reopen completed phases or add product semantics.

## Adversarial constraint and completion unit

Several worktrees may edit different subsystems while a cold gate runs. A court
edit must not recompile production; an index edit must not select unrelated
schema-only courts; another worktree must not overwrite this lane's writable
artifacts. Missing cache configuration, profile drift, stale tools, and an
uncovered crate must be diagnosed before an expensive run. Splitting a binary
must not multiply schema-generic code generation or weaken authority tests.

The first complete migration takes a Relational source or court edit through
registered ownership, preflight, owner compilation, affected courts, and the
handoff gate. Its observable result is the correct selected compilation graph
and unchanged production behavior. Branch publication, exact observation,
retention, recovery, and cleanup remain real production operations throughout.

Prerequisites are global configuration, working sccache, package and court
registries, prepared checkers, and `loop`. They precede subsystem extraction.
Completion then requires the entire Relational destination, all moved courts,
codegen ratchets, caller cutover, old-path deletion, documentation, and review.

## Evidence motivating the work

Source inspection on 2026-10-28 used public checkout `bd68c7b5c6`; these are
structural observations, not measurements of current elapsed time:

- `worth-relational` contains a 696-file, 106,662-line `src/tests` tree in
  addition to runtime courts embedded beside production owners. These physical
  source counts include test code and are not production-size measurements.
- Relational already separates mutable state into named runtime subsystems,
  but they still share one production compilation unit.
- Query's workspace manifest documents schema-generic certification expansion
  exceeding Windows PDB limits. Court extraction must account for downstream
  instantiation, not just count test files.
- Root, Query, independent tools, and private consumers have different manually
  maintained profile tables. An external SDK is not automatically compiled with
  the profiles of its own workspace.
- `scripts/ci/query_gate_environment.py` conditionally injects sccache and
  `scripts/application_build.py` issues a fresh build UUID per invocation.
  Neither arrangement establishes one ordinary configuration for raw Cargo.
- The local Windows parent Cargo configuration shares a writable target across
  worktrees. This is machine configuration, not a portable repository guarantee.
- Boundary-check embeds caller lists; agent-context locates templates through
  its compile-time manifest directory. Prebuilt tools need explicit current-root
  inputs before they can safely serve another checkout.

## Destination tree

Proposed paths below are implementation targets, not existing commands:

```text
rust-toolchain.toml                    exact audited compiler and components
.cargo/config.toml                    wrapper, target isolation, stable flags
config/iteration/
  roles.toml                          role settings and structural ceilings
  packages.toml                       root-workspace package membership/roles
  courts.toml                         test targets, coverage, path prefixes
  tools.toml                          pinned setup tools and checker inputs
  exemptions.toml                     explicit structural migration exceptions
  codegen/                            per-toolchain/target/config baselines
workspaces/<owner>/config/iteration/   that workspace's package/court registry
loop.ps1                              Windows spelling of the loop command
scripts/iteration/
  loop.py                             dispatch and exit status
  preflight.py                        environment/configuration refusal
  packages.py                         manifests and Cargo graph ownership
  profiles.py                         generate/check profile sections
  selection.py                        diff-to-compilation/court closure
  checks.py                           scoped/prepared checker selection
  artifacts.py                        compilation-unit and cache observations
  report.py                           results and narrow reproduction commands
docs/development/windows.md           Windows prerequisites and setup contract
plans/WORTH-relational/iteration-cutover.md
```

Use plain Python for the dispatcher and cheap checks; reuse Cargo and nextest
for building and running. Do not create another executor, proof ledger,
custom compiler cache, or test-discovery language. Existing Rust checkers remain
their semantic authorities and run as prepared binaries with explicit inputs.
Each file above has one responsibility; do not add a generic utilities module.

## Global setup, in dependency order

### G1. Pin configuration and register the complete package graph

Consume the exact qualified stable toolchain from
[Query 9.18.1 Phase 18](../WORTH-query/milestone-9.18.1.md#phase-18-mandatory-current-rust-toolchain-qualification).
Rust 1.99.x is the required qualification floor; that phase also requires
qualifying and adopting a newer official stable release if available when
implemented. The observed Rust 1.94.0 CI/local configuration is legacy state,
not this plan's destination. Do not introduce a competing toolchain decision.

If Phase 18 is already accepted on the integration source, reuse its exact
release/full compiler commit and accepted qualification. Otherwise complete its
standard-library sort/bound review, typed unqualified refusal, canonical
ordering/digest compatibility, public correction probe and private SDK proof
before closing G1. Registry/configuration design can proceed alongside that
work, but final profile/cache/codegen baselines use the adopted qualified pin.
Update the workspace toolchain file, CI and private-consumer pins together under
that existing phase's contract. Never bypass the canonical-sort guard merely
to upgrade the compiler or close this migration.

Pin sccache `0.18.0` and nextest
`0.9.140`, both observed installed on this Windows host. Pin `cargo-llvm-lines`
`0.4.46` in `tools.toml`; its declared minimum Rust is 1.86. G1 must qualify its
actual invocation on the pinned Windows toolchain. A floating latest
installation cannot close this phase.

Register every repository package and explicit/implicit test target across the
root, subworkspaces, tools, and nested consumer workspaces. Cargo package identity
includes source/version, not just name. Metadata is generated once per relevant
manifest/lockfile change and reused by the cheap preflight. Cover build, normal,
dev, target-specific, and supported feature edges. Unknown packages and targets
fail closed; a manifest or registry edit invalidates the selection cache.

Each workspace owns its package registry and generated profiles; the root
registry identifies those workspaces. One canonical role definition generates
the same role settings everywhere. Private consumers adopt the same role
contract in their own repository, rather than being scanned from public tooling.

Initial ordinary-build role policy:

| Role | Optimization | Debug information | Intended ownership |
|---|---:|---|---|
| numerical kernel | 2 | line tables | substantive numerical inner loops |
| kernel | 1 | line tables | reusable storage, validation, graph algorithms |
| glue | 0 | line tables | orchestration and composition |
| UI | 0 | line tables | frequently edited presentation |
| court | 0 | line tables | behavioral evidence |
| testkit | 0 | line tables | fixture construction |
| tool | 3 | line tables | repeatedly executed source/manifest checkers |
| external SDK | 1 | line tables | externally owned SDKs with this declared build role |

Generate dev/test settings and any per-package overrides from those roles.
WORTH packages keep their upstream role when consumed from Git or another
workspace; being an external source does not reclassify a kernel or glue package
as an SDK. Private consumers import that role mapping before generating their
profiles. A change of dependency source cannot silently change optimization.
Existing Signal product profiles, WASM release policy, and real perf lanes keep
their named purposes; they are registered configurations, not ambient overrides.
Temporary PDB or size exemptions must name the affected package, structural
reason, and migration that removes them. Never apply a blanket debug-off fix.

Define the initial ordinary production ceiling as 20,000 physical Rust source
lines per crate, excluding parsed `cfg(test)` bodies and separately owned courts.
Count comments/blanks consistently; generated Rust compiled into the crate still
counts. This is this plan's chosen decay ceiling, not an asserted pre-existing
law constant. Keep the existing 400-line file rule. A below-ceiling crate can
still require a semantic split. Report fan-in and production size separately and
as a product; review actual volatile dependencies rather than inventing a
universal numeric fan-in limit.

Existing unmigrated violations remain explicitly visible under the repository's
scoped-debt policy. Exceptions are exact and cannot grow silently. The migration
gate must not report whole-repository compliance while exemptions remain.

G1 closes when registry completeness and profile generation are checked, the
toolchain is pinned, and every artifact-changing setting has a declared owner.

### G2. Make sccache mandatory and prove useful reuse

Declare `build.rustc-wrapper = "sccache"` in repository Cargo configuration.
Disable incremental compilation for the ordinary cached configuration in files,
not through `CARGO_INCREMENTAL`. This first migration has no incremental escape
lane. Rust sccache does not cache incremental compiler invocations; enabling
both without accounting for eligibility would repeat the current mistake.

Declare path normalization in repository configuration, including worktree roots
and the external SDK source layout where relevant. Cargo's config-relative
`[env]` can supply an absolute `SCCACHE_BASEDIRS` without embedding an author's
path. This is not a Rust relocation guarantee: sccache 0.18's Rust key includes
the compile directory and relevant Cargo environment. Verify the pinned daemon
actually uses the intended roots across successive worktrees; do not assume
per-process environment changes reconfigure an already running shared server.
Never restart another lane's daemon as an ordinary fix.

Use one writable target per worktree/lane. The repository default is its local
`target`, overriding inherited parent target sharing. Subworkspace/tool targets
are explicit children with one writer each. An optional external target must
have declared lane ownership in local configuration; it cannot be a common
mutable directory. A second writer queues for that lane or uses another lane.
The persistent sccache store is shared and lives outside disposable targets.

Keep cache location/capacity separate from semantic compiler settings. Do not
silently remove safety flags, change features, or alter native libraries to
obtain a hit. Raw Cargo and `loop` resolve the same effective compiler settings.

Prove cold eligible compilation stores results, then compile the unchanged same
source/configuration into a separate target and observe cache retrieval. Also
qualify immutable dependencies from two differently named worktree roots, where
their own source paths remain stable. Report relocated first-party library misses
separately; `SCCACHE_BASEDIRS` alone cannot be used to assert those are reusable.
Use a registered small library and disposable, explicitly owned verification
targets; never clean active targets. If stronger relocation support is required,
qualify a supported pinned cache version/configuration before changing the pin;
do not patch cache keys or weaken tracked inputs.
Run this qualification serially enough to attribute cache-stat deltas. A warm
Cargo `Fresh` result is not a sccache hit. Normal runs report hits, misses,
non-cacheable reasons, and errors without demanding a hit on every invocation.

Metadata-only checks and final links are not promised to be cacheable. Neither
zero hits on an ineligible run nor a cold miss is by itself a defect; a bypassed
wrapper, unexpected eligibility loss, or failed same-input qualification is.
G2 closes only with the actual cache proof and documented Windows setup.

### G3. Make volatile inputs and checkers reusable

Move random/time/git/host stamping into a single leaf stamp package linked only
by final application binaries. Preserve the semantic distinction between an
implementation build identity and runtime admission authority. Audit current
`WORTH_IMPLEMENTATION_BUILD_ID` consumers and fresh-ID regression tests before
retiring script-wide UUID injection. Do not replace required identity with a
constant or accidentally reuse an identity after a meaningful implementation
change. Final stamps may change while unrelated libraries remain reusable.

Audit build scripts, proc macros, includes, and environment reads. Deterministic
tracked source includes and the audited compiler-selection build check remain
valid inputs. Resolve checkout-specific checker configuration/templates against
the explicit checked root, or deliberately key a prepared tool to all embedded
inputs. Source from another worktree must never masquerade as current policy.

Extend the existing constitution preparation path. Build tools once during
explicit setup or after their tracked inputs change; run prepared tools during
the loop. Include manifests, locks, templates, caller lists, and other consumed
configuration in freshness. Do not rely solely on mtimes. Tool-build identity
is ordinary build provenance, not a new certification protocol.

Diff checks analyze changed files and affected rule/dependency closures. Full
checks run at the gate. A changed policy/manifest can require a broader closure;
print why. Avoid rescanning the whole tree to discover every edit's scope.
G3 closes when tools use current-root inputs and edit checks perform no implicit
Cargo build or recursive full-tree scan.

### G4. Ship `loop` and the cheap refusal boundary

The Windows entry is `./loop.ps1`; `python scripts/iteration/loop.py` is its
identical dispatcher. The semantic command is `loop` on every platform. Only
the Windows wrapper and setup are qualified in this program.

```text
loop setup
loop doctor
loop where crates/worth-relational/src/branch/owner_services/basis_port.rs
loop edit
loop court
loop court --area relational::branch
loop gate
```

`setup` prepares tools and performs cache qualification; it never installs OS
components without an explicit setup action. `doctor` checks configuration and
explains its source. `where` prints owner, role, dependent courts, path prefixes,
and any unresolved migration boundary. `edit` compiles changed production or
test owners once. `court` runs their registered affected court prefixes and any
affected permitted local-unit families. An empty test selection is a failure
when evidence was requested. Area narrowing cannot turn a handoff gate green.

Default scope is staged, unstaged, renamed, deleted, and untracked changes plus
the branch diff from an explicitly resolved integration base. Print the base.
Support an explicit base for review/CI; never infer it from a moving remote on
each invocation. Use old and new owners for moves/deletions. Shared testkit,
macro, manifest, lockfile, configuration, and tool changes expand their true
dependent closure. Unowned or unclassified changes require a stated broader
selection, not silent omission. Registry coverage is selection policy, not a
proof that the tests themselves are adequate.

`gate` visits every registered workspace's required checks and supported courts
once, deduplicating shared build units. It retains existing semantic/performance
obligations and explicitly reports unavailable platform/oracle requirements.
Unavailable evidence cannot be converted into an all-platform pass. Scheduled
work and already accepted historical phases are not recursively replayed just
to certify this plan. Print a narrow `loop court` reproduction for each failed
test; preserve exact test identity and return the underlying failure status.

Before compiling, refuse artifact-changing ambient variables and overrides:
`RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `RUSTDOCFLAGS`, `CARGO_PROFILE_*`,
`CARGO_INCREMENTAL`, wrapper/compiler overrides, and undeclared target/features.
Check effective Cargo configuration including ancestors and user config, not
just the checked-in file. Reject cache-disable/recache overrides in ordinary
runs. Name only relevant variables, their setting's source, and the fix; do not
dump the environment. Distinguish storage paths from artifact configuration.

Preflight also rejects missing tools, the wrong compiler/target, stale generated
profiles, unregistered packages/targets, stale prepared checkers, and lane target
collisions. Use cached manifests plus explicit invalidation to keep it cheap;
test refusals with focused configuration fixtures rather than expensive worlds.
Replace conditional wrapper injection and migrate existing Makefile, gate,
hook, CI, and agent-brief entrypoints to `loop` after parity is demonstrated.
No parallel permanent orchestration path remains.

### G5. Register structural enforcement and publish Windows instructions

Enforce production/court dependency direction, target counts, runtime-root use
inside source tests, forbidden test mounts, roles/profiles, source ceilings,
test identity/categories, doctest topology, generic expansion, and deterministic
build inputs. Gate-wide scans complement diff-scoped checks.

Artifact checks key a compilation unit by package/source, target kind,
host/target triple, compiler, profile, features, and effective settings. Detect
unexplained duplicate variants of that unit; do not count stale filenames in
`target` or convict legitimate host/proc-macro, metadata, or test builds.
Check/clippy and code-generating builds can legitimately emit different products;
one configuration does not mean one physical file for all commands.

LLVM baselines include the pinned toolchain, target and configuration, total IR,
and the largest instantiation families. Initial global baselines expose debt;
each migrated crate must reduce identified fan-out and carry its resulting
ratchet. Capture each selected compilation root, including downstream courts:
`cargo-llvm-lines` does not include dependencies automatically, and generic work
is charged to its instantiating consumer. Do not enable an ambient fat-LTO
profile to manufacture a whole-program number. Run expensive IR generation only
for invalidated owners at the proper
tier; ordinary edits read valid prior observations instead of rebuilding the
universe. Timings are trend telemetry in CI/run artifacts, never merge thresholds.

Categories are module segments `cost`, `gpu`, `oracle`, and `determinism`.
Select segment boundaries, not words in behavior names. Ordinary evidence keeps
a defect-detecting timeout. Public compile contracts retain meaningful positive
and negative cases in consolidated compiler sessions. Merged 2024 doctests or
court-hosted executable snippets preserve executable DX; edition alone is not
proof that every snippet merged. Report actual exceptional binaries.

Publish and link Windows prerequisites, pinned tools, effective-config diagnosis,
sccache verification, raw-Cargo equivalence, target ownership, failure recovery,
and the exact loop commands. Mac/Linux setup and native qualification remain
unclaimed until an agent with those systems performs them. Existing CI on
another OS is not a Windows setup proof.

All G1-G5 requirements close before declaring the shared foundation complete or
starting Relational's mechanical package extraction. These can be reviewed
commits, but a partial foundation is not a completed phase.

## Repeatable production-owner migration

Apply this same procedure to Relational and each subsequent selected owner:

1. **Map responsibility and custody.** Trace public entry, preparation, effects,
   observation, recovery, and cleanup. Identify actual subsystems, callers,
   borrowed state, sealed minting, wire types, tests, and generic families.
   Compute dependency cycles; resolve ownership before moving files.
2. **Freeze the destination.** Specify production packages, narrow facades,
   runtime-owned state, court dependency surfaces, testkit placement, forbidden
   edges, source ceiling, and meaningful evidence to preserve. Every old source
   owner has a destination; a generic contracts/support bag is not a solution.
3. **Cut over subsystems in dependency order.** Move code and its callers
   together. Replace broad runtime access with existing narrow capabilities and
   explicit orchestration. Preserve sealed phase progression. Delete replaced
   implementation and re-export only the one canonical type through the public
   facade; do not retain two implementations or add public raw constructors.
4. **Cut over courts alongside their owners.** Keep only permitted private-logic
   tests in production. Move runtime/boundary courts to leaf packages grouped by
   dependency surface, each with one explicit test binary. Move genuinely shared
   fixtures to narrow testkits; a shared runtime testkit must not widen every
   algorithm court. Preserve independent oracles and process-global isolation.
5. **Remove codegen multiplication.** Extract type-independent generic bodies,
   use values for non-semantic mode markers, and centralize concrete wire
   encoding in its owner. Preserve static type/authority distinctions and
   numerical hot loops. Recheck totals across all new downstream court binaries.
6. **Close the whole selected owner.** Update registries, profiles, docs, imports,
   controls, and caller lists. Remove old mounts, targets, fixtures, adapters,
   exemptions resolved by the move, and obsolete commands. Run required `loop`
   tiers and review behavior plus composition and test honesty.

Each slice is reviewable and committed. Only a fully migrated owner gets a
completion claim. An intermediate compiled move is a prerequisite, not a
finished migration or evidence that all iteration laws now hold.

## Required proof and selection of the next owner

For the selected family, prove a court-only edit leaves production units fresh;
a production edit rebuilds only its true downstream closure; and a narrow
subsystem court omits unrelated runtime composition. Preserve real public
authority, outcome, cancellation, partial-effect, recovery and lifecycle courts.
Run meaningful topology/configuration fixtures and actual cache qualification.
Record build-unit changes, links, IR, cache eligibility/results, and wall-time
trend with the supported environment. Do not replace them with exact timing gates.

The final gate includes formatting, scoped file caps, boundary/context checks,
affected docs/examples, all required owner/caller evidence, and the registered
handoff portfolio. Fix findings against the scoped final source. Do not add a
persistent implementation journal or machinery that certifies test evidence.

Relational is first by user decision. After it closes, select the next owner
from actual change frequency, size, fan-in, IR expansion, and repeated build
units. Query Execution is a known candidate; no later migration is automatically
authorized by this priority suggestion. Private SDK adoption is a separate
consumer cutover with its own pinned-source qualification.

## QA considerations

Review must distinguish missing infrastructure from claimed completion, a
compiler-cache hit from Cargo freshness, and test selection from evidence
adequacy. Trace configuration through raw Cargo and `loop`, and authority through
every extracted subsystem. The material risks are widened minting, lost teardown
or process isolation, a generic testkit that reconstructs the monolith, stale
checker inputs, and duplicate orchestration. Windows qualification is required
for Windows claims. OS-specific paths, flags, and setup must never be inferred
to work elsewhere from this machine's result.
