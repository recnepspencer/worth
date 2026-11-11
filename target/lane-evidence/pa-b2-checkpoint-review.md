# Independent B2 checkpoint source review

Verdict: **APPROVE** for the scoped source migration. No material findings.
This is not final landing or static-gate certification.

Reviewed against base `e4d3b01c8d32ae3135a4dc40aadd322f8e3906e1`: all six
modified existing Rust files, three deleted Rust files, and the seven new
checkpoint/native checkpoint court files, together with their existing public
fixtures, production authority entry points, moved-court additions, and baseline
deletions. The unmounted `association_checkpoint_journeys.rs` is outside this
review. Read AGENTS.md, Landing 2a, the B2 plan and claim disposition, relevant
engineering/testing laws, and QA Loop / QA Tests / Code Quality QA instructions.
Implementation conclusions were not used as evidence.

## Findings

None. No supported severity finding remains in this source scope.

## Material comparisons

| Risk | Source evidence and assessment |
| --- | --- |
| Lost graceful recovery semantics | Four Native capture/restore cases, two exact branch-target/content cases, five branch lifecycle/transition journeys, two checkpoint-tail/index cases, and three recovered-transition authority cases retain the original public assertions. Selected heads, versions, snapshots, lifecycle posture, and published index generation are observed through existing public owner surfaces. |
| Counterfeit persistence or runtime identity | Continuation receivers are constructed with attachments from the explicitly held original `Arc<TestDurableMedium>` before recovery. Reopen sources come from that medium. No post-construction rebinding/discovery or substitute local provider is introduced. The foreign-runtime-name refusal intentionally starts a separate receiver medium, as its original did; it proves the exact mismatch class and zero installed commits. |
| Weakened authority/refusal behavior | Exact successor/predecessor admission, foreign authority/candidate denials, returned-authority retry, stale-candidate/source-movement denials, unchanged basis descriptors and commit counts remain. The successful transition's single original durable artifact is observed on the actual held medium. |
| Hidden loss of private or injected assertions | Exact image/Root sharing, original partition backing and retained-envelope census, and published/preparation snapshot census portions have same-base-SHA C2 restore references. Both scripted transition roots have same-base-SHA C4 references and name their fixture closure, exact scripts, fate, retry, and no-effect guards. |
| Owner configuration and topology | The builder-invariant unit now uses genuine Memory, while retaining the exact rule ID and unsealed configuration assertion. Production changes only remove a test mount. No facade, dependency, hook, test-only door, suppression, feature, or adapter is added. Baseline changes only remove the three exact rows for deleted files. |

The old `checkpoint_durably` helper performs a logical checkpoint followed by
default Embedded Native capture/filing. New `capture_durably` performs the same
logical capture and genuine filing using the existing TestKit external-member
capture. `file_checkpoint` retains the actual New member bytes and checkpoint;
recovery reads them from the original medium. The preserved claims concern
coverage, receipts, admitted branch contents, and original index identities;
none promises Embedded placement. The changed placement does not weaken those
claims. Exact allocation/identity claims are separately retained in C2.

## Evidence and limits

- Inspected the existing public focused log: 16 selected tests passed, zero
  failures, exit code 0 (`pa-b2-checkpoint-public-fixed.log`).
- Ran no Cargo commands or builds and edited no Rust.
- Ran `scrutinize_rust_functions.py --dirty .`: four advisories, zero scan errors.
  Reviewed each. Three migrated scenario functions remain legible sequences of
  fixture/action/observation; the fourth is the unchanged lineage court body in
  the shortened owner file. No structural finding follows from these lengths.
- Attempted the dirty line-cap shell guard; it reached its enforcement banner
  but did not provide a completed result during this source review. The owning
  session retains responsibility for caps, formatting, boundary/context gates,
  and remaining qualification. No pass is claimed here.
- Native builder-invariant verification was still running when this source
  review concluded. Its required successful result and final scoped gates remain
  conditions of implementation completion, independent of this source approval.

**APPROVE**
