# Stage 2 owner-call rulings

Recommended defaults the coordinator applied during wave 3; each stands until the owner
overrides it. Each entry gives the option set, the ruling, its conditions and the source note.
The wave-4 packets implement them; a packet that cannot meet a ruling's condition stops and
reports instead of choosing another option.

## 1. Commit receipt placement

Source: `packet-notes/W3-09.md` §1 (on the W3-09 branch, merged with wave 3).

Options: keep the receipt in the attempt (the five seams into `application_attempt` stay), or
move it into a new attempt-owned primary-graph child below its holders.

Ruling: create `pg::application_commit` ("the attempt's commit receipt and its evidence, below
output_lineage") at the rank the W3-09 design gives (in the draft stack's `attempt_and_commit`
layer, right after `application_program`; host-order rank 50), and add it to both stack files
(`ownership/stack-primary_graph.json` and `ownership/rerun/stack-primary_graph-draft.json`).

Conditions:
- The receipt's assembling constructor may be `pub(in crate::domain_computation::primary_graph)`
  only if every authority-conferring argument is a sealed permit only the attempt can mint;
  otherwise it stays narrower.
- The certification court's workflow_actor doors move with the receipt.

## 2. The `Uncommitted` / output-demand data cycle

Source: `packet-notes/W3-09.md` §2.

Options: (a) side channel: `CommitDeferred` loses the prerequisite-denial field and the
prerequisite travels beside it; (b) generic carriage (`CommitDeferred<P>`, `Uncommitted<P>`,
`CleanupRequired<P>`); (c) erased custody with a downcast (rejected by architecture law 3).

Ruling: (a), the side channel for the prerequisite refusal, cut at `CommitDeferred`, with its
preconditions:
- `PreparedRebaseDenial` lowered to `vocabulary::provider`;
- `WorthQueryOutputDemandDenialKind` lowered whole (the `application_output_demand::denial::kind`
  unit, kind and recovery posture) to `vocabulary::application_output_demand`;
- `WorthQueryApplicationCommitDenial`'s constructors split so the lowered data carries no mint
  that would widen.

## 3. access_strategy's managed-influence types (for the observed fact)

Source: `packet-notes/W3-09.md` §3.

Options: (i) lower the realized-footprint contract whole (`RetainedManagedInfluence`,
`CapturedManagedInfluence` and the adapter trait) below the observed fact; (ii) keep the
contract in `access_strategy` and give `ManagedSourceRead` an attempt-owned port; (iii) move
`ManagedSourceRead` out of the observed fact into its own family.

Ruling: (i).

## 4. `PendingProducerPublication`

Source: `packet-notes/W3-10.md` §3 (on the W3-10 branch).

Options: A, a checkpoint host route; B, lower the custody object with no new route, after its
two upward field types are lowered by their owners; C, a sealed downcast (ruled out by family
E); D, a generic checkpoint (`WorthQueryOutputCheckpoint<Owner>`); E, status quo (the one
reference stays as recognized custody in the baseline).

Ruling: E now; then B, once `WorthQueryManagedApplicationRecoveryPerformed` and
`WorthQueryRecoveredProgramOutputSource` have been lowered by their owners.

## 5. `ComputationPrior`'s home: still open

Source: `packet-notes/W2-04.md` (the required_context.rs owner call) and the W2-04 leftovers.

The two directions on the table: the output-demand context's prior slots move into the
producer's own context in contribution, or `ComputationPrior` is lowered once output_lineage is
reached through `PriorRecordSelection`.

Ruling: none yet. A wave-4 design step writes the options with a recommendation. Until the owner
rules, its three holders stay untouched:
`application_output_demand/registry/required_context.rs`,
`invariant_projection/operation_reader/decision_reads.rs`,
`invariant_projection/operation_reader/computation_calls.rs`.

## Earlier rulings this file does not repeat

The layering (host-supertrait order first, draft stack for ties), the stack defects
(`runtime_host`, `vocabulary` and `computation_retention` placement), the stray inherent impl
rule, the proposal-03 rulings and the wave-2 owner calls (graph_layout, application_principal,
option 3 for the admission subject) are recorded in the coordinator's messages, `packets.md`,
`packets-w3.md` and `steps.log`.
