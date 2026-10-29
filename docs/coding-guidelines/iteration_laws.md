# Iteration Laws

> Iteration speed is architecture. It is decided by the shape of the crate
> graph, the code each edit makes the compiler regenerate, the build
> configuration and the test topology. It is not decided by how fast a machine
> happens to be today. These laws hold by construction, and each is enforced by
> a check that answers the same way on every machine, under any load.

Use with [Testing Evidence Laws](testing_laws.md) (especially 18 and 19),
[Domain Structure Laws](domain_structure_laws.md) and MENTALITY §3. A law
without a check is category 5: hope.

## I. Compilation topology

1. **A crate is the unit of compilation, invalidation and parallelism.** An
   edit recompiles its crate and everything above it; crates compile in
   parallel only when neither depends on the other. Crate boundaries are
   chosen by Domain Structure Laws 1–5, and a crate past the size ceiling is a
   decay signal requiring a split along its next domain boundary or a
   registered exemption with its reason. *Check: production lines per crate
   against the ceiling and the exemption registry.*

2. **Production crates contain no courts.** `src/` may hold `#[cfg(test)]`
   unit tests only for the module's own private logic, and they may not
   construct the application runtime, a session, or any type from a crate the
   module's crate doesn't own. No `#[path]` into `tests/`. Courts live in
   **court crates**: leaf packages that nothing depends on. *Check: source
   scan for `#[path]` test mounts, and a registry of runtime roots that are
   forbidden in `src` test modules; `cargo metadata` shows no production
   dependency on a court crate.*

3. **A court crate is one dependency surface and one test binary.** Group
   courts by the narrowest dependency set they need (declarations, installed
   runtime, wire/MCP, cost), not by file or by milestone. Each court crate has
   `autotests = false` and exactly one test target. Shared fixtures live in
   `*-testkit` crates, compiled once, which production never depends on.
   *Check: the test-target registry: every package's test targets are listed
   with their reason, and anything else fails.*

4. **Edit locality is a design requirement.** A test edit never recompiles
   production code. A production edit recompiles only the court crates that
   depend on it. *Check: follows from laws 2 and 3, verified on the
   `cargo metadata` graph.*

5. **Volatility flows to the leaves.** Code that changes often (features,
   operations, product wiring) sits in crates with low fan-in. Code with high
   fan-in (math, geometry, schemas) changes rarely and only behind stable
   facades. *Check: fan-in × production size per crate, reported at every
   gate; a new high-fan-in dependency on a volatile crate is a review
   finding.*

6. **A doctest never costs its own link.** Executable examples (DX Law 18)
   compile as merged doctests (edition 2024) or as courts in the crate's court
   crate. Per-example doctest binaries are forbidden. *Check: manifest and
   edition scan.*

## II. Code generation

7. **Generic shell, concrete core.** A generic function's body may only
   contain what depends on its type parameters: extract the type-dependent
   facts (constants, descriptors, one erased call) and delegate the rest to a
   non-generic function compiled once. Monomorphization is paid for every
   instantiation, in every downstream crate and court binary. *Check: per-crate
   LLVM IR line totals and the largest instantiation families (`cargo
   llvm-lines`), ratcheted. These numbers are deterministic for a pinned
   toolchain and target.*

8. **A mode that doesn't change the code isn't a type parameter.** Mode,
   strategy and phase markers that only select a branch are values or consts,
   pruned before any deeper generic call. A type parameter must change what
   is computed. N members × M modes of a macro-expanded generic path is a
   quadratic codegen bill. *Check: the law 7 ratchet; review.*

9. **One concrete call site per encoding.** Serialization and similar
   visitor-generic derives are monomorphized where a concrete encoder meets
   them. Each wire type is encoded and decoded through a concrete function in
   its owning crate, never ad hoc across crates. Derives on types that never
   cross a boundary are removed. *Check: the law 7 ratchet.*

## III. Build configuration

10. **One build configuration, declared in files.** Every setting that
    affects artifacts (profiles, per-package optimization, debug info,
    rustflags, linker, features, environment) lives in `Cargo.toml` and
    `.cargo/config.toml`. A raw `cargo` command and the scripted command
    produce identical artifacts. Scripts and agents never export settings
    that change artifacts. *Check: a config scan that rejects
    `CARGO_PROFILE_*`, `RUSTFLAGS` and `CARGO_ENCODED_RUSTFLAGS` in scripts
    and launchers.*

11. **Optimization is a role, declared once.** Every package appears in one
    crate registry with its role: numerical kernel, kernel, glue, UI, court,
    testkit, tool, or external SDK. Each role has one optimization level and
    debug setting, shared by `check`, `clippy`, `build` and `test`, so they
    reuse one artifact. The profile sections of `Cargo.toml` are generated
    from the registry, never hand-edited (MENTALITY §10). In a repository
    with several workspaces, each workspace root carries its registry and
    generated profiles, and a role means the same settings in all of them.
    *Check: a package missing from the registry fails, and `Cargo.toml`
    profiles that differ from the generated ones fail.*

12. **One artifact per crate per profile.** Two builds of the same crate in
    one target dir from profile, flag or feature drift are a defect. Feature
    sets are unified across the workspace. *Check: after each gate, count
    artifact hashes per crate in the target dir; more than one fails.*

13. **Build inputs are deterministic.** No `env!`, `build.rs` or
    `include_*!` that reads time, random identifiers, git state or host paths
    outside a single leaf stamp crate linked only into final binaries.
    *Check: source scan.*

14. **The cache is configuration, not a habit.** The compiler cache wrapper
    and its path-independent base directories are declared in
    `.cargo/config.toml`, so every cargo invocation (scripted, raw, any agent,
    any machine) uses it. Each lane gets its own target dir (writer
    isolation); immutable dependencies are shared across lanes and worktrees
    through the cache; incremental state applies only to workspace crates and
    is swept by policy at landing. *Check: law 19's preflight fails when the
    wrapper is missing, bypassed or overridden; every gate records cache hits.*

## IV. Test identity and topology

15. **A test's category is its path, never its name.** Categories are a
    fixed set of module segments: `cost` (growth, count, timing, large
    scale), `gpu` (needs a device), `oracle` (spawns an external oracle) and
    `determinism`. Everything else is an ordinary court. Every profile filter
    and test group selects by category segment, never by name pattern.
    *Check: over `cargo nextest list`, the filters contain no name regex, and
    a timing call, an elapsed-time read or a 2^14+ size constant outside
    `cost` fails.*

16. **A test name is a behaviour sentence.** snake_case, stating the
    behaviour or invariant protected; no `test_`/`it_` prefix, no milestone
    or ticket names (Domain Structure Law 4), unique within its court crate.
    The full path `crate::area::[category::]behaviour` is its identity, so a
    filter on any prefix selects a coherent family. *Check: over `cargo
    nextest list`.*

17. **Ordinary courts are fast by construction.** An ordinary court builds
    the smallest world that proves its claim (Testing Law 7), shares
    immutable setup without shared fate (Testing Law 9) and never re-solves
    what a fixture can supply. A time ceiling in the ordinary profiles exists
    to catch defects, not to define performance: a court that hits it is
    fixed or proven to belong in `cost`. *Check: the profile ceiling (a
    detector; Testing Law 22 governs real performance claims).*

## V. The loop

18. **Scope is derived, not chosen.** One repository command, `loop`, owns
    iteration:
    - `loop edit` compiles only the crates that own the changed files;
    - `loop court` runs only the court crates and path prefixes that cover
      those crates;
    - `loop gate` runs everything, once, at handoff;
    - `loop where <path>` prints the owning crate, its role, its court crates
      and their prefixes.
    
    Scope comes from the diff, the crate registry (law 11) and the court
    registry (which production crates and areas each court crate covers),
    never from an agent's guess. Agents, briefs and footers say `loop`, never
    hand-composed cargo invocations. A brief may narrow `loop court` to its
    lane's prefixes. *Check: the court registry covers every production crate;
    footers and briefs contain no raw cargo commands.*

19. **Every run checks its own configuration first.** Each `loop` tier starts
    with a preflight of milliseconds that refuses to run when the
    configuration has drifted:
    - an artifact-changing variable set in the environment (`RUSTFLAGS`,
      `CARGO_PROFILE_*`, `CARGO_INCREMENTAL`, `RUSTC_WRAPPER` overrides);
    - the cache wrapper missing or inactive;
    - the toolchain not the pinned one;
    - generated profiles out of date;
    - an unregistered package or test target.
    
    The refusal names the drift and its fix. A tier that runs on a drifted
    configuration is how a workspace silently stops using its cache. *Check:
    the preflight itself, with fixture tests for each drift.*

20. **The right next command is the easy one.** Adding a court is placing a
    module in a registered court crate, with no new manifest or target.
    `loop where` answers "where does this go and how do I run it" in one
    call. Failures print the narrowest command that reproduces them (court
    crate, path prefix, test). The gate prints, for each failure, the `loop
    court` command that reruns just it. *Check: courts on `loop`'s own
    output.*

21. **Enforcement runs at the speed of the loop.** Every check above runs on
    the diff by default and on the whole tree at the gate. Checkers run
    prebuilt or as plain Python, never through `cargo run` inside the loop.
    A check slower than the edit tier is itself a violation.

22. **Gates use deterministic measures; timings are telemetry.** Wall time
    varies with the machine, load and OS, so it never decides a merge. Gates
    use measures that are identical everywhere for a pinned toolchain: test
    target count, artifact count per crate, LLVM IR lines per crate, doctest
    binaries, the crate graph. Wall-time samples of the three tiers are
    recorded at each landing as a trend, and a regression is investigated at
    its structural cause.

## Named failure modes

- **Court-in-production:** test code compiled into a production crate.
- **Binary-per-file:** every test file is its own link.
- **Surface collapse:** courts with different dependency needs share one
  binary, so every edit rebuilds all of them.
- **Profile drift / flag drift / feature drift:** one crate built twice
  because two invocations disagree.
- **Ambient configuration:** artifact-changing settings exported by a
  script or an agent instead of declared in files.
- **Volatile input:** a build reads time, randomness or host state, so
  nothing caches.
- **Mode-as-type:** a branch selector instantiates whole code paths.
- **Instantiation fan-out:** generic bodies copied into every downstream
  crate.
- **Name-regex taxonomy:** categories inferred from words in test names.
- **Timing gate:** a merge decided by wall time on whatever machine ran it.
- **Slow sentinel:** a checker that costs more than the loop it guards.
- **Guessed scope:** an agent picks what to build and run from memory, so it
  runs too much, or too little to be the right evidence.
- **Nominal narrowing:** a `-p` that still builds a monolith, because the
  structure makes the narrow command wide.
- **Silent cache loss:** the cache stops applying and nothing says so.

**AI deprogramming note:** Your default when something is slow is to measure
it, tune the slow thing, and move on. That fixes today and teaches nothing to
the next lane. The question is which law the slow thing broke, and why no
check caught it. Fix the cause, then add the check. Never "verify speed" by
timing a run on your machine and calling it done; the next machine, load or
OS will disagree. Put structural properties in the gate, and keep timings as
evidence of trend.
