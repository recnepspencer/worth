# Signal and Bridge authored recovery evidence

Authority: [Milestone 3](milestone-3.md), Phase 5 and Delivery tracks, track H.
This map separates the preparatory byte contracts from operational readmission.

| Requirement | Enforcement or named court | Status |
|---|---|---|
| R2: static declarations survive evaluation independently of discovered edges | `authored_bytes_exclude_evaluated_edges_versions_and_node_state` | Passing public Signal graph court |
| R2: retirement removes authored edges even after discovered adjacency changes | `retiring_a_source_removes_declarations_after_evaluation_changes_the_read_set` | Passing; includes slot reuse, serde memory reconstruction and checkpoint memory reconstruction |
| R2: declaration cycles cannot hide behind a changed discovered read set | `declaration_cycles_refuse_when_evaluation_has_removed_the_runtime_edge` | Passing |
| Phase 5: explicit versioned, checksummed Signal byte format | `authored_image_has_explicit_empty_payload_and_refuses_unknown_versions`; `checksum_and_exact_length_cover_the_authored_frame` | Passing graph-section codec courts |
| Phase 5: authored policy and declaration fields are retained | `definition_policy_changes_are_source_even_when_cached_versions_match`; private `declaration_codec_preserves_nondefault_contracts_schema_policies_and_scopes` | Passing graph-section codec courts |
| D2: Signal owns the definition envelope, including an opaque Bridge-owned section | `definition_transition_retains_exact_authored_image_bridge_section_and_predecessor` | Passing descriptive artifact court; Bridge section interpretation pending |
| D5: original association and exact branch coordinates travel in branch bytes | `branch_effect_preserves_original_attempt_association_and_all_recorded_axes` | Passing descriptive artifact court; operational publication pending |
| D6/D17: Home-scoped descriptive identity vocabulary cannot carry process identity | Sealed `DurableIdentity` implementations; exact typed descriptor fields; `foreign_definition_owner_and_foreign_attempt_home_refuse_as_typed_data` | Passing descriptive artifact court; live issuer cutover pending |
| D2: wasm-safe port, shared three-way publication fate | `SignalDurableMedium` has no Store/filesystem dependency; `SignalPublicationCompletion` has private fields | Passing `worth-signal-wasm` check for `wasm32-unknown-unknown` with the port present |
| D17: durable before live | Actual medium-binding `DurablyCommitted` consumption | Pending; descriptive bytes mint no witness |
| Phase 5: multi-branch owner readmission, cold derived state and recovered-owner descriptor admission | Real Store-backed Signal owner close/reopen courts | Pending |
| Phase 5 / R4: Bridge conditional definitions, fresh lowering/retention and typed diagnostic absence | Bridge installation and replay courts | Pending |
| Phase 5: same/changed build, graceful close and quiescent kill | Store-backed owner courts | Pending |
| D15 / Phase 5: mixed publication and Home/World/Host admission | Granted integration seams and mixed-operation courts | Deferred to coordinator's exact seam grant |

The graph-section codec does not encode values, dirty/aspect versions, cause
sets, traces, telemetry, resource or temporal runtime. Its bytes are descriptive
source and do not admit a durable Signal owner. No Home reopen, mixed operation,
Store-backed owner recovery or oracle result is inferred from codec courts. The
wasm result comes from its separate target check.

Logs live under `target/lane-evidence/`. The first codec review rejected missing
authored retirement propagation; the reverse declaration index and the named
retirement court above address that finding. An approving review of the corrected
range is still required before a safe point is reported.
