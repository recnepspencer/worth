# Affected Root normal checks

The opt-in affected path derives a bounded Root request from a clean committed
edit. It uses the existing court ownership, compiler capture, native test,
rustdoc and source-check owners. It reports partial coverage for a manual Root
landing decision. It installs no hook and grants no push permission.

After fetching and integrating the intended base by meaning:

```sh
python3 scripts/ci/query_gate.py --changed-since <full-origin/master-commit> --out <new-output-directory>
```

The base must equal the fetched `origin/master` and be an ancestor of HEAD.
The command refuses dirty, hidden or untracked inputs, unsupported changes and
an existing output directory. It checks the same source and fetched base during
and after execution. A fresh application-build identity reaches every child;
old output and old execution are never imported.

An ordinary `worth-execution` source edit derives that owner and its declared
`worth-court-execution` leaf from real member manifests and `COURT_OWNERS`.
Metadata resolves actual native targets; dependencies and compiler inputs retain
their existing owners. Unrelated production units are excluded. Nonignored
process, oracle, cost, determinism and compile-fail categories remain selected
within the derived owners. Ignored/scheduled scale tests are not forced. A
required court without an active ordinary case refuses. Missing, failed, extra
or repeated observations cannot qualify.

Selected libraries get scoped doctests using their actual library names. Strict
Clippy checks the same selected package roots with their declared dev profile
and default features. No affected native, Clippy or doc request promotes itself
to workspace scope. The selected packages, unselected Root packages and actual
active test packages remain visible in the plan and selection outputs.

The first input contract covers Root `src/**/*.rs` and `tests/**/*.rs` with an
existing forward court association. It also admits the exact normal-tooling
owners and associated pure protocol tests listed in `query_gate_normal.py`,
plus this guide when its provider changes. Such tooling-only adoption is marked
`SOURCE-ONLY / NO RUST COURT COVERAGE`; it runs no native tests or Clippy.
Unknown manifests, locks, feature/config changes, fixtures/assets, arbitrary
docs/scripts, nested workspaces and testkits without forward ownership refuse.
Query, Bank, Store and other workspace extensions require their own contracts.

Source checks retain existing formatters, line caps and the constitution checker.
The normal command uses already-prepared checker executables and refuses missing
or stale tools. It never bootstraps them implicitly. After explicit dependency
cache preparation, separately admitted first-use bootstrap is:

```powershell
pwsh -NoProfile -File scripts/prepare-constitution-hook.ps1
```

Both bootstrap builds are locked/offline. Normal children are offline; Clippy
and scoped docs also pass explicit locked/offline arguments. Native preparation
retains the existing runner arguments and compiler-unit checks. Any changed lock
or other source input prevents a source-bound result. Bootstrap and cache cost
must be reported separately and included in the first-use total.

Each suite keeps its existing JSON result. `summary.json` binds the exact plan,
commit/tree, fetched base, fresh build identity, output location, required
commands, partial coverage and run-artifact hashes. Assess it at the existing
manual landing boundary:

```sh
python3 scripts/test_cost/landing_plan.py --repository <checkout> --normal-result <output-directory> --expected-commit <tested-HEAD> --expected-base <full-fetched-base>
```

An assessment is evidence for Root's explicit decision, not a full Root landing
qualification, a token, a branch-policy change or authorization for an untested
merge commit. Root still checks the fresh base, performs the existing ordinary
PR procedure and inspects the postmerge result; expected PR HEAD alone does not
bind the base atomically.

Legacy explicit suites, current-workspace defaults, `--full` and the complete
`root_gate.py` wrapper retain their existing behavior. They cannot be combined
with `--changed-since`. This first path has no measured cold/warm speed claim;
real execution and measurements require separate host/target admission.
