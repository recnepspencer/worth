IN PROGRESS. Reviewed source HEAD `5c495d7e5d`. The final documentation commit
contains this report and evidence handoff; no push.

Commits, each with the requested Co-Authored-By trailer:

- 8745bcbb1b — retire bootstrap/proposal frames: prepaid returning Native
  candidate/validation and Query Native/Bridge/service publication phases.
- 60f851e1e2 — compare complete recorded Native receipt meaning; migrate both
  consumer retries to RecoveredCommitted, preserving Native changes, output,
  drift and no-reexecution. No recovered live result/witness.
- 97f0fc97d2 — compose Candidate through the existing typed fixture factory,
  retaining its distinct schema, program and membership. No alias/baseline door.
- a7e917ecd4 — exclusive serving/retiring/retired/released Bridge custody and
  backed route/sealing phases, retaining retired meaning and field drop order.
- 5c495d7e5d — keep an advancing retirement task borrowed in its owner through
  typed refusal and observer unwind; consume only after Complete.

Runtime evidence below is under target/lane-evidence/. Every log records the
exact command, HEAD, dirty status, times and exit. Counts are pass/fail/ignored;
ordinary entries count executions. Commands/selectors are also in plan/audit JSON.

| Lane / command | HEAD | Counts | Log |
|---|---|---|---|
| Ordinary bins/examples: cargo build -p worth-query-consumer-root -p worth-query-topology-entry --bins --examples; owned standalone execution |97f0fc97d2|1/3/0|main-entry-round1-consumer/{build,worth-query-consumer-root,invariant_reads,required_feature,shared_execution}.log|
| Same four ordinary entries |a7e917ecd4|1/3/0|main-entry-round2-consumer/same filenames|
| Owned Bank nextest, exact assessment/transfer parents, widths 0/1/2/5 |97f0fc97d2|2/0/1|main-entry-bank/run.log|
| Same two parents |a7e917ecd4|2/0/1|main-entry-bank-final/run.log|
| Owned nextest: twelve affected Candidate/managed provider names |97f0fc97d2|12/0/0|main-entry-consumer-courts/run.log|
| Owned nextest: same_directory_reopens_reads_and_commits_without_replaying_initializer_or_old_key |a7e917ecd4|1/0/0|main-entry-home-final/run.log|
| Same Home case after unwind correction |5c495d7e5d|1/0/0|main-entry-home-unwind/run.log|

Bank's ignored event is an unexecuted scale-probe metadata event. Both selection
audits admit exactly two nonignored parents. No ignored, query-scale, exhaustive
oracle or scenario test ran. Prior pinned provider evidence is not represented as
a whole-source rerun after the later layout changes.

Compiler/gate lanes (counts N/A; compilation is not test execution):

| Command | Logged HEAD/status | Final evidence |
|---|---|---|
| cargo check --tests -p worth-relational, Root manifest |04e727e98b dirty|main-entry-round1-native-check{,-inner}.log:0|
| Query clippy --lib -p worth-query-execution -- -D warnings |04e727e98b dirty|main-entry-round1-query-strict-corrected{,-inner}.log:0|
| Consumer/topology clippy --all-targets -- -D warnings |04e727e98b dirty|main-entry-round1-consumer-strict-corrected{,-inner}.log:0|
| Query clippy --tests -p worth-query-execution -- -D warnings |97f0fc97d2 dirty|main-entry-round2-query-strict{,-inner}.log:0|
| Query clippy --lib -p worth-query-execution -- -D warnings |5c495d7e5d clean|main-entry-unwind-query-strict{,-inner}.log:0|
| cargo fmt --check for all three touched crates |5c495d7e5d|main-entry-fmt-{native,query,consumer}.log:0|
| bash check_workspace_rust_line_caps.sh dirty; diff 04e727e98b |5c495d7e5d|main-entry-caps-{dirty,diff}.log:0; 31 Rust files|
| cargo run boundary-check -- --root . |5c495d7e5d|lifecycle-main-entry-boundary.log:1; 15 excluded diagnostics|
| cargo run agent-context -- check |5c495d7e5d|lifecycle-main-entry-context.log:0; no generation needed|

Failures and corrections: wrong-workspace Native test selection omitted Root dev
dependencies (158 diagnostics); the Root invocation corrected our command. One
new comparison used the wrong Native facade (E0425); corrected to durability.
Shared fixture visibility caused E0603; corrected at the owner, without caller
permissions. Original failed logs retain their uncorrected command names. A
PowerShell quoting error prevented a queue script from being created; its
round2-runtime-batch.log records exit2 before Cargo; the v2 driver is preserved.
Our new retirement extraction lost an inner task on observer unwind; 5c495d7e5d fixes
the source. The real Home case passes, and borrowing structurally retains that
task; no injected-panic test is claimed.

OPEN P1, integration: all three real examples still exit0xc00000fd on their genuine
1048576-byte main reserve. The two authorized rounds are spent. Complete final
paths: main-entry-round2-*-frames.json and main-entry-stack-handoff.md. Mutation
preparation and output-demand/Signal execution are the next whole-path targets.
Diagnostic census logs are observations, not passing courts; PDB source-line
resolution failed and is not claimed.

Boundary15 remain coordinator-owned: P-A six Native BC9002; P-C two Native
transition, three World BC9002 and four Store BC7013. Candidate is absent from
the findings. Baselines/permissions are unchanged. UI metadata lock churn was
saved and restored byte-for-byte; net UI diff is empty.

Fresh independent review of04e727..5c495: REJECT, one P1 (three main failures),
agreed OPEN; no other material finding. Full review: main-entry-independent-review.md.
No broad certification was run; earlier held ledgers remain in force.

Turn counts: 2 cargo check +6 clippy =8;7 focused builds (2 ordinary,5 nextest);
5 named nextest runs;8 ordinary program executions;6 diagnostic re-executions.
Also2 owned Cargo tool runs,3 final fmt checks and2 line-cap checks. No third
main-stack build, cache cleanup, branch switch or push.
