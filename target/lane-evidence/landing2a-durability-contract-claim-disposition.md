# Durability contract and medium claim disposition

Source-only review at HEAD 45b6e87a0a. No Rust edits, builds or tests. Main finite scope: all currently mounted src/tests/durability/contracts and src/tests/durability/medium children: **65 files, 191 named roots, 0 missing mounts, 0 unclassified roots**. Existing external/moved court tests are analogues below, not counted as remaining Native roots. Several mounted Memory/value roots are explicitly not adapter-causal.

Main roots: **26 wholly HAPPY, 86 C4, 19 C2, 6 owner units, 54 mixed**. Classification follows assertions and fixture actions. Positive private schema construction is not automatically a fault; physical pin/dictionary/cost claims are C2 even when their inspection API is public. Conversely private semantic fields alone do not make a claim C2. Mixed rows name the exact split.

An additional **24 named owner roots** in actual Complete/ACK, checkpoint transition/rebuild and kind-authentication closures are listed separately. That is a bounded owner extension requested for this review, not a claim to have reviewed all other durability/runtime owner tests.

## Mechanical boundary decision

1. HAPPY source/reopen tests keep real owned external medium handles. Existing worth-testkit-relational::durable_medium::TestDurableMedium is already used by the mounted certification court. Constructors take/return Arc<TestDurableMedium>; receiver builds with medium.attachment() before recovery, then uses medium.recovery_source(). No local attached_* discovery, bind_durable_medium, in_crate_test_medium, new local publisher or trait bridge.
2. Genuine owner codec/compiler/config units can keep actual Native Memory. Call runtime_with_test_schema() (CertificationCore/test schema) or the same custom registry/custom invariant builder with .build(). For genuine retained-log reconstruction call source.durability_authority().checkpoint(execution), source.durability().recovery_plan(execution, mode), receiver.durability_recovery().recover(execution, plan). This proves Memory reconstruction explicitly. It is not a journal crash/reopen substitute.
3. Pure local Root/mirror/codec/cost assertions may use actual Memory-produced inputs where no physical medium claim exists; preserve original actual Root/arena/dictionary/pin state and full private assertions. For example Complete partial-aspect/pinned-delete currently uses durable factories but the disputed local original arena/Complete fold can be an actual owner Memory source, like the existing Complete Empty test. This does not authorize removing true source-selection or durable-reopen assertions.
4. C4 retains concrete mutations, exact original *_for_test references, scripted fates and intended no-effect oracles. C2 retains actual reader/alias lifetime, original producer and measured counters/input scale. Moving those to C2/C4 means preserving their full fixture closure, not keeping their old shared local attachment.
5. No new public schema/merge-parent creation or proof-minting door is proposed. Public RelationalTransactionIntent::with_schema_transition already carries intent; the positive fixtures' live target registry installation remains private and must be preserved as owner construction when no shipping registry-change recipe has been established.
6. Two private physical families still require a concrete owner input/court cutover: real medium-bound partial ACK and the exact detach-once medium binding unit. Memory alone cannot satisfy them. Their exact unsatisfied claims are specified below; this review does not paper over them with a replacement provider.

## Every named contract/medium root

Paths are relative to crates/worth-relational/src/tests/durability/. Lines refer to the reviewed source revision.

### contracts/associated_effects.rs

- **associated_effects_refuse_memory_and_absent_settlement_evidence** (41) — **HAPPY + C4**. Memory association refusal is public. MissingSettlementEvidence half removes canonical publication route via remove_canonical_publication_route_for_test; preserve that exact injected route loss in C4.
- **associated_effects_discover_cold_subject_without_reading_unrelated_history** (77) — **HAPPY + C2 + C4**. Public discovery/readmission returns selected association, CheckpointReadmitted settlement and original semantic result. C2 checks selected/unrelated envelopes remain nonresident; C4 deliberately drops unrelated artifact to prove selected readmission does not demand it.

### contracts/associated_effects_discovery.rs

- **native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata** (11) — **HAPPY + C2 + C4**. Normal pending/cold known/orphan/unrelated association metadata enumeration and Memory refusal are public. Helper deletes offered bodies; preserve missing-body campaign C4 and exact envelope nonresidency C2 rather than a warm enumeration substitute.

### contracts/branch_root_checkpoint/foreign_targets.rs

- **checkpoint_recovery_rejects_foreign_branch_target_substitution** (4) — **C4**. Substitutes a foreign branch target in checkpoint cell; CorruptCheckpoint foreign-stream refusal and zero installed immutable commits.
- **tail_recovery_rejects_foreign_branch_target_substitution** (49) — **C4**. Mutates tail envelope branch cell to foreign target through envelope_mut_for_test; same foreign-stream denial before state installation.

### contracts/branch_root_checkpoint.rs

- **checkpoint_recovery_rejects_missing_exact_branch_root_artifact** (4) — **C4**. Clears exact branch-root artifacts; missing image refusal, no recovered commits.
- **checkpoint_recovery_rejects_root_artifact_without_commit_envelope** (40) — **C4**. Removes authoritative commit envelope while retaining root image; missing-envelope refusal and empty target.
- **checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission** (70) — **C4**. Duplicates checkpoint envelope; duplicate identity/artifact refusal before root readmission.
- **checkpoint_recovery_readmits_shared_root_once_for_sibling_heads** (107) — **HAPPY + C2**. Public sibling heads restore same admitted target and both read one entity. C2: branch_roots len=1 and Arc::ptr_eq of readmitted roots prove physical dedup/reuse.
- **tail_recovery_resolves_the_checkpoint_target_root_not_the_fork_source_head** (158) — **HAPPY**. Actual main/source divergence after checkpoint fork; recovered main and storm each contain their own two-record truth, not source's later tail.
- **checkpoint_recovery_rejects_duplicate_global_partition_images** (194) — **C4**. Duplicates global partition image; CorruptCheckpoint duplicate partition image, empty target.
- **checkpoint_recovery_rejects_duplicate_branch_root_partition_images** (225) — **C4**. Duplicates exact-root partition image; same typed refusal before installation.

### contracts/branch_root_content_binding.rs

- **checkpoint_rejects_previous_branch_root_format_before_readmission** (5) — **C4**. Rewrites root format to previous version and recomputes image digest; assert_corrupt_recovery_leaves_no_recovered_state preserves format-specific denial.
- **checkpoint_rejects_altered_root_content_with_recomputed_image_digest** (22) — **C4**. Changes actual root content and recomputes image digest; canonical owner binding still denies StorageContentMismatch.
- **checkpoint_rejects_omitted_root_partition_with_recomputed_image_digest** (40) — **C4**. Omits partition and recomputes image digest; preserve exact missing-content denial/no target state.
- **checkpoint_rejects_same_commit_truth_root_relabeling** (59) — **C4**. Relabels same commit truth-root descriptor; recomputed owner image cannot bypass StorageContentMismatch.
- **checkpoint_rejects_same_commit_schema_root_relabeling** (80) — **C4**. Relabels same commit schema-root descriptor; SchemaRootMismatch before installation.
- **tail_rejects_same_commit_root_descriptor_substitution** (101) — **C4**. Mutates actual tail envelope descriptor; StorageContentMismatch and no recovered state.
- **tail_replay_reconstructs_the_owner_content_root** (138) — **HAPPY**. Unmodified production tail reconstructs exact original admitted content descriptor. Public observed basis.descriptor/roots can compare the same immutable semantic target.
- **checkpoint_owner_root_is_stable_across_fingerprint_symbol_ids** (162) — **C4**. Installs structural fingerprint images with two invented symbol IDs into cloned plans, recomputes digests, compares identical StorageContentMismatch details. This is a denial-stability campaign, not ordinary dictionary persistence.

### contracts/branch_root_schema_binding/exact_contracts.rs

- **recovered_exact_roots_interpret_records_with_their_own_schema_contracts** (4) — **HAPPY + owner schema-history fixture**. Genuine v1/v2 branch histories recover exact old-name/new-display contracts, reject foreign projection scopes and v2 authoring on v1 root. Private target registry installation precedes valid transition publication; this is positive owner fixture construction, not itself a fault. Keep exact schema-history owner check with actual Native Memory checkpoint/reconstruction if public live target-registry route is unavailable; external durable continuation must use genuine source and preserve full claim. No new schema authority door.

### contracts/branch_root_schema_binding.rs

- **branch_roots_with_one_schema_deduplicate_the_durable_carrier** (4) — **C2**. Multiple exact branch roots emit exactly one durable schema carrier. Physical carrier dedup is the assertion, not merely visible truth parity.
- **recovery_rejects_a_missing_branch_root_schema_carrier** (28) — **C4**. Clears branch-root schema carriers; missing schema carrier denial and no target effects.
- **recovery_rejects_a_tampered_branch_root_schema_carrier** (43) — **C4**. corrupt_schema_root_for_test changes original carrier commitment; digest mismatch denial.
- **recovery_rejects_a_schema_carrier_swapped_between_exact_roots** (58) — **C4**. Forced two-schema setup then swaps schema_carrier_digest between roots; linkage mismatch denial before installation.

### contracts/branch_root_shared_target_binding.rs

- **checkpoint_rejects_later_shared_root_cell_with_relabeled_schema_root** (10) — **C4**. Later sibling cell sharing target has relabeled schema root; exact canonical linkage denial.
- **checkpoint_rejects_later_shared_root_cell_with_relabeled_truth_root** (35) — **C4**. Later sibling cell sharing target has relabeled truth root; exact canonical linkage denial.

### contracts/branch_transition/checkpoint_state.rs

- **checkpoint_size_tracks_branch_state_not_covered_transitions** (6) — **HAPPY + C2**. Public recovered transition head resumes after covered sequence. C2: 32 branch lifecycles produce 64 versus 128 transitions, compact to released covered log, and checkpoint bytes stay within direct+32 despite archive/deletion history; preserve scale and measured manifest sizes.
- **a_deleting_checkpoint_cell_requires_its_retired_name** (68) — **C4**. Deletes retired-name set while retaining Deleting cell; exact deleting-without-retired-name denial, empty target.

### contracts/branch_transition/fates.rs

- **every_transition_fate_preserves_the_durable_before_live_boundary** (134) — **C4**. Matrix scripts Committed, ProvenNoEffect and Indeterminate transition fates for fork/archive/request-deletion/delete; compare actual live footprint, offered artifact sequence/follows/digest, unresolved blocking and durable sequence continuity.
- **an_unresolved_commit_also_blocks_branch_transitions_with_its_original_fate** (249) — **C4**. Scripts unresolved commit then attempts branch transitions; no transition offered and exact original unresolved fate returned.

### contracts/branch_transition/reconstruction.rs

- **memory_checkpoint_keeps_replay_reconstructible** (31) — **HAPPY Memory (not provider causal)**. Actual Memory checkpoint/replay reconstructs covered fork transitions with CompactAfterCheckpoint. Keep Memory reconstruction explicit; no disk/reopen claim.
- **replay_reads_only_the_gap_after_its_checkpoint** (91) — **C2**. Counting transition medium observes replay reading at most two missing sequences strictly after checkpoint coverage. Preserve missing-gap setup and read observer.
- **recovery_plan_after_native_capture_keeps_covered_forks** (140) — **HAPPY + C2**. Public checkpoint capture keeps covered fork target and subsequent main head. C2: exact read-protection canonical lower bound covered+1; preserve actual protection owner rather than infer from head parity.
- **a_memory_fork_can_replay_its_compacted_durable_source** (206) — **HAPPY + owner posture**. Actual fork is Memory and replays compacted durable source without publishing to source medium. Public source offered-count/positions proves no writes; exact durability.medium().is_none() is an owner configuration assertion, not C2.

### contracts/branch_transition/recovery.rs

- **a_durable_fork_without_a_subsequent_commit_reopens_and_commits** (6) — **HAPPY**. Real durable fork with no subsequent target commit reopens, commits, and reopens again with target head intact.
- **a_fork_covered_by_a_checkpoint_after_its_source_moves_accepts_a_target_tail** (27) — **HAPPY**. Checkpoint-covered fork retains selected target after source advances and target tail publication.
- **a_fork_in_the_transition_tail_is_merged_before_the_target_commit** (49) — **HAPPY**. Transition-tail fork is applied before target commit; exact target receipt and transition artifacts match source.
- **lifecycle_postures_and_retired_names_survive_checkpoints_and_crashes** (72) — **HAPPY**. Real lifecycle transitions recover Live/Archived/Deleting/deleted postures and retired names; reusing retired names refuses through owner fork port.
- **independent_runtimes_encode_byte_identical_transition_histories** (136) — **HAPPY**. Two independent actual branch-transition scripts emit byte-identical original transition histories (five artifacts).

### contracts/branch_transition/tampering.rs

- **a_damaged_transition_frame_is_refused_at_the_medium_read** (19) — **C4**. Flips actual transition-frame bytes; source read refuses DigestMismatch.
- **a_sequence_gap_and_invalid_follows_positions_are_refused** (36) — **C4**. Deletes sequence 1 and reseals altered follows in sequence 2; NonContiguousBranchTransitionTail, with pristine positive rejoin guard.
- **a_commit_carrying_an_unknown_branch_cell_cannot_replace_its_missing_fork** (72) — **C4**. Removes actual fork transition while tail commit names its cell; CorruptCheckpoint missing-fork refusal, pristine guard retained.

### contracts/caller_association.rs

- **caller_association_bounds_and_rebinding_refuse_without_retaining_publication** (69) — **HAPPY/value + C2**. Public/value: empty/oversize association bounds and serialization, duplicate binding refuses, candidate association preserved. C2: pending allocation snapshot empty and pending route count +1/returns after discard. Split exact retained-state census.
- **caller_association_survives_embedded_checkpoint_and_refuses_old_semantics** (123) — **HAPPY + owner codec + C4**. Public association survives original artifact re-encode. Private embedded checkpoint codec verifies recovered association; old format byte rewrite denies UnsupportedLegacySemantics (C4). Exact codec/model assertion can be local value unit with genuine Memory-produced artifact, no false durable reopen.
- **caller_association_survives_cold_checkpoint_history_and_journal_tail** (161) — **HAPPY + C2**. Public original association/position/commit/digest/bytes/semantic result survive real checkpoint and tail. C2: selected directory present, journal placement absent and positioned_if_resident None proves cold posture.

### contracts/canonical_artifact_neutrality.rs

- **independent_runtimes_publish_byte_identical_canonical_artifacts** (162) — **owner canonical compiler/value (Memory)**. Two actual Memory runtimes with different instance IDs emit byte-identical artifacts for create/fork/merge/schema-transition script. Positive private target-registry fixture remains owner, not C4 merely for set_schema_registry_for_test. Already no medium dependency.
- **a_recovered_runtime_continues_with_the_reference_artifacts** (177) — **HAPPY + owner schema-history fixture**. Actual durable source, journal-only/checkpoint-tail reopen and continuation emit every reference artifact byte-identically. The final valid schema transition uses owner-private registry installation; preserve that owner encoding unit separately from public durable prefix/continuation. Do not Memory-fake this root's real journal/crash claim.

### contracts/canonical_commit_artifact/refusals.rs

- **frame_defects_refuse_before_decoding_each_with_its_own_refusal** (118) — **C4 (Memory artifact)**. Actual exported artifact bytes: flipped digest, wrong magic/version, truncated payload/header, invalid payload shape and trailing bytes each get exact frame refusal.
- **header_rewritten_with_a_recomputed_digest_disagrees_with_the_payload** (183) — **C4 (Memory artifact)**. Reseals actual frame after moving header position/commit; decoded payload disagreement refuses, not digest rejection.
- **intact_frames_with_foreign_payloads_refuse_at_decode_or_canonical_form** (208) — **C4 (Memory artifact)**. Reseals undecodable/foreign padded payload; exact PayloadDecode vs NonCanonicalPayload denial.

### contracts/canonical_commit_artifact.rs

- **every_retained_commit_readmits_from_its_artifact_unchanged** (65) — **owner codec + C2 (Memory)**. Actual Memory mixed create/edit/relation/schema history exports and readmits equal authoritative payload and required-output codec retirements. Positive private schema fixture is owner compiler setup, not corruption. Exact symbol_texts dictionary equality is C2; rest is owner canonical value/encoding evidence, already no medium.
- **encoding_a_commit_is_deterministic_across_encodes_and_readmission** (104) — **owner codec (Memory)**. Two encodes and readmit/re-encode yield identical original bytes, including genuine private-authored schema-transition history. No physical persistence or injection claim; already Memory, no attachment cause.
- **retained_artifacts_follow_strictly_increasing_positions** (129) — **owner codec (Memory)**. Actual Memory original artifacts have strictly increasing positions; from_untrusted_bytes preserves position/commit/digest. Positive private schema-history fixture is owner setup, not C4.
- **positioned_agreement_uses_original_work_budget_and_preserves_authoritative_digest** (149) — **C2 (Memory)**. Large >64KiB payload, two-pass work ceiling, zero residual charged memory, request cancellation/deadline and unchanged original bytes. Exact Arc distinctness and authoritative digest agreement are guards in this measured work campaign.

### contracts/checkpoint_recovery/checkpoint_fallback.rs

- **durability_contract_damaged_checkpoint_refuses_rather_than_shortening_history** (5) — **C4**. Damages actual filed checkpoint bytes; refuses CorruptCheckpoint rather than silently shortening history, no target head.

### contracts/checkpoint_recovery/empty_head.rs

- **durability_contract_empty_checkpoint_restores_a_live_retained_head** (5) — **HAPPY + C2**. Public real empty checkpoint restores live admitted Empty basis. C2: selected root identity=0 and exact retention head_installs=1.

### contracts/checkpoint_recovery.rs

- **durability_contract_checkpoint_tail_recovery_preserves_post_checkpoint_commits** (6) — **HAPPY + C2**. Public real checkpoint+tail restores two commits, coverage 1+1, latest/branch receipt and earlier replay envelope. C2: retained checkpoint partition_images pointer is identical to original captured allocation, not copied/re-encoded.
- **durability_tail_replay_releases_each_commit_snapshot_beyond_live_capacity** (53) — **C2**. 71 tail commits exceed configured live snapshot capacity; recovered published_snapshot_handle_count=0 proves replay closeout releases every transient snapshot. Recovered count is correctness guard.
- **durability_contract_recovery_fails_closed_without_exact_branch_cells** (97) — **C4**. Clears checkpoint branch cells; omitted exact-cell state denies before recovered state.
- **durability_contract_recovery_rejects_branch_cell_target_without_catalog_artifact** (134) — **C4**. Checkpoint cell references commit omitted from immutable catalog; exact references-missing-artifact denial before installation.
- **durability_contract_recovery_rejects_tail_checkpoint_drift_before_mutation** (185) — **C4**. Mutates tail branch-cell checkpoint via envelope_mut_for_test; drift detected before installing second commit.
- **durability_contract_checkpoint_recovers_index_metadata** (234) — **HAPPY**. Actual registered/built index metadata restores generation identity and source commit after checkpoint.
- **durability_contract_checkpoint_recovers_lineage_authority** (282) — **HAPPY + owner codec**. Public recovered graph has two nodes/two Create events and exact per-record lineage IDs. Private checkpoint-lineage digest counts match every original envelope and node counter matches nodes length; preserve these exact semantic encoding assertions as owner codec/value checks, not C2 just because decoder is private.

### contracts/declared_schema_binding.rs

- **another_label_with_identical_contracts_refuses_before_service** (74) — **HAPPY**. Actual durable schema v1 refuses a receiver declaring identical contracts under label v7; exact DeclaredSchemaMismatch records expected/found versions for tail/checkpoint cases.
- **after_a_durable_transition_only_its_target_is_served** (92) — **HAPPY + owner schema-history fixture**. Actual durable source after valid v1->v2 transition refuses declared v1 and serves v2 in journal/checkpoint cases. Public intent.with_schema_transition already exists; target registry installation remains owner-private. Keep exact private schema-history unit separate; preserve actual medium for external physical reopen claim, no newly invented authority door.

### contracts/derived_index_checkpoint/artifact_affinity.rs

- **legacy_index_affinity_is_checked_before_replacing_a_live_recovery_target** (5) — **C4**. Mutates checkpoint cached generation branch affinity; recover into nonempty target refuses and preserves original observation.
- **envelope_index_affinity_requires_admission_before_installation** (55) — **C4**. copy_original_for_test/body_mut_for_test inject foreign envelope generation affinity; typed refusal precedes installation and actual generation stays equal.

### contracts/derived_index_checkpoint/deleted_branch_reader.rs

- **deleted_branch_reader_retains_only_its_exact_scoped_generation** (5) — **C2**. Actual reader pin retains exactly selected scoped generation after branch deletion; restored checkpoint omits it, dropping reader permits exactly one reclaim and no residue. Preserve pin lifetime/oracle.

### contracts/derived_index_checkpoint/historical_reader.rs

- **historical_reader_from_live_head_survives_head_retirement** (4) — **C2**. Historical reader from live head survives retirement; exact old/new generation presence, zero reclaim while pinned then one on release.

### contracts/derived_index_checkpoint/lagging_scoped.rs

- **lagging_inherited_and_own_stream_scoped_generations_survive_checkpoint_reopen** (7) — **HAPPY + C2**. Public current observed generation is None for a lagging cache and actual truth remains correct for inherited/own streams. C2: exact retained stale generation ID/source/applicability survives checkpoint independently of current authority. No fabricated generations.

### contracts/derived_index_checkpoint/scoped_affinity.rs

- **real_shared_parent_root_scoped_checkpoint_restores_lookup_and_content** (71) — **HAPPY**. Actual shared-parent-root scoped generation readmits with original generation/branch, indexed lookup candidates and exact branch content parity after recovery.
- **digest_valid_unissued_scoped_branch_denies_without_replacing_target_state** (135) — **C4**. Rewrites/reseals generation bytes to unissued foreign branch; exact affinity refusal preserves target head, definitions, generations and visible truth; no foreign branch installed.

### contracts/derived_index_checkpoint.rs

- **reclamation_and_checkpoint_keep_sibling_head_generation** (61) — **C2**. Reclamation/checkpoint preserve sibling-head and main generations. Actual retained generation membership is the claimed physical lifecycle.
- **live_fork_scoped_generation_readmits_from_shared_basis** (108) — **HAPPY + C2**. Public fork scoped build succeeds; exact internal generation survives recovered shared basis (C2). Public read_published_generation_for_observation can retain behavior companion, not physical retained-map proof.
- **repeated_edits_checkpoint_only_the_live_index_payload** (147) — **C2**. 24 actual create/build publications; retained bytes*4 < legacy bytes and retained generation count<24. Actual cache size/reclamation growth claim.
- **checkpoint_index_artifact_survives_tail_replay_without_envelope_caches** (179) — **HAPPY + C2**. Public real checkpoint/tail permits rebuilt current index behavior. Exact empty envelope cache/format=2, preserved first generation, omitted unjournaled cache and rebuilt internal generation are owner residency/retention evidence C2.
- **checkpoint_index_artifact_missing_payload_denies_before_recovery** (210) — **C4**. Removes actual index payload from checkpoint variants; CorruptCheckpoint before recovery.
- **checkpoint_recovery_denies_a_digest_valid_foreign_generation_affinity** (252) — **C4**. Substitutes digest-valid generation with foreign affinity; exact branch-affinity denial.
- **reclamation_keeps_a_pinned_old_reader_until_release** (295) — **C2**. Real old reader pins generation while middle is reclaimed; restored checkpoint excludes pin-only historical generation; releasing pin removes original generation, keeps latest.
- **repeated_closed_branches_leave_no_latest_generation_residue** (337) — **C2**. Repeated scoped branch builds/deletes each reclaim exactly one generation; no latest generation or retired binding residue after loop.

### contracts/descriptor_continuity.rs

- **durability_contract_failure_schema_mismatch_is_explicit** (5) — **HAPPY**. Real source schema disagrees with separately declared receiver schema; typed SchemaMismatch/SchemaRegistryShape versions.
- **durability_contract_failure_descriptor_semantics_version_mismatch_is_explicit** (48) — **C4 (Memory)**. Mutates RecoveryPlan.descriptor_semantics_version to 99; exact expected99/found1 mismatch. Already genuine Memory model, no adapter cause.
- **durability_recovery_plan_preserves_explicit_verification_mode** (82) — **HAPPY Memory**. Explicit Normal/Audit recovery planning returns requested verification_mode unchanged; Memory path, no persistence assertion.
- **contradictory_descriptor_version_refuses_before_recovery_or_owner_effects** (113) — **C4**. Rewrites canonical envelope descriptor version in actual medium; PositionAdmission(SemanticResult(SchemaMismatch)) before recovery/runtime identity/head effects.
- **durability_contract_failure_descriptor_canonical_basis_version_mismatch_is_explicit** (148) — **C4**. Forced registry plus artifact descriptor canonical-basis rewrite; exact Drift/DescriptorCanonicalBasisVersion/Rejected DigestParity report.
- **durability_recovery_emits_authority_continuity_diagnostic_before_execution** (230) — **HAPPY**. Public recovery authority-continuity diagnostic exists before execution with accepted DigestParity layer.
- **durability_certification_recovery_authority_continuity_is_explained_and_counted** (267) — **HAPPY + C2**. Public explanatory diagnostic fields verification mode/parities/layer retained. C2: actual replay_digest_parity_checks >=1 and mismatch counter=0.

### contracts/fork_provenance_checkpoint.rs

- **recovery_rejects_one_sided_fork_provenance_in_checkpoint** (125) — **C4**. Helper omits fork provenance or fork source from captured checkpoint; exact denied recovery helper.
- **recovery_rejects_mismatched_fork_provenance_identity_in_checkpoint** (140) — **C4**. Rewrites provenance branch identity; deny mismatch before target installation.
- **recovery_rejects_one_sided_fork_provenance_in_tail** (151) — **C4**. Tail fixture retains only one side of required fork provenance/source; both variants deny.

### contracts/global_generation_reclamation.rs

- **repeated_global_same_basis_builds_reclaim_duplicates_but_keep_pinned_root** (5) — **C2**. 16 duplicate same-basis global builds reclaim while actual selected generation remains reader-pinned; checkpoint saves only current; release reclaims selected old generation. Exact counts and pin lifetime.

### contracts/lineage_allocator_denials.rs

- **tail_created_lineage_allocator_exhaustion_denies_before_replay** (5) — **C4**. Rewrites tail lineage ID to exhaustion and rebuilds lineage basis; CorruptCanonicalArtifact tail-lineage-ID-exhausted before replay.
- **live_lineage_allocator_exhaustion_denies_before_public_effects** (41) — **C4**. Forces live lineage allocator near u64::MAX; rejected actual publication preserves exact frontiers/head/live entities. Keep injection and effect oracle.

### contracts/lineage_artifact_recovery/lineage_regression.rs

- **tail_created_lineage_regression_below_checkpoint_high_water_denies** (5) — **C4**. Mutates/rebases tail created-lineage ID below checkpoint high-water; typed canonical-artifact refusal.
- **tail_lineage_event_regression_below_checkpoint_high_water_denies** (36) — **C4**. Mutates/rebases tail event ID below checkpoint high-water; exact event-does-not-advance refusal.

### contracts/lineage_artifact_recovery.rs

- **checkpoint_duplicate_lineage_event_ids_deny** (5) — **C4**. Duplicate event ID in checkpoint envelopes, rebuild basis, require CorruptCheckpoint reused ID.
- **tail_duplicate_lineage_event_ids_deny** (47) — **C4**. Duplicate event ID across tail envelopes, rebuilt basis, CorruptCanonicalArtifact reused ID.
- **checkpoint_and_tail_lineage_event_id_collision_denies** (87) — **C4**. Checkpoint/tail event ID collision refuses canonical tail.
- **tail_lineage_commit_and_branch_cross_splices_deny** (126) — **C4**. Cross-splices tail lineage commit/branch authority and rebuilds basis; cross-spliced refusal.
- **checkpoint_lineage_commit_cross_splice_denies** (158) — **C4**. Cross-splices checkpoint lineage commit authority; checkpoint refusal.
- **replay_and_durable_lineage_payload_conflict_denies** (188) — **C4**. Conflicting replay/durable lineage payloads with recomputed bases; create-lineage-ID denial.
- **checkpoint_lineage_node_deletion_denies_before_installation** (234) — **C4**. Clears checkpoint lineage nodes; lineage-counter corruption denies before installation.
- **checkpoint_lineage_node_remap_denies_before_installation** (260) — **C4**. Remaps checkpoint lineage node entity targets; both variants reject exact-entity mismatch.
- **checkpoint_lineage_node_swap_denies_before_installation** (294) — **C4**. Swaps two checkpoint lineage nodes; exact-entity mismatch before installation.
- **checkpoint_duplicate_lineage_entity_mapping_denies_before_installation** (339) — **C4**. Duplicates lineage entity mapping; exact-entity mismatch before installation.

### contracts/lineage_event_allocator_recovery.rs

- **tail_lineage_event_allocator_exhaustion_denies_before_replay** (6) — **C4**. Rewrites tail lineage event to exhaustion, rebuilds basis; denies before replay and installs no immutable commits.
- **unresolved_durable_publication_blocks_descendants_and_recovers_last_checkpoint** (66) — **C4**. Scripts IndeterminateWithoutEffect; descendants blocked with original fate, recovery returns checkpoint lineage frontiers, retries original position with correct parent.
- **multi_event_reservation_exhaustion_denies_before_public_effects** (138) — **C4**. Forces event frontier near u64::MAX for multi-event publication; no public effect, frontier/head/entities unchanged.
- **branch_local_sparse_slot_create_publishes_and_recovers_exact_lineage** (195) — **HAPPY + owner lineage observation**. Actual feature create has sparse slot>1 and emitted event targets equal its lineage ID; recovered mapping exact. Private nodes_snapshot lookup is semantic lineage mapping, not automatically C2; use public lineage_access().for_record for lineage ID and public graph/events for target equality.

### contracts/managed_field_text/missing_dictionary.rs

- **native_field_text_missing_tail_dictionary_refuses_without_target_or_source_effect** (6) — **C4**. Drops required managed string dictionary row in genuine tail/reseals; readmission denies without target head/symbol table/source byte changes. Independent producer commit_texts refuses unresolved symbol reference.

### contracts/managed_field_text.rs

- **native_field_text_checkpoint_then_tail_recovers_exact_json_after_original_owners_drop** (84) — **HAPPY + C2**. Public real checkpoint+tail restores exact JSON after original owners drop. C2: actual symbol/key dictionary resolution, charged_payload_bytes None for ordinary client key, retained text owner/custody and managed JSON symbol presence.
- **native_field_text_checkpoint_readmission_denial_keeps_bytes_and_target_root_intact** (150) — **C2**. Memory policy admits only one byte, demands first managed-text scope; exact PolicyMemoryExhausted requested/admitted/ancestor, original native bytes and target root unchanged.
- **native_field_text_configuration_requires_current_explicit_readmission** (212) — **owner codec + C2**. Legacy SymbolTableSnapshot decode refuses without explicit current readmission; current native readmission preserves JSON. C2 exact managed text charged_payload_bytes=JSON.len and dictionary ownership; no runtime/reopen claim.
- **native_field_text_prior_checkpoint_format_refuses_before_managed_text** (243) — **C4**. Rewrites captured checkpoint to prior format; UnsupportedLegacySemantics before managed text, original bytes remain decodable.

### contracts/merge_replay_continuity.rs

- **durability_contract_recovery_preserves_merge_parent_order** (5) — **HAPPY**. Genuine public prepared merge recovery preserves exact ordered parent receipts, merge-parent branches and merge-base commit list.
- **durability_contract_replays_empty_intent_merge_currentness_once** (63) — **HAPPY**. Empty-intent exact-shared merge recovery advances currentness once; generation, truth version and selected commit/version equal actual postmerge basis.
- **durability_contract_replays_merge_from_typed_authority_when_diagnostics_are_absent** (129) — **HAPPY + owner projection + C4 setup**. Production artifacts already omit diagnostic entries; original helper redundantly rewrites/reseals medium envelope and tests nested-byte size difference. Public typed merge authority/recovered head and full durable meaning parity can remain HAPPY; retain private diagnostic stripping/projection sensitivity check in owner/C4 lane (includes original-input attribution caveat).
- **durability_contract_rejects_parent_order_tampering_at_source_admission** (226) — **C4**. Swaps parent order in actual medium envelope; source admission rejects ReceiptMismatch even after intact frame.
- **durability_contract_does_not_reconstruct_missing_merge_authority_from_diagnostics** (274) — **C4**. Removes typed merge authority while leaving diagnostics; replay refuses reconstruction instead of inventing authority from diagnostics.

### contracts/native_checkpoint/allocation.rs

- **native_checkpoint_retains_all_admitted_capture_owners_and_restores_editable_truth** (25) — **HAPPY + C2**. Public recovered editable world/identity/next commit is positive guard. C2 actual request memory refusal, cancellation, System/funded byte equality, Arc payload pointer sharing, capture owner retained-byte quote, capacity hold and release after last alias. No fake Memory reopen.

### contracts/native_checkpoint/speculative_symbols.rs

- **unpublished_client_keys_leave_checkpoint_dictionary_equal_and_published_keys_restore** (4) — **HAPPY + C2**. Public published name survives recovery and unpublished name does not become truth. C2: exact symbol frontier/dictionary, every captured model axis and directory schema/record identity equality, symbols contain/absent and native checkpoint ownership.

### contracts/native_checkpoint.rs

- **native_checkpoint_round_trip_restores_a_live_editable_world** (56) — **HAPPY**. Actual native checkpoint restores real editable world into different runtime identity; follow-up commit updates recovered head. Construct receiver with original retained medium at build, no bind_durable_medium.
- **recovery_authority_only_admits_a_basis_from_the_recovered_image** (96) — **HAPPY**. Actual recovered authority accepts only basis of its recovered image and refuses genuinely moved basis. Public restore_native_checkpoint_with_authority/admit_basis.
- **native_checkpoint_round_trip_restores_current_relation_adjacency** (141) — **HAPPY**. Real relation world restores adjacency and projected relation lookup with original relation/target IDs.
- **repeated_native_captures_do_not_retain_checkpoint_images** (235) — **C2**. Repeated native capture leaves runtime retained checkpoint registry count unchanged; actual retention claim. Recovery/editability is positive guard.
- **corrupt_native_checkpoint_is_denied_without_publishing_recovered_state** (272) — **C4**. Corrupt from_untrusted_bytes refuses CorruptCheckpoint and installs no immutable commits.
- **native_checkpoint_rejects_a_foreign_runtime_name_without_publishing_state** (293) — **HAPPY**. Genuine separately configured foreign runtime_name is refused RuntimeNameMismatch with no installed state; no forged checkpoint needed.
- **native_checkpoint_preserves_unsealed_builder_invariant_authority** (323) — **owner runtime configuration**. Exact recovered custom-invariant registry rule ID and initial_custom_invariants_sealed=false. Keep actual builder custom rule and same Native Memory checkpoint/restore for this private configuration unit; it makes no medium-persistence assertion. Public callback-after-recovery companion can prove rule behavior without exposing the private flag.

### contracts/native_checkpoint_restore_benchmark.rs

- **synthetic_native_restore_and_image_digest_benchmark** (13) — **C2**. Ignored reason: run explicitly to measure a bounded synthetic native checkpoint restore. RECORDS=8192, chunks4096, DIGEST_SAMPLES=16; Instant measures buffered/streamed digest and restore elapsed, reports image/checkpoint bytes. Keep buffered_digest plus streamed hash equality/restored count guards and actual workload in C2.

### contracts/record_allocation_recovery.rs

- **tail_replay_consumes_canonical_reused_record_allocation** (5) — **HAPPY + C2**. Public current/historical names survive canonical reused-record allocation replay. C2: original reusable slot count, exact reused local_slot, advanced generation, retained old reader and reservation allocator evidence.
- **checkpoint_restores_burned_append_frontier_beyond_all_retained_roots** (78) — **C2**. Burned append reservation beyond all retained roots advances durable frontier; recovered next slot is original+2 and generation1. Actual allocator state, not C4 malformed input.
- **recovery_releases_orphaned_append_reservation_without_reusing_its_gap** (114) — **C2**. Genuine outstanding append reservation is orphaned at recovery; pending reservations empty and burned gap not reused (next original+2).
- **recovery_returns_orphaned_reclaimed_reservation_to_reuse** (150) — **C2**. Genuine reclaimed-slot reservation orphaned at recovery is returned to reuse; pending empty, exact same local slot with newer generation.
- **tail_replay_rejects_canonical_allocation_targeting_a_live_slot** (202) — **C4**. Mutates canonical allocation to target live slot; ReplayFailure unavailable Entity, no installed commits.
- **tail_replay_rejects_missing_canonical_allocation_evidence** (250) — **C4**. Deletes canonical allocation evidence; missing ordinal0 replay denial.
- **tail_replay_gap_admission_cannot_replace_a_different_canonical_effect** (284) — **C4**. Changes canonical effect in gap-admission evidence; exact different canonical effect denial.
- **metadata_only_merge_rejects_unconsumable_allocation_evidence** (329) — **C4**. Adds unconsumable allocation evidence to metadata-only merge; no mutation replay path denial.

### contracts/recorded_component_checkpoint.rs

- **recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor** (239) — **HAPPY + C2 + C4**. original_custody(false,false): genuine cold recorded predecessor/successor proof, descriptor/readability/currentness/no republish. C2 Weak/residency/exact component-root custody. C4 helper drops unrelated artifact before readmission to prove no unrelated demand.
- **recorded_component_checkpoint_readmits_original_empty_predecessor** (243) — **HAPPY + C2 + C4**. original_custody(true,false): same full closure with actual original Empty predecessor/schema. Preserve Empty branch, no invented descriptor or authority.
- **recorded_component_checkpoint_omits_released_components_and_plain_observations** (247) — **HAPPY + C2 + owner codec**. Released component leases/plain observations are omitted from exact manifest roots/retained_empty/recorded_components. Public recorded readmission omission can be tested; decoded manifest roster and exact retained-root inventory are owner retention C2. No failure injection in this root.
- **recorded_component_checkpoint_refuses_chk15_before_source_admission** (295) — **C4**. Rewrites native magic/format to CHK15; UnsupportedLegacySemantics before source admission.
- **recorded_original_retirement_preserves_both_leases_across_busy_and_stop** (313) — **HAPPY + C2 + C4**. original_custody(false,true): actual recovered proof retirement Busy and request stop; C2 exact terminal explicit/dropped releases +1/+2 and real held roots. Retains unrelated-artifact fault and original custody helper; do not replace with ordinary public close alone.

### contracts/rejected_and_parent_closure.rs

- **durability_contract_recovery_ignores_rejected_relation_integrity_attempts** (5) — **HAPPY**. Actual invalid cardinality transaction is rejected; latest commit/patch and relation aspect history digest stay unchanged and recovery ignores attempted effect.
- **durability_contract_failure_missing_authoritative_parent_closure_is_explicit** (144) — **C4**. Constructs RecoveryPlan with only child via PositionedCanonicalCommit::for_test and missing authoritative parent; exact MissingAuthoritativeParentClosure/CanonicalHistoryDrift.

### contracts/retention_inspection_store.rs

- **durability_contract_recovery_rebuilds_branch_head_root_obligations** (5) — **HAPPY + C2**. Public recovered feature entity/relation truth intact. C2 exact active_snapshot_count=0 and branch_pinned_entities/relations=0 prove root obligations did not mutate legacy record pins.
- **durability_contract_recovery_preserves_inspection_truth_bundle** (59) — **HAPPY + C2**. Exact graph/kind/connectivity/history/branch/latest/recent commit inspection bundle equality across genuine recovery is HAPPY. Shared capture_inspection_truth_bundle also compares retention_summary and record_retention (actual pin/retained-state data): retain these C2 assertions separately; no private-visibility gap. Replace legacy latest_commit alias with historical_latest_commit.
- **durability_contract_branch_heads_do_not_mutate_legacy_record_pins** (128) — **C2**. After create/fork/main update/feature update, inspect_record_retention branch_pins stays exactly0. Actual pin semantics.

### contracts/schema_transition_continuity.rs

- **durable_recovery_and_schema_mismatch_test** (5) — **HAPPY + owner schema-history fixture + C2**. Positive owner-private target registry installation plus genuine schema-transition commit; recovered transition/continuation/reconciliation, authority continuity and separately declared receiver mismatch are semantic behavior. C2 actual replay_digest_parity_checks>=1. Private setup is not a corruption fault; no established public target-registry replacement recipe in this scoped search.
- **durability_contract_failure_aspect_plan_mismatch_is_explicit** (162) — **HAPPY**. Separately declared receiver has different entity aspect plan revision; actual source plan refuses with exact EntityAspectPlanRevision mismatch.
- **durability_contract_failure_relation_integrity_plan_mismatch_is_explicit** (231) — **HAPPY**. Separately declared receiver relation cardinality contract plan differs; exact RelationIntegrityPlanRevision family/contract IDs mismatch.

### contracts/semantic_result/bulk.rs

- **semantic_result_bulk_entities_and_relations_seal_and_readmit_original_bindings** (63) — **HAPPY + C2**. Genuine bulk32 entities/31 relations yields exact original key/ID/endpoint binding correspondence and readmitted semantic result. C2 exact original symbol dictionary includes bulk-input-00.
- **semantic_result_bulk_malformed_correspondence_refuses_after_frame_reseal** (109) — **C4**. Changes semantic correspondence in actual frame then reseals; frame succeeds, position admission refuses conflicting semantic result.

### contracts/semantic_result/custom_versions.rs

- **semantic_result_preserves_custom_enforcement_version_distinct_from_aggregate_request** (60) — **HAPPY + C4**. Genuine installed custom invariant callback observes proposed effect version distinct from aggregate predecessor; exact recorded callback/version readmission HAPPY. Second half modifies semantic enforcement record/reseals actual bytes and asserts refusal C4.

### contracts/semantic_result.rs

- **semantic_result_discovery_must_be_present_in_each_actual_index_entry** (13) — **owner codec**. Actual index entry explicitly carries Unassociated and serde roundtrip equal; remove association_discovery key, deserialize must fail naming missing field. This is strict value encoding, not recovery authority/fault campaign. Use genuine Native Memory-created entry; no physical durable claim.

### contracts/tail_checkpoint_validation.rs

- **durability_contract_recovery_rejects_tail_target_without_immutable_artifact** (6) — **C4**. Mutates tail target descriptor to missing immutable artifact; CorruptCheckpoint recovery root unavailable, second commit not installed.

### medium/branch_transition_races.rs

- **durable_fork_and_source_commit_complete_without_deferral** (67) — **C4 + C2**. Real slow publication plus publication_cell_for_test rendezvous forces fork/source-commit lock schedule; exact contiguous commit/transition streams and no deferrals. Preserve gate/contact/control refs; C2 exact deferral work counter.
- **durable_fork_and_source_archive_complete_in_lock_order** (106) — **C4 + C2**. Forced slow publication races fork/source archive and waits deterministic gate; exact stream sequence, no deferrals. Existing private/gated schedule campaign.
- **durable_fork_waits_for_another_branch_commit** (137) — **C4 + C2**. Blocks another branch commit at medium gate while fork waits; afterwards exact follows=2/contiguous streams, no deferrals.
- **memory_commit_waits_behind_a_transition_without_consuming_its_candidate** (168) — **C4 + C2 (Memory)**. pause_publication_for_test on LiveBranchTransitionStep forces Memory reservation schedule; candidate later publishes position2, counter deferrals0. Already Memory, no provider needed for this root.

### medium/checkpoint_member_storage.rs

- **filed_external_members_survive_checkpoint_replacement_and_obey_read_limits** (7) — **HAPPY**. Genuine ExternalMembers checkpoint replacement retains prior actual member; read_member with limit0 gives exact LimitExceeded and locator.length returns original bytes. Preserve real filed checkpoint/source.

### medium/durable_before_live.rs

- **a_refused_publication_was_offered_to_the_medium_and_moved_nothing_live** (63) — **C4**. Scripts ProvenNoEffect, exact live footprint unchanged, offered position2/head1, pending index-definition frontier remains and next accepted envelope carries original definition delta. Preserve private oracle; no ordinary success-only replacement.
- **a_refused_position_is_not_consumed** (122) — **C4**. Scripts refused publication then real acceptance; position reused2, burned candidate commit ID never reissued and correct parent/head.
- **every_acknowledged_commit_is_the_artifact_the_medium_holds_at_its_position** (147) — **HAPPY**. Each acknowledged actual publication is read back from the same medium's positions1..3 and readmits exact commit/position.
- **a_memory_only_runtime_publishes_without_a_medium** (175) — **HAPPY Memory + owner posture**. Memory runtime publishes normally and retains one canonical log entry. attached_durable_medium(None) is only adapter discovery assertion: retain exact owner durability.medium().is_none() in owner unit or public medium offered-count companion; no physical durability claim.

### medium/embedded_cost.rs

- **embedded_restore_allocation_grows_linearly_in_relation_bearing_regions** (8) — **C2**. stats_alloc child-process probe compares restore allocation for four times relation-bearing regions; wide<=narrow*7+256KiB, child marker required. Keep measured_restore and original process/global allocator closure, not a renamed happy restore.

### medium/lifecycle_lock_order.rs

- **fork_against_checkpoint_capture** (87) — **C4 + C2**. exclusive_before_transition pauses actual exclusive snapshot admission and observes transition contact; checkpoint vs fork schedule completes without deadlock, deferrals0.
- **fork_against_runtime_fork** (91) — **C4 + C2**. Same deterministic private pause/contact schedule, runtime fork vs branch fork; bounded completion and deferrals0.
- **fork_against_reclamation** (95) — **C4 + C2**. Same schedule, index reclamation vs branch fork; completion and deferrals0.
- **archive_against_checkpoint_capture** (99) — **C4 + C2**. Same schedule, checkpoint vs archive; exact private rendezvous and bounded completion.
- **archive_against_runtime_fork** (103) — **C4 + C2**. Same schedule, runtime fork vs archive; preserve exact gate and contacts.
- **archive_against_reclamation** (107) — **C4 + C2**. Same schedule, reclamation vs archive; no deadlock and deferrals0.
- **checkpoint_registry_is_readable_while_a_fork_is_durable_not_yet_live** (112) — **C4/private schedule**. SlowFirstPublication gate holds transition durably-before-live; private branch_cells_snapshot/lifecycle_snapshot remain readable with one original cell and eventually one transition. Correctness schedule, not timing/growth C2 merely timeout.

### medium/lineage_frontiers.rs

- **checkpoint_restore_protects_published_lineage_identities_in_each_lane** (45) — **HAPPY + C2**. Image(Memory)/Embedded/Cold actual checkpoint lanes; postrecovery published lineage IDs never reused. C2 exact discarded-candidate issuance frontiers, restored durable_floor and cold nodes payload empty; retain lane distinctions and source.
- **inflight_tail_replay_resumes_after_the_largest_durable_lineage_values** (132) — **HAPPY + C2**. Actual prepared candidate publishes after checkpoint while later candidate was discarded; tail replay restores next lineage IDs beyond actual durable values. C2 exact internal identity_frontiers; semantic emitted event/target non-reuse public companion.

### medium/recovered_publication.rs

- **a_recovered_runtime_publishes_its_next_commit_at_the_next_position** (9) — **HAPPY + C2**. Public original cold commit read/size refusal, no republish during recovery, coverage checkpoint2+tail1, next position4/parent, second reopen4. C2 Arc::ptr_eq original and reopened cold source proves same concrete source allocation.

### medium/recovery_refusals.rs

- **an_artifact_whose_frame_is_damaged_is_refused_at_the_read** (48) — **C4**. Damages frame bytes in journal; exact DigestMismatch at source read.
- **an_artifact_that_does_not_readmit_refuses_as_corrupt** (65) — **C4**. Corrupts commit header with intact frame; CorruptCanonicalArtifact at readmission.
- **a_gap_in_the_journal_refuses_as_non_contiguous** (80) — **C4**. Drops middle journal artifact; NonContiguousCanonicalTail.
- **a_repeated_entry_refuses_as_non_contiguous** (94) — **C4**. Repeats journal entry; NonContiguousCanonicalTail.
- **a_tail_that_does_not_continue_from_the_checkpoint_refuses** (109) — **C4**. Drops first tail position after checkpoint; refuses noncontiguous rather than skipping gap.
- **a_checkpoint_filed_under_the_wrong_position_refuses_as_corrupt** (128) — **C4**. Files genuine checkpoint under wrong position; CorruptCheckpoint.
- **a_damaged_checkpoint_refuses_as_corrupt** (154) — **C4**. Damages filed checkpoint payload; CorruptCheckpoint.

### medium/reservation_wait.rs

- **a_durable_publisher_waits_for_the_reservation_instead_of_deferring** (61) — **C4/private schedule + C2**. SlowFirstPublication and private reservation contact synchronize two publishers; positions2/3, exact offered positions. C2 deferrals0/assignments3.
- **a_cancelled_waiter_stops_before_any_effect** (104) — **C4/private schedule + C2**. Same actual gated reservation, cancel waiter after contact; no offered/live effect, retry succeeds position3. C2 exact deferrals0/assignments2.

### medium/single_owner.rs

- **a_forked_runtime_never_inherits_the_medium** (14) — **HAPPY + owner posture**. Actual runtime fork publishes Memory changes without offering to original medium; original positions/offers unchanged. Exact attached_durable_medium(None) becomes owner-local medium presence check, no C2.
- **recovery_moves_the_medium_into_the_rebuilt_runtime** (32) — **HAPPY**. Actual recover replaces runtime owner identity and next commit publishes position2 into original retained medium. Build receiver with explicit original medium; no late bind/discovery.
- **an_attachment_the_medium_moved_out_of_publishes_nothing** (60) — **owner binding state (medium required)**. Constructs private RelationalDurableMediumBinding around actual attachment, detach returns Some once/None next, publish refuses MediumMoved and original medium receives no offers. This is legal owner lifecycle unit, not C2 merely private; Memory cannot prove detach-once/no-offer. Requires genuine owner medium or public behavior split, never new local trait bridge.

### medium/unresolved_fate.rs

- **an_unresolved_publication_installs_nothing_and_refuses_every_later_commit_alike** (31) — **C4**. Scripts IndeterminateWithoutEffect; original fate position/commit/digest, live head/log/index/snapshot footprint unchanged, all later commits return same fate without another offer.
- **reopening_resolves_an_absent_unresolved_commit_as_never_committed** (100) — **C4**. Absent unresolved journal effect reopens as never committed; exact original position safely reused with checkpoint parent.
- **reopening_resolves_a_present_unresolved_commit_as_committed_exactly_once** (131) — **C4**. IndeterminateAfterEffect durable commit reopens exactly once; next position/parent correct and commit ID not reissued.

## Actual mounted helper closure

All 36 main-scope files defining non-test functions are named here. Other mounted files carry only the tests listed above or module mounts/imports. Helpers returning fixed-size arrays were included (not lost at the semicolon in their Rust return type).

- **contracts/associated_effects.rs**: `perform`. perform is genuine validate/bind association/prepare/compare-and-publish/settle; public counterpart already lives in testkit durable_publication::perform. Keep removed-route fault and residency oracle apart.
- **contracts/branch_root_checkpoint/foreign_targets.rs**: `replace_checkpoint_target`. replace_checkpoint_target is a C4 descriptor mutation; preserve exact original root/cell linkage and denial.
- **contracts/branch_root_checkpoint.rs**: `current_branch_entity_count`. current_branch_entity_count ports through admitted branch observation/read, while Arc-sharing/dedup assertions remain C2.
- **contracts/branch_root_content_binding.rs**: `checkpoint_recovery_plan`, `selected_root_image`, `recompute_partition_image_digest`, `install_fingerprint_image`, `selected_main_checkpoint`, `basis_target`, `current_main_descriptor`, `replace_descriptor`, `assert_corrupt_recovery_leaves_no_recovered_state`, `corrupt_recovery_error`. Plan/root selectors, descriptor replacement, image-digest recomputation, fingerprint injection and corrupt-recovery helpers are the C4 closure. current_main_descriptor can use actual public basis descriptor for the one unmodified semantic root.
- **contracts/branch_root_schema_binding.rs**: `checkpoint_plan_with_one_root`, `assert_corrupt_schema_recovery`, `assert_schema_recovery_denial`. checkpoint_plan_with_one_root takes actual source/capture; corrupt schema helpers preserve typed refusal/no effect. Physical carrier dedup C2; actual swapped carrier C4.
- **contracts/branch_root_shared_target_binding.rs**: `shared_root_plan`, `later_shared_root_cell`. shared_root_plan/later_shared_root_cell deliberately target later sibling copy in C4; do not replace with first root or ordinary foreign-runtime check.
- **contracts/branch_transition/fates.rs**: `publication_refused`, `normalize`, `apply`, `footprint`, `assert_blocked`. Action normalization/apply/footprint/assert_blocked form one scripted C4 state matrix; private footprint is the no-effect oracle, not justification for a public mutation bridge.
- **contracts/branch_transition/mod.rs**: `fork`, `artifacts`, `reopen`. fork/artifacts/reopen are shared by semantic, cost and fault roots. Public version keeps explicit medium and builds receiver with its attachment; Memory version calls retained-log recovery directly. Do not retain runtime-based discovery.
- **contracts/branch_transition/reconstruction.rs**: `replay`, `cold_source`, `read_transition`, `publish`, `publish_transition`. replay helper is ordinary; CountedTransitionReads trait implementation is read-cost instrumentation C2 and must not become a replacement local attachment. The exact read-gap observer belongs its C2 fixture.
- **contracts/branch_transition/tampering.rs**: `refusal`. refusal must consume actual corrupted source and fail for the expected recovery class; retain original intact-source guard.
- **contracts/caller_association.rs**: `propose`, `publish`, `transport`. propose/publish are ordinary owner-admitted flows; transport is exact artifact/readmission helper. Move public workflows onto existing testkit, keep pending-route/reservation census C2 and obsolete-byte checks C4.
- **contracts/canonical_artifact_neutrality.rs**: `branch`, `run_script`, `script_opening`, `script_close`, `transition_main_to_v2`, `artifacts`, `first_difference`, `enclosing_fields`, `assert_same_artifacts`. branch/script_opening/script_close/artifacts/comparer preserve full bytes. transition_main_to_v2 is positive owner schema fixture. Keep first_difference/enclosing_fields as diagnosis, not expected result; no actual fault solely for private registry construction.
- **contracts/canonical_commit_artifact/refusals.rs**: `exported_frame`, `write_u64`, `reseal`, `rewrite_payload`, `refusal`. exported_frame/write_u64/reseal/rewrite_payload/refusal preserve original actual frame and intended rejection layer; C4 bytes campaign.
- **contracts/canonical_commit_artifact.rs**: `memory_runtime_with_mixed_history`, `commit_schema_transition`, `exported_artifacts`, `commitment_value_bytes`. memory_runtime_with_mixed_history/commit_schema_transition/exported_artifacts are already actual owner Memory; commitment_value_bytes supports measured work C2. No provider needed for this whole file's roots.
- **contracts/declared_schema_binding.rs**: `registry`, `durable_runtime_declaring`, `reopen_declaring`, `assert_refused_as_declared_schema`, `transition_main`. registry is pure declaration; durable_runtime_declaring/reopen_declaring need explicit medium, not a new durable factory that hides it. transition_main is positive owner-private target registry fixture; assert_refused_as_declared_schema is typed semantic oracle.
- **contracts/derived_index_checkpoint/scoped_affinity.rs**: `fork_checkpoint`, `name_on_branch`. fork_checkpoint uses real shared parent basis, generation and checkpoint; name_on_branch ports through exact observation/read. Retain byte-rewrite/reseal in C4 for foreign unissued branch root.
- **contracts/derived_index_checkpoint.rs**: `name_index`, `scoped_name_index`, `build`, `build_on_branch`. name_index/scoped_name_index/build/build_on_branch are real owner index operations. Keep each test's actual generation IDs and original reader lifetime; no fabricated cache entries for HAPPY or C2.
- **contracts/fork_provenance_checkpoint.rs**: `fork_checkpoint_plan`, `fork_tail_plan`, `assert_recovery_rejects_fork_checkpoint`, `provenance_only`, `source_only`, `wrong_provenance_branch`. fork checkpoint/tail plan builders and provenance_only/source_only/wrong_provenance_branch are exact C4 mutations; assert_recovery_rejects_fork_checkpoint preserves target-empty denial.
- **contracts/lineage_artifact_recovery.rs**: `recover_error`. recover_error plus original published_lineage_mut_for_test/rebuild_lineage_basis_for_test calls are C4. Do not replace precise lineage corruption with a damaged frame.
- **contracts/managed_field_text.rs**: `commit_text`, `read_exact`. commit_text/read_exact use genuine managed JSON publication; public JSON parity ports. Retained managed text ownership, symbols and charged bytes stay C2; explicit codec input checks stay owner/C4 as named.
- **contracts/mod.rs**: `schema_transition_for_subscriber_impact`, `verified_at_digest_parity_value`. schema_transition_for_subscriber_impact constructs ProposedSchemaTransition value, verified_at_digest_parity_value constructs diagnostic expected value. These helpers neither create runtime nor publish; sharing them does not make roots one disposition.
- **contracts/native_checkpoint/allocation.rs**: `request`. request creates actual funded/cancelled Execution requests; keep with memory ownership and last-alias release C2 oracle.
- **contracts/native_checkpoint.rs**: `descriptor`, `prepare_scope`, `evaluate`, `durable_runtime_with_builder_invariant`. NativeCheckpointRule descriptor/prepare_scope/evaluate are real registered custom invariant; private unsealed-state unit can use same builder.custom_invariant(...).build(). Public capture/restore/editability ports retain original medium at build.
- **contracts/native_checkpoint_restore_benchmark.rs**: `buffered_digest`. buffered_digest is the independent buffered SHA oracle; retain exact tag, length prefix, 8192 records, chunks4096, 16 samples, Instant intervals and ignored reason together C2.
- **contracts/recorded_component_checkpoint.rs**: `current`, `capture`, `original`, `original_with_retained_successor`, `recover`, `original_custody`. current/capture use real ports; Original must retain medium explicitly. original_custody includes real unrelated-source drop, residency, private basis demand, retirement busy helper and terminal release accounting; split C4/C2 claims before using public recorded_component_recovery/validation analogues.
- **contracts/semantic_result/bulk.rs**: `publication`. publication creates actual entity/relation chain; original semantic IDs/key/endpoint correspondence remains public, dictionary assertion C2, malformed serialized correspondence C4.
- **contracts/semantic_result/custom_versions.rs**: `descriptor`, `prepare_scope`, `evaluate`. ObserveProposed rule descriptor/prepare_scope/evaluate records actual callback versions. Preserve real callback vector as independent producer evidence, not an invented semantic record.
- **medium/branch_transition_races.rs**: `runtime`, `await_condition`, `assert_streams`. runtime returns runtime+contents+gate; await_condition guarantees gate release on failure; assert_streams includes C2 deferral count. Original private publication_cell/pause refs remain C4; no scheduler sleeps as ordering.
- **medium/durable_before_live.rs**: `live_footprint`, `commit_one`. commit_one is real admitted transaction. live_footprint preserves head/log/patch/live entities plus actual snapshot count/capacity as C4 no-effect oracle; public happy ACK root can reuse external testkit with explicit medium.
- **medium/embedded_cost.rs**: `measured_restore`. measured_restore builds 8 vs32 regions, 128 entities per region plus relations, measures process allocator in exact-name child; preserve child marker/global allocator harness, not just restored truth.
- **medium/lifecycle_lock_order.rs**: `exclusive_before_transition`. exclusive_before_transition owns original pause_exclusive_snapshot_for_test/contact rendezvous and bounded completion. Keep schedule labels/checkpoint/fork/reclamation matrix; private lifecycle snapshot is ordinary state evidence, timeout alone is not cost C2.
- **medium/lineage_frontiers.rs**: `discard_after_a_durable_anchor`. discard_after_a_durable_anchor genuinely prepares/discards candidate; preserve Image/Embedded/Cold lanes, exact spent vs durable frontier and actual later published IDs.
- **medium/recovery_refusals.rs**: `runtime_with_commits`, `refusal_class`. runtime_with_commits creates actual journal, refusal_class checks source/recovery intended class. Keep each concrete corrupted-source action, no all-errors-are-OK oracle.
- **medium/reservation_wait.rs**: `slow_runtime`, `await_second_contact`. slow_runtime plus await_second_contact synchronize real contention; exact offer/position effects and cancelled retry guards remain. Counters C2; private contacts/gate C4.
- **medium/slow_publication.rs**: `await_publication`, `pass`, `arm`, `open`, `wait_until`, `cold_source`, `read_transition`, `publish`, `publish_transition`. SlowFirstPublication and PublicationGate are existing local trait delegator/gate, used by races/reservation/lifecycle tests. They must not survive as a renamed replacement local publisher. External testkit has scripted fates; scoped search did not establish an existing matching publication gate API.
- **medium/unresolved_fate.rs**: `commit_one`, `main_head`. commit_one/main_head preserve original offered artifact/fate and actual head; port public fault workflows with retained external medium, private footprint stays owner oracle.

## Shared local adapter closure and allowed existing reuse

Current actual local path is tests/support/durable_medium/runtime_attachment.rs:
- TestDurableMediumAttachment implements the local RelationalDurableMedium, forwarding initial input publication, selection, cold source, transition/commit publication and in_crate_test_medium.
- AttachedTestMedium wraps Arc<TestDurableMedium> and exposes recovery_source()/attachment(); attached_durable_medium(runtime) discovers it through runtime.durability.medium().in_crate_test_medium(); attached_test_medium assumes it exists.
- bind_durable_medium mutates freshly built runtime state via attach_medium. All of these are retirement targets, not patterns for new helpers.

tests/support/runtime.rs::durable_runtime_with_test_schema[_profile], durable_runtime_with_declared_aspect_schema and custom constructors in declared_schema_binding/native_checkpoint create the hidden local medium. Existing runtime_with_test_schema[_profile] is honest Memory; do not add another Memory facade with a durable name.

tests/support/durability.rs carries the transitive discovery:
- checkpoint_into_medium captures then calls attached_test_medium.file_checkpoint;
- durable_recovery_source discovers source;
- medium_recovery_plan binds receiver then reads source;
- checkpoint_durably checks attached_durable_medium to choose whether to file native capture;
- recovery_plan_for chooses source vs retained Memory log by discovery;
- checkpoint_and_recover_with -> recover_with chooses and late-binds similarly.
Replace each public caller's workflow with explicit original medium/source parameters. Local Memory units use the direct production retained-log methods without runtime-type discovery.

tests/support.rs::capture_inspection_truth_bundle contains graph/kind/connectivity/history/branch/latest/recent commit observations **and** retention_summary/record_retention. The latter remain exact C2 claims, not ordinary visible-truth-only comparisons. Use historical_latest_commit for its legacy latest_commit alias.

The manifest already has worth-testkit-relational as a dev-dependency. That does **not** make its typed Native attachment an implementation of the local cfg(test) Native trait: these are two compilation identities. durable_medium/bytes.rs explicitly provides identity-neutral ByteJournal (u64 -> original bytes) and MemberBytes. Reusing those byte stores is allowed existing dependency direction; wrapping them in another local RelationalDurableMedium is exactly the bridge this task removes. External court tests and external testkit share the shipping Native identity and can reuse the actual typed attachment directly.

## Additional concrete owner roots (24)

Paths below are relative to crates/worth-relational/src/. Mount evidence: native_checkpoint.rs -> tests.rs -> selected_ack; component_custody.rs -> tests; checkpoint_complete.rs -> tests.rs -> four test children; checkpoint_transition.rs -> tests; runtime_rebuild checkpoint restore child; native_checkpoint_codec.rs -> kind_authentication_tests.

### durability/data/native_checkpoint/tests/selected_ack.rs

- **selected_ack_partial_transfer_preserves_later_cells_and_resumes_same_capture** (24) — **C2 + interruption campaign**. work_limited_ack_request(1) stops actual acknowledge_selected after first covered cell; later covered cells and postcut cell remain untouched, directory not installed, same capture Arc and source-placement owners retained on retry. Real medium selection is necessary; Memory ACK would be CaptureNotBound/NotSelected.
- **selected_ack_exposes_backed_data_before_retirement_and_releases_holds_only_on_completion** (119) — **C2 + interruption campaign**. Budget stops after DATA placement before retirement: selected manifest/frontier visible, completed frontier still old, prior directory and captured basis Weak owners retained. Pending new definition publishes correctly; resume same capture releases old directory/basis only at complete ACK, later cell untouched. Preserve every pointer/frontier/custody assertion.

### durability/data/native_checkpoint/component_custody/tests.rs

- **native_custody_correction_delayed_canonical_selection_retains_actual_root_and_interrupted_ack** (101) — **C2 + interruption campaign**. delayed_selection(false) retains actual original canonical root/basis/schema through original lease release, real filing, interrupted ACK and observer drop; only successful ACK releases basis then reclamation frees root.
- **native_custody_correction_delayed_empty_selection_retains_actual_schema_and_interrupted_ack** (105) — **C2 + interruption campaign**. Same delayed_selection(true) for actual Empty component/schema; do not replace with canonical-root-only case or fabricated basis.
- **native_custody_correction_last_actual_capture_abandonment_releases_shared_root_custody** (109) — **C2**. Two real aliases of unselected capture keep actual original basis alive; dropping last actual capture releases it, no explicit ACK. Exact Weak owner lifecycle.

### durability/authority/checkpoint_transition/tests.rs

- **acknowledged_transition_admits_only_its_exact_successor_and_releases_snapshot** (59) — **HAPPY + C2**. Recovered authority acknowledges exact successor, one new immutable/log commit and accepts successor/rejects predecessor. C2 exact zero published snapshot handle/capacity counters after transition. Actual public prepare/compare/acknowledge APIs already have court analogues.
- **foreign_recovery_and_foreign_candidate_are_refused_before_effects** (113) — **HAPPY + C2**. Genuine foreign recovery runtime/candidate denied before effects, original authority recoverable and valid candidate succeeds. C2 zero live snapshot counts; no forged proof injection.
- **moved_source_and_stale_candidate_cannot_extend_recovery_lineage** (177) — **HAPPY + C2**. Genuine moved source/stale candidate refused; exact before descriptor/commit count preserved and predecessor authority remains valid. C2 zero residual published snapshots.
- **refused_durable_transition_returns_its_authority_and_moves_nothing** (212) — **C4**. Scripts ProvenNoEffect for recovered transition; returns original authority, no head/log/snapshot effect, retry commits position2. Preserve exact script and returned-authority custody.
- **unresolved_durable_transition_installs_nothing_and_names_its_fate** (254) — **C4**. Scripts indeterminate recovered transition; no live state, exact offered position/digest named in original unresolved fate and no snapshot leak.

### durability/authority/runtime_rebuild/checkpoint_restore_tests.rs

- **equal_exact_root_reuses_the_readmitted_partition_substrate** (12) — **C2**. Calls owner restore_branch_root_images/prepare_partitions and pointer-compares readmitted first generation: equal exact Root reuses physical substrate. Actual durable/codec fixture, not public identity parity.
- **native_restore_work_counts_revised_history_sibling_roots_and_partition_images** (35) — **C2**. NativeRestoreWork exact byte/envelope/root/partition/cell/index counts under history revisions/sibling root and extra partition. Captured section growth plus pinned reader correctness guard.
- **shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write** (132) — **HAPPY**. Only actual old snapshot reads1 after next publication while latest reads2; no pointer/cost census in root. Snapshot retention semantics can port through public snapshot/read API despite pinned wording.
- **divergent_sibling_is_not_reused_as_the_storage_mirror** (179) — **HAPPY + C2**. Public recovered main/sibling truth2 each. C2 owner mirror generation shares main exactly and does not share divergent sibling.
- **divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse** (235) — **C4 + C2**. Directly increments main-image snapshot_pins[0] to force divergent mirror; private perpartition generation/pointer reuse assertions. Keep mutation and physical reuse oracle together.
- **recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads** (299) — **HAPPY/owner immutable catalog**. Original vs recovered catalog identities/parentage/roots/canonical payload digest equal for main/sibling. No ptr_eq asserted; not C2 just private commit_artifact. Public original artifact readmission and history immutable identity/receipt/basis can cover semantic parts; exact catalog digest accessor assertion can remain owner unit with genuine source.
- **equal_image_cannot_bypass_global_contract_readmission** (348) — **C4**. Clears global aspect contracts while exact root/mirror images equal; prepare_partitions must refuse aspect readmission, never bypass due to reuse.

### durability/log/native_checkpoint_codec/kind_authentication_tests.rs

- **checkpoint_readmission_refuses_retired_integer_array_digest_before_restore** (13) — **C4**. Rewrites actual digest binary representation to retired integer array; decode refuses CorruptCheckpoint before restore.
- **forged_cross_region_kind_commitment_is_refused_before_member_reads** (55) — **C4**. Rehashes all copies of forged cross-region kind commitment; owner canonical root refuses StorageContentMismatch before zero member reads. Existing ForgedKindSource is a fault source, not license for a replacement happy publisher.

### durability/authority/checkpoint_complete/tests/empty_schema.rs

- **complete_empty_archive_and_initial_installation_preserve_unused_original_schema** (9) — **HAPPY + C2 (Memory)**. Original empty archive + real initial schema installation preserves unused original physical constraint. C2 actual old-root/schema Arc change and Weak retirement, exact complete manifest/schema image roster, funded output alias retains charge until last drop. Already Memory, no attachment dependency in this root.
- **complete_refuses_recovered_empty_without_original_birth_authority** (130) — **HAPPY**. Genuine ordinary native restore of Empty lacks original birth authority; public Complete prepare refuses OriginalAuthorityUnavailable. Preserve distinction ordinary restore vs genuine Complete source; no promotion of ordinary checkpoint.

### durability/authority/checkpoint_complete/tests/partial_aspect_update.rs

- **complete_preserves_partial_update_without_demanding_untouched_aspect_reference** (11) — **HAPPY + C2**. Public Complete all-prefix fold must keep actual name-only update and untouched lifecycle binding. C2 exact revision_symbol_references membership, actual original resident arena lifecycle/axes before/after, decoded authoritative value and dictionary symbol resolution. No fabricated expected root.

### durability/authority/checkpoint_complete/tests/pinned_delete.rs

- **complete_fold_matches_original_relation_delete_with_snapshot_and_replay_pins** (9) — **HAPPY + C2**. Real snapshot/replay pins coexist with relation delete; Complete contains genuine current deleted-retained root and externally retained creation root. C2 actual root Arc/axes, mirror pin counts>0 vs immutable arena snapshot/replay counts0, exact lifecycle/retired_at physical columns and original axes unchanged. Preserve independent original arena oracle.

### durability/authority/checkpoint_complete/tests/request_control.rs

- **complete_original_request_denial_and_cancel_preserve_original_empty_source** (8) — **HAPPY + C2 (Memory)**. Actual public weak Complete service refuses policy memory0/cancel and later succeeds. C2 exact no charged memory, original Root ptr_eq and captured_sections total>0. Already Memory, not provider causal.

The selected ACK helper work_limited_ack_request, component_custody::delayed_selection, checkpoint_transition::{recovered_world,candidate,main_basis}, checkpoint_restore::{shares_first_generation,shares_partition_generation}, and ForgedKindSource method closure are part of those rows. Keep exact source/capture/root ownership, not only final head equality.

## Public signatures that already exist

Production signatures, with facade types shortened:
```rust
RelationalRuntimeBuilder::build_durable(
    self, medium: Box<dyn RelationalDurableMedium>
) -> Result<RelationalRuntime, RelationalRuntimeConstructionDenial>;

DurabilityAccess::recovery_plan(
    &self, execution: &ExecutionRequest<'_, '_>, mode: RecoveryVerificationMode
) -> Result<RecoveryPlan, RelationalRecoveryStop>;
DurabilityAccess::recovery_plan_from_source(
    &self, execution: &ExecutionRequest<'_, '_>,
    source: RelationalDurableRecoverySource, mode: RecoveryVerificationMode
) -> Result<RecoveryPlan, RelationalRecoveryStop>;

DurabilityRecoveryAuthority::recover(
    &mut self, execution: &ExecutionRequest<'_, '_>, plan: RecoveryPlan
) -> Result<RecoveryOutcome, RelationalRecoveryStop>;
DurabilityRecoveryAuthority::recover_with_authority(
    &mut self, execution: &ExecutionRequest<'_, '_>, plan: RecoveryPlan
) -> Result<(RecoveryOutcome, RecoveredRelationalRuntimeAuthority), RelationalRecoveryStop>;
DurabilityRecoveryAuthority::restore_native_checkpoint(
    &mut self, execution: &ExecutionRequest<'_, '_>, checkpoint: &RelationalNativeCheckpoint
) -> Result<RecoveryOutcome, RelationalRecoveryStop>;
DurabilityRecoveryAuthority::restore_native_checkpoint_with_authority(
    &mut self, execution: &ExecutionRequest<'_, '_>, checkpoint: &RelationalNativeCheckpoint
) -> Result<(RecoveryOutcome, RecoveredRelationalRuntimeAuthority), RelationalRecoveryStop>;
DurabilityRecoveryAuthority::commit_checkpoint_transition(
    &mut self, execution: &ExecutionRequest<'_, '_>,
    recovered: RecoveredRelationalRuntimeAuthority, candidate: PreparedRelationalCommitCandidate
) -> Result<RecoveredCheckpointTransition, RecoveredCheckpointTransitionError>;

DurabilityAuthority::native_checkpoint(
    &self, execution: &ExecutionRequest<'_, '_>, policy: ExecutionAllocationPolicy<'_, '_>
) -> Result<RelationalNativeCheckpoint, RelationalNativeCheckpointCaptureDenial>;
DurabilityAuthority::native_checkpoint_with_policy(
    &self, execution: &ExecutionRequest<'_, '_>, policy: ExecutionAllocationPolicy<'_, '_>,
    artifact_policy: RelationalCheckpointArtifactPolicy<'_>
) -> Result<RelationalNativeCheckpoint, RelationalNativeCheckpointCaptureDenial>;

RelationalRuntime::complete_checkpoint_service(&self) -> CompleteRelationalCheckpointPreparation;
CompleteRelationalCheckpointPreparation::prepare(
    &self, execution: &ExecutionRequest<'_, '_>,
    artifact_policy: RelationalCheckpointArtifactPolicy<'_>
) -> Result<CompleteRelationalNativeCheckpoint, RelationalNativeCheckpointCaptureDenial>;
RelationalNativeCheckpoint::acknowledge_selected(
    &self, execution: &ExecutionRequest<'_, '_>, context: &mut MapKernelContext<'_, '_>
) -> Result<(), RelationalCheckpointAcknowledgementDenial>;

RelationalTransactionIntent::with_schema_transition(
    self, transition: ProposedSchemaTransition, policy: Option<SchemaReconciliationPolicy>
) -> Self;
```

Sources: runtime/builder/mod.rs:232; durability/access/recovery_plan_entrypoint.rs:20/35; durability/authority/recovery.rs:29/61/91/102; authority/checkpoint_transition.rs:52; authority/checkpointing.rs:13/28; runtime/state/runtime_state/complete_checkpoint_preparation.rs:19/26; durability/data/native_checkpoint.rs:183; mvcc/transaction/intent.rs:153.

Read selected heads using branch_identity -> observe_branch -> history().branch_head_for_observation(&basis.observation()) -> Result<Option<RelationalCommitReceipt>, RelationalBranchBasisDenial>. historical_latest_commit(&self)->Option<RelationalCommitReceipt> is immutable global-latest identity, not a branch selector. read_truth().try_read_version(&ExecutionRequest, VersionId)->Result<Result<RelationalReadView,HistoricalVisibilityDenial>,RelationalExecutionDenialCause> has **two** Result layers. Public current index authority uses read_published_generation_for_observation; semantic lineage mapping uses lineage_access().for_record; private retained-map/allocator/pointer checks remain owner/C2. Production merge planning/preparation requires bind_merge_planning_request/bind_merge_execution_request, not in-crate raw-request overloads.

External TestDurableMedium::new()->Arc<Self>, attachment(&Arc<Self>)->Box<dyn RelationalDurableMedium>, recovery_source()->RelationalDurableRecoverySource, file_checkpoint(RelationalNativeCheckpoint), offered()/offered_transitions(), committed_artifacts(), artifact_bytes(), scripted publication/transition fates already exist. Unlike the local wrapper, external recovery_source returns its source directly. Keep actual Arc in caller/testkit Original; never rediscover it from runtime.

## Existing mounted external analogues

Base directory: crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/. Its mod.rs mounts these ordinary proofs already; no new test target is needed.
- associated_effects.rs, recovered_associated_effects.rs, recorded_component_recovery.rs and recorded_component_validation.rs use real original association/source/recovery authority. recorded_component_recovery first root admits actual Empty/historical subjects, re-exports original descriptors, tests foreign-runtime refusal and does not substitute latest root.
- recorded_component_checkpoint.rs exercises cold original subject/source fault and actual successor derivation. It uses external testkit Original/capture/recover handles already; private Weak owners and terminal accounting in Native are additional C2 evidence.
- semantic_result.rs is existing original semantic callback/correspondence/readmission evidence; bulk/custom-version public parts can share that shape. Keep exact dictionary and resealed malformed correspondence in their assigned lanes.
- branch_history_recovery.rs::checkpoint/recover/current_head carries Arc<TestDurableMedium>, files genuine native capture, reads medium.recovery_source() and observes selected heads. branch_checkpoint_journeys.rs already compares historical_latest_commit and branch reopen truth.
- recovered_publication.rs and result_snapshot_cleanup.rs are actual publication/recovered-authority/snapshot closeout shapes, not a fresh medium discovered from runtime.
- index_catalog/recovery.rs:214/315 demonstrates real file_checkpoint -> acknowledge_selected under request.run_native_with_checkpoint. It distinguishes stale capture vs actual selected definition frontier. index_catalog/wire.rs/definition_lookup_snapshot.rs provide existing semantic index rows.
- source_initial_inputs/checkpoint_admission/operational.rs:139 and capture_cost.rs:25 show actual Complete/source capture ACK. cold_materialization/selected_image.rs, retained_indexes.rs, indexes.rs and root_projection.rs exercise original image/currentness/cache behavior through public source admission. These cover ordinary producer/selection/readmission; they do not prove Native's private pointer/pin or partial-ACK progress fields.
- worth-testkit-relational/src/durable_publication.rs and checkpoint/recovery/recorded_assertions submodules already own genuine schema, source, captured checkpoint, actual original references and assertions. Reuse those instead of copying Native private support across the facade.

## Exact remaining private boundaries and observation gaps

**Real partial ACK, not Memory selection.** selected_ack's two roots and delayed_selection(false/true) prove concrete source-placement/capture owners, partial phase progress and release only upon completed actual ACK. In medium/binding/checkpoint_selection.rs:94-151, Captured phase upgrades the actual binding and observe_checkpoint_selection queries the medium; only Selected constructs ActualCheckpointSelection and enters Acknowledging. Native checkpointing binds publication to runtime.durability.medium() (authority/checkpointing.rs), and acknowledge_selected refuses CaptureNotBound when publication is absent. Replacing a durable factory with Memory therefore cannot preserve these exact tests. Public Complete/ACK workflows above cover real external selection, but not per-cell source-placement same_owner, capture Arc::ptr_eq, prior directory Weak retirement or exact pending/completed index frontiers. Preserve those C2 owner claims with a real owner input/court cutover; do not retain a renamed local publisher or pass a caller-supplied 'selected' receipt as actual selection.

**Detach-once binding owner unit.** medium/single_owner.rs:60 creates the private binding, detaches the medium Some once/None next and proves MediumMoved/no offers. Public fork/recover workflows cover external behavior, but do not make this exact owner state assertion unnecessary. Memory has no medium to detach. No source-only, assertion-preserving replacement for this local provider was established in this scoped review. This is a concrete owner-input requirement, not a missing public authority door.

**Positive schema target installation.** Public intent.with_schema_transition exists. Scoped searches in runtime/state/runtime_state and schema found private set_schema_registry_for_test/SchemaContinuityAuthorityInput target registry, initial installation, and normal public intent; no general public live target-registry replacement recipe was established. Thus positive canonical encoding/schema-history fixtures may remain true owner units with actual Memory history where their claims are local. Their real journal/reopen assertions must stay genuine externally or in the owner's real durability lane. Do not misclassify positive setup as corruption, or add an operational authority door just to move a test.

**Per-envelope original input attribution.** merge_replay_continuity::durability_contract_replays_merge_from_typed_authority_when_diagnostics_are_absent calls assert_durable_commit_meaning_eq. tests/support/truth_comparison.rs compares recorded_original_inputs() separately from full body meaning. Public envelope Serialize includes every body field+semantic_result, so external serialized projection can preserve all body equality while omitting exactly diagnostic entries and derived index cache. It does not include the retained per-envelope attribution Option. That private record stores optional result_image, prestate_schema, result_schema and MutationConfig. If that exact subclaim must port, the minimal missing immutable observation belongs to history/replay and returns original per-envelope attribution DATA only, never authority/mutation. Existing CompleteSourceArtifact::input_commitment_leaves_for_test observes normalized source leaves under a sealed caller boundary; it is not a drop-in retained-envelope observation. Do not compare None/None in Memory and call attribution preserved.

Other private semantics (custom-invariant unsealed flag, strict index serialization, exact compiled/schema/canonical values) need no new public observation if kept as owner units. Root/pin/dictionary/allocation identity/cost assertions need their C2 owner evidence, not broad new production exposure. Malformed plan/envelope controls belong C4, not new public mutation APIs.

## Finite scope limitations

All 191 main roots and 24 requested owner-extension roots have a claim disposition; **0 unclassified**. This does not establish that every C2/C4/private physical fixture can already compile after adapter deletion. The precise unsatisfied owner selection/detachment inputs above remain review findings. No source mutation, compilation, test run, boundary gate or Complete ACK completion is claimed.

Other directly discovered durability/runtime owner files (checkpoint concurrency, fresh source, native codec inspection/manifest integrity, published snapshot cleanup, symbol reservations and retained export skip tests) are outside this finite contracts/medium + named Complete/ACK extension. They are not silently certified by this artifact.

