# Independent corrected-candidate review

**APPROVE — the partial five-public-port / four-C2 candidate from b736be6f47.**

No material in-scope defect found. This approves the actual dirty decomposition,
not Landing 2a completion, the complete cold-family migration, or a green full
boundary gate.

## Reading and execution boundary

Reviewed the tracked patch against `b736be6f473f27870e932507bcbc2dcbf5b17687`
with three lines of diff context, the new cold-residency court, and the directly
relevant fixture, external provider, policy, manifest, frontier, and mount owners.
Read AGENTS.md, the coding guidelines, the Milestone 3 moved-court obligation,
and the source-ceiling/frontier provisions of test-convention.md. Applied the QA
loop, code quality, and test review skills. Other review reports and their
conclusions were not used as authority.

No builds or tests were run by this reviewer. Evidence below comes from the
supplied logs and read-only inspection, including small source/metadata/mount
checks. This report is the only file written by this review.

## Findings

1. **Informational — full boundary closure remains outstanding.**
   `landing2a-cold-public-boundary-final.log` exits 1 and reports 14 findings:
   five control/construction seals and nine BC9002 source-court findings. Three
   BC9002 paths are the unchanged local provider family (`cold_source.rs`,
   `cold_source/protected_reads.rs`, and the local durable-medium runtime
   attachment). The other eleven findings are outside this dirty candidate.
   None names the new court or the changed testkit fixture. These files are
   absent from the dirty inventory inspected here. AGENTS.md's untouched-debt
   rule permits a scoped conclusion, but this evidence cannot support a full
   BC-gate or full migration claim. No enforcement baseline, exception, or
   caller list was weakened in this candidate.

No severity P0/P1/P2/P3 correction is required for this partial candidate.

## Claim preservation and causal boundaries

| Claim / failure risk | Inspection and evidence |
|---|---|
| Damaged-member transaction read becomes a wrong-reason failure or loses write refusal | The public port retains the `Region(entity.partition_id)` / `IntegrityMismatch` failure, repaired-provider sticky retry, unchanged provider read count, exact preparation refusal, and unchanged observed main-branch head. The observed head uses the public branch observation contract. |
| Tail replay stops at an unrelated failure or mutates the receiver | The fixture produces a real checkpoint followed by a genuine update artifact. The port retains `ColdStorageUnavailable`, `IntegrityMismatch`, and the empty receiver branch-head assertion. |
| History / stream ports broaden reads or lose sticky-body protection | Both ports retain indexed-receipt no-body reads, bounded reopen observations, no retired-journal fallback, selected-position-only provider reads, successful warm equality, sticky typed damage after provider repair, unchanged non-envelope reads, and unchanged frontier/attached-medium publication observations. |
| Inspection / compiled-artifact family loses a surface or silently changes profile | All five surfaces remain: version summary, commit inspection, recent inspection, recent branch inspection, and compiled artifact. Compiled artifact uses ChipSimulation for producer and receiver; the other surfaces use CertificationCore. Healthy nonempty/warm behavior and exact envelope-position damage are retained. |
| Whole-test C2 classification drops mixed public behavior | Only four entire private mechanism/accounting courts are deleted. The mixed pin release, concurrent reference, and discarded visibility-round tests remain mounted; there is no whole-test move of these mixed claims. |
| Split introduces an artificial runtime/provider identity or hidden adapter | The court and testkit import `worth_relational::facade`; the provider implements that library's public `RelationalDurableColdSource` trait externally. Build metadata identifies the same path package for Native and the testkit/court. No local-to-external provider adapter, runtime lookup, memory-runtime substitution, or private Root export was added. |

The shared source constructor captures original Native checkpoint members and
canonical commit artifacts from an explicitly owned producer
`TestDurableMedium`. Canonical member locators retain SHA-256 and byte length.
It assembles an unadmitted recovery source, with only artifacts after the
checkpoint's covered position in the optional tail. Its existing recovered
fixture delegates to the same constructor and retains its own actual medium.
The moved refusal/envelope courts attach a separate explicitly held receiver
medium and inspect that medium where publication is part of the claim.

The fault posture matches the original fixture: the original local fixture
starts with damaged record-region bytes; each moved court explicitly sets the
external source's matching `damaged` flag. Envelope damage remains independently
controlled by `commits_damaged`. The provider continues to distinguish canonical
member reads from retired-journal reads and returns owned bytes through the
real request-aware port.

The old and new serial fixture policies both use
`SerialMemoryBudget::new(None)`, a fresh cancellation token, and no deadline.
There is no widened fixture allocation allowance or replacement authority path.
The six selected nextest cases reserve three slots apiece; all six stored CPU
records report matching declared/reserved slots, exit 0, complete accounting,
and no observation errors. The configuration retains zero retries.

## C2 and remaining tree

The four exact restore references were checked directly with `git show` and
each contains its named deleted test:

- `publication_cost_distinguishes_new_and_reused_schema_authority` —
  `b736be6f47:crates/worth-relational/src/branch/root_tests.rs`.
- `replay_pin_of_a_cold_partition_is_typed_and_has_no_effect` —
  `b736be6f47:crates/worth-relational/src/tests/durability/medium/cold_isolation.rs`.
- `resident_pin_custody_refuses_unread_mirror_frames_after_exact_source_admission`
  — `b736be6f47:crates/worth-relational/src/tests/durability/medium/cold_pins.rs`.
- `selected_branch_views_preserve_cold_probe_and_demand` —
  `b736be6f47:crates/worth-relational/src/tests/durability/medium/cold_invariants.rs`.

Their assertions concern private publication allocation identity/accounting,
resident-only selected-state probing, physical replay-pin witnesses and counts,
or the private mirror replay-pin demand/custody mechanism. Milestone 3 records
the restore obligation rather than retiring them. The earlier request-root
allocation court is already part of the base and is not an additional deletion
in this candidate.

Traversing the actual remaining `cold_members.rs` module graph found 80 source
files and 79 module edges with **zero missing targets**, including the provider's
protected-read child and exhaustive children. The root test's
`root_tests/request.rs` mount also exists. The retained release and concurrency
tests are present. `replay_pin_concurrency.rs` and
`cold_visibility_round_counters.rs` are byte-equal to the base after newline
normalization; `cold_pins.rs` retains the release test and loses only the named
private custody test. No ignored test was added. The source-court baseline is
unchanged.

## Court topology and evidence adequacy

The new package is a workspace member, unpublished, `autotests = false`, with
exactly one registered test target. Its normal dependency graph resolves to
the Native product frontier; the ceiling exception names only
`worth-relational`, the cold-residency area, and the precise
`relational/relational_certification/cold_residency` owner. The certification
parent delegates that child explicitly. This is justified by the existing
certification court's 35,909 Rust source lines, above the 15,000-line split
threshold. The new court has 545 Rust lines total; its largest source file has
198 lines. The split does not expose a product API or add another production
build.

Supplied verification inspected:

- `landing2a-cold-public-proof-root.log`: one owned build and selected run;
  **6 passed, 0 failed, 0 selected ignored**, comprising all five migrated tests
  plus `recovered_nonhead_envelope_custody_ends_with_the_last_inflight_read`,
  the affected existing recovered-fixture caller. The existing unrelated
  ignored cost test shown by libtest is filtered out, not selected evidence.
- `landing2a-cold-public-check.log`: initial court/testkit check exits 0. The
  later owned build supplies compilation evidence for the expanded five-port
  packet rather than relying on this earlier check alone.
- `landing2a-cold-public-format-final.log`: package formatting exits 0.
- `landing2a-cold-public-line-caps-final2.log`: dirty guard passes all 14 Rust
  files, including untracked new-court files.
- `landing2a-cold-public-context.log`: context check exits 0.
- `landing2a-cold-public-boundary-final.log`: the residual 14 findings described
  above; no green full-gate claim.

The public court and changed fixture source modification times precede the
owned proof run starting 16:57:39 UTC; final formatting did not rewrite these
inputs. The later source changes in this candidate concern restoration of
retained private-tree tests and the four C2 deletions, not changed public proof
inputs. The supplied proof selection, runner, package metadata, exact test
names, reservation configuration, and CPU records are consistent with the six
reported passes. No Query/Store product input changed in this candidate.

The retained source courts have not been newly executed by this review. Their
mount existence is statically established; broader migration/gate execution
remains outside this partial-candidate approval.

## Final mechanical fixture correction

Inspected the subsequent two small `cold_pins.rs` hunks: the fixture field is
renamed from `source_state` to `_source_state`, with a comment naming its old-root
keepalive responsibility, and its initializer now explicitly assigns the same
`source_state` value. The `SnapshotState` remains owned by the same fixture, in
the same field position and with the same lifetime; construction, mirror
probing, assertions, and drop behavior are unchanged. No surviving field access
needs renaming after the private C2 test's removal. This avoids an unread-field
warning while preserving the retained source basis rather than dropping it or
suppressing diagnostics. Approval is unchanged. No build or test was started
for this inspection, and the six public proof inputs remain unchanged.
