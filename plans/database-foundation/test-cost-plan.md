# Test-cost census and prebuilding

Scope: root, Query, Bank and Store. worth-ui is excluded. This implements Track B
step B3 under milestone-3's verification tiers; it changes test execution, never
Store's durability, recovery, authority, or sweep membership contracts.

## Boundary review

Cargo metadata and compiler JSON own package, artifact and diagnostic identity.
The runner owns compilation and prerequisite lifetimes. A running court consumes
prebuilt executables and compiler outcomes; it may not acquire the package lock
or create another build graph. Fixture diagnostics must be attributed by Cargo's
target identity, never by process exit alone. A missing prerequisite is a typed
refusal, never success. The source scanner reports candidates; only resolved
compiler invocations enter enforcement.

Existing competing compilation paths are trybuild sessions, Store's
`independent_executable` first-use builds, Store's per-fixture compiler boundary
driver, Proof's band-guard specimen builds and Bank's metadata call. The latter
two preserve real dependency-tree/IR and metadata observations respectively.
The Store certification-absence fixture requires a separate production graph.

The lane inventory must start from all discovered tests, ignoring default
filters, and compare every nonignored identity against the complete partition
set. Ignored tests stay visible as scheduled-only and are never executed here.
Legacy logs lack artifact/CPU/setup data: absence of measurements cannot become
a zero, a passed budget, or permission to remove a landing lane.

## Implementation sequence

1. `scripts/test_cost/` owns standard-library census, retained command logs,
   Cargo artifact accounting, lexical source sites, inventory multiplicity,
   compile-size inputs and non-gating CPU/setup telemetry. Fixture workspaces
   protect attribution, missing data, duplicate graphs and coverage failures.
2. Merge `codex/m3-courts` before execution-path changes. Establish shared
   prebuild/result reading in the existing gates, with complete Cargo JSON.
   Move uniform trybuild lists with a codemod; retain all fixture expectations.
   Prebuild executable prerequisites before any court starts.
3. Enforce no in-test compilation, one compilation per declared compiler unit,
   and exact lane multiplicity. Reserve slots from measured process CPU rather
   than test names. A whole-pool reservation requires measured parallel work.
4. Store: batch compile-fail inputs by required feature environment, preserve
   production/certification claims and lockfile versions, move pass cases into
   ordinary courts, pool only independent sweep cases, and fold formal builds.
   Drop serial workspace execution only after coverage passes. Keep TLC intact.
5. Retain before/after censuses and ranked residue under target/test-cost. Run
   affected nonignored suites once through prebuilt execution, repair moved
   selectors/snapshots, merge courts again, and run required constitution,
   generated-context, format and dirty line-cap checks. Obtain independent
   adversarial review of the final commit range and exact lane logs.

## QA considerations

An unrelated compiler error cannot satisfy a semantic denial. Batched failures
must still compile every fixture and retain successful counterparts. Child
selectors must name moved modules exactly, and process claims still use real
independently compiled executables. Sweep placement preserves every case,
continuation, panic and comparison; adaptive or shared-state loops stay serial.
Timings are telemetry, never merge gates. No optimization profile, system
configuration, Cargo feature, compatibility path or sampling is introduced.
