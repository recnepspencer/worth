# Independent final review: B2 checkpoint slice

Reviewed the staged source/test/spec/baseline snapshot against base
`e4d3b01c8d32ae3135a4dc40aadd322f8e3906e1`. HEAD is still that base and the
tracked working tree has no unstaged delta. The untracked, unmounted
`association_checkpoint_journeys.rs` was excluded. No build, Cargo invocation,
or verification lane was run by this review.

Read the repository instructions, coding guidelines, applicable milestone-3
Moved courts/landing provisions, and the external `landing_2a.md` ruling.
Did not read the implementation plan, ledger, earlier review, progress report,
or draft conclusions. Evidence was limited to the expressly permitted raw and
umbrella checkpoint logs.

## Findings

1. **INFO — Whole boundary gate remains open.**
   `pa-b2-checkpoint-boundary.log` exits 1. It includes the existing Native
   `src/tests/support/durable_medium/runtime_attachment.rs` BC9002 seam, plus
   untouched Query/Store/World and Native home-program-transition files. This
   slice does not change that adapter or claim to remove its remaining callers.
   These are pending next-slice/integration work, not a green whole-repository
   gate. The checkpoint slice adds no baseline entries and deletes exactly the
   three rows for the removed checkpoint source courts.

No P0, P1, P2, or P3 defect was found in the reviewed checkpoint slice.

## Source and assertion review

| Risk | Distinguishing source/evidence | Result |
|---|---|---|
| Discovery adapter or substitute history | New public fixtures explicitly own an `Arc<TestDurableMedium>`; both source and recovery receivers use its attachment; plans receive its actual recovery source. Existing filing stores captured members and original journal bytes. | No new discovery adapter, private mutable root, or regenerated commit history. |
| Unfunded read custody | The existing medium's cold-source implementation freezes original bytes through `OwnedNativeReadSnapshot::capture`; request arrays, ordered cells, and snapshot owner retain request memory reservations. | Public moves use the existing funded source rather than a new raw seam. |
| Authority laundering | Successor courts use owner-prepared candidates and real native recovery authority; they retain exact foreign-runtime/candidate refusals, stale/source-moved distinctions, authority return/retry, unchanged descriptors/counts, and exact durable occurrence census. | Semantic assertions preserved publicly. |
| Lost private/fault claims | Base comparisons locate removed root sharing/image census/backing pointers and snapshot-handle census. New C2 entries name their original SHA:path and fixture closure. C4 entries name both actual scripted outcomes and preserve the offered-fate/no-effect/retry obligations at the original SHA:path. | Per-claim move ruling followed; public observations are not presented as replacement allocation proofs. |
| Incomplete cutover | Removed source files/mounts match destination families; four Native checkpoint journeys moved publicly; one private builder-configuration assertion remains on Memory. The now-unused import and scripted variant are removed. | No duplicate mounted checkpoint court or new suppression; the remaining owner assertion makes no durable-medium claim. |

Branch target, lifecycle, tail coverage, index generation/source identity,
editable restore, adjacency, runtime-name refusal, and recovery-basis assertions
remain at the public boundary. The Memory conversion preserves the exact
custom-invariant registry/sealing observations. Production transition behavior
is unchanged; its source edit removes only the displaced test mount.

All touched Rust files are below 400 lines. The function advisory candidates
are coherent scenario setup/action/assertion sequences; the existing lineage
recovery body is not a new semantic change. No manifest, feature, public
authority surface, or generated AGENT_CONTEXT file changes are introduced.

## Evidence assessment

- All permitted run headers identify the same base HEAD and explicitly report a
  dirty tree; none claims a clean committed candidate.
- Initial public build failed on two nonexistent imports and private
  `current_version_id`; staged code uses the public testkit imports and owner
  observation. The fixed public build succeeds and exact-name selection/run
  logs show **16 distinct selected, unignored public courts passed**.
- The initial Native attempt has metadata and unit-graph success only, then
  stops. It supplies no executable build/test result. The resumed Native build
  succeeds; exact selection and test completion show **one owner test passed**.
  The completed public run was not rerun or counted twice.
- The final Clippy run succeeds. The removed `Stop` import and
  `IndeterminateWithoutEffect` warnings are absent; two warnings remain in the
  untouched native medium refusal payload fields. No suppression was added.
- Formatting, function scan, final dirty line caps, and agent-context logs exit
  zero. The final cap log includes 15 dirty Rust files; the extra untracked
  association file is outside this review. Boundary exits one as noted above.
- Tests preceded formatting and the two dead-code deletions. Those subsequent
  changes do not alter the tested checkpoint behavior, and final Clippy checks
  both affected packages/all targets. No broader lane or Landing-2a/Tier-3
  approval is inferred from this focused evidence.

**APPROVE** — staged B2 checkpoint slice only. The remaining adapter boundary
and whole integration/landing gates remain pending.
