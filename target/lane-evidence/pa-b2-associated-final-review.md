# B2 associated-original and medium final review

Independent final review of the working change against
`0615dd733c4949ec0a50804db2660d6b60295d3f`, including all nine untracked mounted
court files. No builds, tests, Rust edits, or configuration edits were performed
by this reviewer. Only this report was written.

## Numbered findings

1. **Blocking/high/medium: none in this batch.** The public graceful claims
   preserve their original observable assertions. Removed residency,
   allocation, pin, release and manifest-growth claims have C2 restore
   references; route/body corruption and old-format mutations have C4 restore
   references. I compared the original source bodies at the base directly to
   the new courts and retained Native units, rather than accepting the
   implementer's or source review's disposition as evidence. No new local
   provider, admission adapter, fabricated original, weakened typed result, or
   Memory substitute for a durable claim was found.
2. **Informational: wider-task gates remain incomplete.** Boundary exits 1
   with twelve findings, and Clippy exits 0 with two existing local-medium
   refusal-payload warnings. These do not establish a green whole-task or
   Landing 2a gate. The twelve findings reference eleven files, each verified
   unchanged from the reviewed base. The remaining owned
   `src/tests/support/durable_medium/runtime_attachment.rs` BC9002 adapter
   still requires the coordinator's separate closure. The two warnings are
   `TestRecoverySourceRefusal::Commit` and `::Transition` payload fields at
   `src/tests/support/durable_medium.rs:56-57`; this batch deletes unused
   methods in that file but does not alter either variant or its consumers.
   No warning suppression or enforcement exception was added. Under AGENTS'
   scoped-debt rule these results are disclosed, not represented as passing
   repository enforcement.

## Reading boundary and claim pressure

Read AGENTS.md, every document in `docs/coding-guidelines/`, the qa-loop,
qa-tests and code-quality-qa skills, the applicable milestone delivery/landing
and Moved courts text, and `orchestration/m3/spec/landing_2a.md`. Read the
tracked patch with short context, each original removed body at the base, all
nine new court files, and directly used public fixtures/owners. The existing
source-review report was read for comparison only; its conclusion was not
used as clearance.

- **Associated discovery:** the public courts retain actual pending-route
  discovery before settlement, the exact Memory refusal, ordered
  known/orphan/unrelated association inventory, filtered inventory, selected
  CheckpointReadmitted settlement, and original semantic-result/frame
  equality. Exact cold/resident census is C2; missing route and actual body
  deletion guards are C4, with the original names and base paths.
- **Caller association:** preparation and performed-envelope checks, original
  receipt/result/association checks, and cold checkpoint plus tail count,
  coverage, position, identity, digest and byte equality all remain. Wire
  length/transport checks and embedded original codec re-encoding remain
  genuine Native value/codec subjects. The embedded codec unit now uses
  actual Memory publication and makes no durable claim. Route/allocation
  census is C2; CHK13/CHK14 mutation and UnsupportedLegacySemantics are C4.
- **Recorded originals:** existing TestKit fixtures retain the real
  predecessor and successor leases through capture, then release them.
  Recovered admission uses the actual owner authority and original recorded
  references for both Canonical and Empty predecessors. Exported reference
  equality includes descriptor and target, preserving the original canonical
  descriptor assertion. After actual reclamation, funded public snapshot
  reads require the original entity counts and empty relations. Semantic
  result, historical-currentness refusal and no-republication checks remain.
  Private full-root demand, cold/resident posture, omission census and
  release/busy/stop accounting are explicitly C2; unrelated-body deletion is
  C4. Same-SHA fixture and removed re-export restoration instructions exist.
- **Medium ownership/continuation:** the exact acknowledged stream positions,
  absent Memory/fork binding, no fork offers, in-place owner replacement and
  next medium position remain. Recovered original-frame bytes, source byte
  limits, no recovery publication, checkpoint2/tail1 coverage, new commit
  identity/parent/position4, second recovered count4 and exact head receipt
  remain. Concrete cold-source allocation pointer identity is explicitly C2.
- **Checkpoint members and branch history:** selected prior New/Carried
  locators feed the recapture recipe. A real replaced member must disappear
  from its membership while its original bytes and exact limit refusal remain
  readable. Protected-source covered+1 and exact recovered A/main heads,
  compacted-source Memory fork replay, and exact immutable content-root
  reconstruction remain. Both original 32-branch lifecycle scripts preserve
  covered64/128 and resumed65/129. Physical covered-log release and the
  manifest-growth bound are C2.

The only new production behavior is the immutable
`DurabilityAccess::has_bound_durable_medium` DATA observation. It reads the
owner's actual binding slot; fork construction clears that slot while carrying
cold readers separately. It neither issues authority nor decides publication
admission, and the capability index states that distinction. Outcome-sensitive
paths continue to consume typed owner results. The court baseline loses nine
rows and gains none; no target, manifest, dependency, control caller or
construction door is added. All nine new files are 42-145 lines in the
existing certification binary. The six reported function advisories were
inspected: their bodies are coherent publication, source-protection,
replacement, continuation, original-custody journeys, or unchanged private
retention mechanics; size/arity alone is not a finding.

## Execution evidence and provenance

Current HEAD equals every umbrella/raw log's recorded HEAD above. Their working
directory is this worktree and their dirty-state headers are false for
"empty", consistent with the reviewed tracked and untracked batch. The raw
selection/run logs name the mounted final tests. The logs identify a dirty
run, not a content-addressed snapshot of every working byte; this review also
inspected the final source. Tests preceded deletion of the six now-uncalled
support items and formatting. The final diff confirms those later code edits
only remove the orphan probe/re-exports and unused medium head/drop helpers;
Clippy then recompiles both affected packages with all targets. No surviving
test body or production mutation semantics changed in that cleanup.

| Evidence | Independently observed result |
| --- | --- |
| `pa-b2-associated-public.log` and raw failed prebuild | E0308 at `medium_member_retention.rs`: checkpoint members supplied where locators are required; no test body ran |
| `pa-b2-associated-public-fixed.log` and raw build/run | Build succeeds, no compiler diagnostic; 17 distinct selected test `ok` events, exit0 |
| `pa-b2-associated-native.log` and raw build/run | Three selected Native test `ok` events, exit0; eight initial warnings enumerated below |
| `pa-b2-associated-clippy.log` | Exit0 after cleanup; only the two pending refusal-payload warnings remain |
| `pa-b2-associated-format.log` | Formatting command exit0 |
| `pa-b2-associated-line-caps.log` | PASS, 24 dirty Rust files; six Git CRLF conversion notices are informational |
| `pa-b2-associated-functions.log` | Six inspected advisories, zero scan errors, exit0 |
| `pa-b2-associated-boundary.log` | Twelve findings, exit1; no reported file changed in this batch |
| `pa-b2-associated-context.log` | Exit0 |

The failed public prebuild is resolved by exhaustively obtaining locators from
the actual first selection's New/Carried members; the replacement and byte
assertions remain. The Native prebuild's eight warnings comprise the two
surviving payload warnings plus unused `assert_recovered_cut_and_lane_busy`
import/function, `head`, journal `remove`, `drop_artifact` and
`transition_head`. Those six unused items are absent in the final source and
the final Clippy warning set confirms their removal. Raw run summaries include
existing ignored/filter bookkeeping; all 17+3 selected bodies passed, with no
selected body ignored, failed, or replaced by an expensive excluded campaign.

Boundary's exact residual families are BC7012 on Query's retained_program,
four BC7013 construction references in Store's transition catalog/compilation/
native-effect courts, and seven BC9002 courts: Native body_digest,
catalog_digest and runtime_attachment; World's artifact retention,
occurrence-custody retention and released_source; and bank-http-adapter's
authorization-denial instruction tests. Each reported path was compared to
the base and is unchanged. This is scoped approval of the associated-original
and medium batch, not approval to land the wider task or milestone with these
gates unresolved. Landing 2a's broader coherence, delivery-track update and
Tier-3 candidate obligations remain with its coordinator.

APPROVE
