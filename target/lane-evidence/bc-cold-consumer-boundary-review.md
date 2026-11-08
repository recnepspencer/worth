# Independent cold-provider consumer boundary review
Reviewer:/root/bc_cold_consumer_boundary_review. Source41ab134fa1. Read-only;
no edits,Cargo,build or test. Navigation closure bc-cold-module-closure.json:
80files/11596lines/86tests,mounted from cold_members.rs,no missing module path.

1. External ImmutableCheckpointMemberSource already implements the local immutable
source contract: selected protection,funded custody,damage/failure controls and
read recording. Reuse it directly,no adapter.
2. Private manifest decoding is not inherently a missing public capability.
Working external checkpoint_admission/cold_materialization/selected_image::member
uses minimal DTOs over public bytes and WORTH-REL-CHK18. Compute canonical locators
from actual SHA256/length as testkit cold_recovery does. Older index custody can
select locators from checkpoint directory; restart growth private fold-span claims
still need an independent artifact-value calculation or explicit decision.
3. Existing public replacements: publication diagnostic_access().snapshot;
observe_branch + branch_head_for_observation; run_branch_root_reclamation_pass and
retired_branch_root_count; inspect_what_happened().inspect_record_retention(...)
returns pins.replay_pins. Last call demands the entire mirror,so never use it before
a cold-refusal action; it would contaminate that proof. It can preserve1->1->1->0
in replay_retention_releases_from_owned_resident_custody_after_cache_clear and
1->0 in concurrent_replay_references_share_one_physical_pin_set after appropriate
public operations. Private cache-clear/action selection needs separate handling.
4. Exact private authority actions:
-cold_isolation::replay_pin_of_a_cold_partition_is_typed_and_has_no_effect manually
constructs SnapshotState,calls resident_replay_pins,proves Cold/Head-only denial
with zero I/O,then separately installs two physical holders and checks2->1->0.
-cold_pins::resident_pin_custody_refuses_unread_mirror_frames_after_exact_source_admission
passes a selected private SnapshotState directly to demand_replay_pins.
Owner visibility/pins/pin_authority/replay_pins.rs marks these actions pub(crate).
Public retain_version_for_replay demands history and manages shared references;
it cannot directly preserve resident-only no-I/O action or two independent-holder
claim. Need genuine owner restructuring/value extraction or claim-placement ruling;
no observer door/forwarding bridge. Ownership probe shows this file unchanged
against Lifecycle and master from7d0e5189bc,so this is not a new file-permission
blocker; the unresolved issue is the honest assertion/authority decomposition.
5. Zero reads alone does not establish every exact private-state predicate:
-cold_fork_edits::head_probes_admit_each_forks_shared_region_without_reading_history
observes symbol-qualified per-fork Cold->Resident state;
-cold_materialization_doors::residency_retained_kind_demands_only_its_cold_head_and_preserves_damage
checks unrelated Head and continued Full coldness;
-cold_symbol_view_levels::head_symbols_never_cover_full_history_or_inherit_its_page_damage
asserts absent history-only symbol,<400resident texts,actual history length and
Full covering Head; cold_candidates/cold_symbols compare actual resident dictionary.
PinnedRead.symbols() obtains Full,so probing it changes Head-only state. Capturing
a checkpoint likewise does not expose the resident dictionary unchanged.
6. Feasible public slice: sticky member/tail refusals,public validation,current/full
read matrices,storage adjacency,observation reads,metadata pins and visibility
operations. Use genuine producer commits,external checkpoints and testkit provider;
map every existing fault/read/counter/publication assertion before removal.

Verdict: provider migration and substantial public consumer work are feasible;
full86-test closure is not established simply by replacing private probes with
reads. Two authority-action cases and resident dictionary/cache controls require
actual owner decomposition/claim-placement resolution. No unsupported-capability
claim is based only on private manifest DTOs or pin counter visibility.
