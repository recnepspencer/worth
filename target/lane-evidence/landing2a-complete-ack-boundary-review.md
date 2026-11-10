# Independent Complete/ACK boundary review

Reviewer /root/complete_ack_contract_review; source only, no builds/tests/edits.
The governing landing documents name the gate without adding a new contract.
No concrete correctness defect found.

1. Actual contract: milestone-3-durable-owner-contracts.md:1359 and1441 requires
   the same selected attempt, frozen covered cells, backing, cursor and remaining
   work; never recapture or republish to retry.
2. Store checkpoint/owner.rs retains both prepared sections, original subjects
   and actual receipt. journey.rs uses owner.pending(), skips already-selected
   physical publication, then Native ACK before World ACK before completion.
3. Store relational section/checkpoint/publication.rs resolves the SAME subject
   and resumes SelectedReplacement's original expected version, verified reader,
   member cursor and displaced ownership.
4. Native medium/binding/checkpoint_selection.rs retains AcknowledgingCheckpoint,
   canonical/region cursors and retirement progress. checkpoint_history/publication.rs
   captures original cells once; ACK transfers backing only to those cells;
   component holds release only after successful advancement.

Public flow courts to qualify/preserve:
- interrupted_owner_ack_resumes_original_attempt_and_reopens_unacknowledged_selection
- selected_checkpoint_survives_observer_drop_online_history_and_mutation_before_ack_recovery

Narrow Native per-cell transfer courts (not equivalent to the two public flows):
- selected_ack_partial_transfer_preserves_later_cells_and_resumes_same_capture
- selected_ack_exposes_backed_data_before_retirement_and_releases_holds_only_on_completion
Owner: durability/data/native_checkpoint/tests/selected_ack.rs.
The distinction is an evidence boundary, not a discovered production defect.
Gate qualification remains unrun in this slice; no completion claim.
