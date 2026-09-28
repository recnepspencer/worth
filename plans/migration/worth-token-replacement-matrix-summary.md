# Worth Token Replacement Matrix Summary

Total WORTH substring occurrences: 2371

Review pass updated ambiguous decisions: 383
Remaining `medium`/`review` decisions: 0

## By Crate

| Crate | Occurrences |
|---|---:|
| `worth-query` | 182 |
| `worth-relational` | 214 |
| `worth-runtime-bridge` | 10 |
| `worth-signal` | 435 |
| `worth-signal-wasm` | 1530 |

## By Confidence

| Confidence | Occurrences |
|---|---:|
| `high` | 2281 |
| `keep` | 90 |

## By Category

| Category | Confidence | Occurrences |
|---|---|---:|
| `canonical_namespace_case_reviewed` | `high` | 149 |
| `proper_noun_punctuation_case_reviewed` | `high` | 1 |
| `const_or_env_keep` | `keep` | 30 |
| `enum_variant_case` | `high` | 61 |
| `forged_word_damage` | `high` | 568 |
| `camel_identifier_forged_case_reviewed` | `high` | 2 |
| `forgery_word_damage` | `high` | 23 |
| `forget_word_damage` | `high` | 8 |
| `internal_js_cache_sentinel_case_reviewed` | `high` | 12 |
| `internal_js_sentinel_case_reviewed` | `high` | 3 |
| `literal_route_fixture_keep` | `keep` | 33 |
| `media_type_namespace_case_reviewed` | `high` | 2 |
| `placeholder_path_case_reviewed` | `high` | 1 |
| `proper_noun_case_reviewed` | `high` | 107 |
| `protocol_slug_case_reviewed` | `high` | 44 |
| `repository_url_out_of_scope_keep` | `keep` | 5 |
| `rust_type_case` | `high` | 864 |
| `screaming_const_keep` | `keep` | 12 |
| `screaming_const_or_env_keep_reviewed` | `keep` | 10 |
| `snake_case_identifier_reviewed` | `high` | 3 |
| `test_path_case_reviewed` | `high` | 4 |
| `type_case` | `high` | 17 |
| `type_or_brand_case` | `high` | 403 |
| `type_or_camel_identifier_case_reviewed` | `high` | 3 |
| `typescript_error_sentinel_case_reviewed` | `high` | 6 |

## Reviewed Ambiguous Decision Rules

| Prior ambiguity | Matrix decision | Rationale |
|---|---|---|
| `WORTH` prose/product spelling | `Worth` / high | Capital-case product proper noun, not caps-lock token. |
| `WORTH.*`, `application/vnd.WORTH.*`, `WORTH-*` protocol namespaces | `worth.*`, `application/vnd.worth.*`, `worth-*` / high | Machine namespaces and protocol slugs should be lowercase canonical values. |
| `searchRoute:WORTH` fixture route values | keep / keep | Reviewed in context as literal test/domain route data, not product naming. |
| `https://github.com/recnepspencer/WORTH` | keep / keep | External repo-name casing is explicitly out of scope for this pass. |
| `__WORTH...` JS/TS sentinel fields | `__Worth...` / high | Preserve sentinel shape while removing random caps-lock product casing. |
| `WORTH_QUERY_*`, `WORTH_GRAPH_*` | keep / keep | Screaming-snake constants/env-like symbols are structurally uppercase. |
| `WORTH_scope` and trybuild `WORTH_` file path text | lowercase `worth_` / high | Rust function/file naming should follow snake_case lowercase. |

## Top Unique Token Replacements

| Count | Current token | Proposed token | Category | Confidence |
|---:|---|---|---|---|
| 463 | `WORTHSignalJsError` | `WorthSignalJsError` | `rust_type_case` | `high` |
| 239 | `WORTHSignalJsError::invalid_input` | `WorthSignalJsError::invalid_input` | `rust_type_case` | `high` |
| 107 | `WORTH` | `Worth` | `proper_noun_case_reviewed` | `high` |
| 105 | `crate::boundary::errors::WORTHSignalJsError` | `crate::boundary::errors::WorthSignalJsError` | `rust_type_case` | `high` |
| 64 | `_WORTHd` | `_forged` | `forged_word_damage` | `high` |
| 63 | `WORTHd_performance` | `forged_performance` | `forged_word_damage` | `high` |
| 50 | `WORTHd` | `forged` | `forged_word_damage` | `high` |
| 39 | `WORTHd_transition` | `forged_transition` | `forged_word_damage` | `high` |
| 32 | `searchRoute:WORTH` | `searchRoute:WORTH` | `literal_route_fixture_keep` | `keep` |
| 30 | `WORTHd_handle` | `forged_handle` | `forged_word_damage` | `high` |
| 23 | `WORTH-resource-external-v1` | `worth-resource-external-v1` | `protocol_slug_case_reviewed` | `high` |
| 23 | `WORTHSignalJsError::from` | `WorthSignalJsError::from` | `rust_type_case` | `high` |
| 21 | `WORTHSignalJsError::internal` | `WorthSignalJsError::internal` | `rust_type_case` | `high` |
| 21 | `WORTHd_summary` | `forged_summary` | `forged_word_damage` | `high` |
| 16 | `EffectAuthorityOwner::WORTHRelational` | `EffectAuthorityOwner::WorthRelational` | `enum_variant_case` | `high` |
| 15 | `WORTHd_denial` | `forged_denial` | `forged_word_damage` | `high` |
| 15 | `WORTHd_lifecycle` | `forged_lifecycle` | `forged_word_damage` | `high` |
| 15 | `WORTHd_row` | `forged_row` | `forged_word_damage` | `high` |
| 13 | `WORTHRelational` | `WorthRelational` | `enum_variant_case` | `high` |
| 13 | `WORTH_QUERY_GRAPH_OBLIGATION_DISPATCH_ENVELOPE_SCHEME` | `WORTH_QUERY_GRAPH_OBLIGATION_DISPATCH_ENVELOPE_SCHEME` | `const_or_env_keep` | `keep` |
| 12 | `WORTHries` | `forgeries` | `forgery_word_damage` | `high` |
| 11 | `WORTHRuntimeBridge` | `WorthRuntimeBridge` | `enum_variant_case` | `high` |
| 9 | `WORTHSignalResourcePatchBrand` | `WorthSignalResourcePatchBrand` | `type_or_brand_case` | `high` |
| 9 | `WORTH_SIGNAL_SKIP_TOPOLOGY_DEBUG_ASSERTS` | `WORTH_SIGNAL_SKIP_TOPOLOGY_DEBUG_ASSERTS` | `const_or_env_keep` | `keep` |
| 9 | `__WORTHSignal.outputProjection.panel.1` | `__WorthSignal.outputProjection.panel.1` | `type_or_brand_case` | `high` |
| 7 | `.__WORTHSignalCallbackCapture` | `.__WorthSignalCallbackCapture` | `type_or_brand_case` | `high` |
| 7 | `WORTHSignalJsError::callback_failure` | `WorthSignalJsError::callback_failure` | `rust_type_case` | `high` |
| 7 | `WORTHd_digest` | `forged_digest` | `forged_word_damage` | `high` |
| 7 | `WORTHd_node` | `forged_node` | `forged_word_damage` | `high` |
| 7 | `__WORTHSignalCallbackCapture:` | `__WorthSignalCallbackCapture:` | `type_or_brand_case` | `high` |
| 6 | `BlockedOnWORTHStore` | `BlockedOnWorthStore` | `enum_variant_case` | `high` |
| 6 | `WORTHd_bundle` | `forged_bundle` | `forged_word_damage` | `high` |
| 6 | `WORTHd_cancelled` | `forged_cancelled` | `forged_word_damage` | `high` |
| 6 | `WORTHd_packet` | `forged_packet` | `forged_word_damage` | `high` |
| 6 | `WORTHd_performance_closeout` | `forged_performance_closeout` | `forged_word_damage` | `high` |
| 6 | `WORTHd_retry` | `forged_retry` | `forged_word_damage` | `high` |
| 6 | `WORTHd_scenario_matrix` | `forged_scenario_matrix` | `forged_word_damage` | `high` |
| 5 | `WORTH-resource-external-delivery-v1` | `worth-resource-external-delivery-v1` | `protocol_slug_case_reviewed` | `high` |
| 5 | `WORTH.query.evidence-identity.v1:` | `worth.query.evidence-identity.v1:` | `canonical_namespace_case_reviewed` | `high` |
| 5 | `WORTHSignalCoreProfile` | `WorthSignalCoreProfile` | `type_case` | `high` |
| 5 | `WORTHSignalMaxAspects` | `WorthSignalMaxAspects` | `type_case` | `high` |
| 5 | `WORTHd_matrix` | `forged_matrix` | `forged_word_damage` | `high` |
| 5 | `WORTHd_rows` | `forged_rows` | `forged_word_damage` | `high` |
| 5 | `__WORTHSignal.outputProjection.audit.panel.1` | `__WorthSignal.outputProjection.audit.panel.1` | `type_or_brand_case` | `high` |
| 5 | `https://github.com/recnepspencer/WORTH` | `https://github.com/recnepspencer/WORTH` | `repository_url_out_of_scope_keep` | `keep` |
| 4 | `WORTH.query.evidence-identity.v1:WORTH.test.stable-digest-v1:` | `worth.query.evidence-identity.v1:worth.test.stable-digest-v1:` | `canonical_namespace_case_reviewed` | `high` |
| 4 | `WORTHSignalResourceContinuationPostureBrand` | `WorthSignalResourceContinuationPostureBrand` | `type_or_brand_case` | `high` |
| 4 | `WORTHSignalResourceProcessingJobPostureBrand` | `WorthSignalResourceProcessingJobPostureBrand` | `type_or_brand_case` | `high` |
| 4 | `WORTHd.digest` | `forged.digest` | `forged_word_damage` | `high` |
| 4 | `WORTHd_closeout` | `forged_closeout` | `forged_word_damage` | `high` |
| 4 | `WORTHd_ready_wake` | `forged_ready_wake` | `forged_word_damage` | `high` |
| 4 | `WORTHd_replay` | `forged_replay` | `forged_word_damage` | `high` |
| 4 | `WORTHd_witness_digest` | `forged_witness_digest` | `forged_word_damage` | `high` |
| 4 | `WorthQueryDeclarationEntryLowerOwnerCrate::WORTHSignal` | `WorthQueryDeclarationEntryLowerOwnerCrate::WorthSignal` | `type_or_brand_case` | `high` |
| 4 | `__WORTHInvalidApiRequestParams__:` | `__WorthInvalidApiRequestParams__:` | `typescript_error_sentinel_case_reviewed` | `high` |
| 3 | `EffectAuthorityOwner::WORTHRuntimeBridge` | `EffectAuthorityOwner::WorthRuntimeBridge` | `enum_variant_case` | `high` |
| 3 | `WORTH.query.declaration.v1` | `worth.query.declaration.v1` | `canonical_namespace_case_reviewed` | `high` |
| 3 | `WORTH.query.evidence-identity.v1` | `worth.query.evidence-identity.v1` | `canonical_namespace_case_reviewed` | `high` |
| 3 | `WORTHFormValueType:` | `WorthFormValueType:` | `type_case` | `high` |
| 3 | `WORTHSignalResourceDeliveryBrand` | `WorthSignalResourceDeliveryBrand` | `type_or_brand_case` | `high` |
| 3 | `WORTHSignalResourceDownloadBrand` | `WorthSignalResourceDownloadBrand` | `type_or_brand_case` | `high` |
| 3 | `WORTHSignalResourceUploadTransportPostureBrand` | `WorthSignalResourceUploadTransportPostureBrand` | `type_or_brand_case` | `high` |
| 3 | `WORTH_QUERY_SUBSCRIPTION_PHASE_SEVEN_COMPILE_FAIL_TARGET_COUNT` | `WORTH_QUERY_SUBSCRIPTION_PHASE_SEVEN_COMPILE_FAIL_TARGET_COUNT` | `const_or_env_keep` | `keep` |
| 3 | `WORTH_QUERY_SUBSCRIPTION_PHASE_SEVEN_GOLDEN_PATH_COUNT` | `WORTH_QUERY_SUBSCRIPTION_PHASE_SEVEN_GOLDEN_PATH_COUNT` | `const_or_env_keep` | `keep` |
| 3 | `WORTH_scope` | `worth_scope` | `snake_case_identifier_reviewed` | `high` |
| 3 | `WORTHd_admitted` | `forged_admitted` | `forged_word_damage` | `high` |
| 3 | `WORTHd_admitted_completion` | `forged_admitted_completion` | `forged_word_damage` | `high` |
| 3 | `WORTHd_denied` | `forged_denied` | `forged_word_damage` | `high` |
| 3 | `WORTHd_denied_completion` | `forged_denied_completion` | `forged_word_damage` | `high` |
| 3 | `WORTHd_equivalence` | `forged_equivalence` | `forged_word_damage` | `high` |
| 3 | `WORTHd_grace_window` | `forged_grace_window` | `forged_word_damage` | `high` |
| 3 | `WORTHd_host_advisory` | `forged_host_advisory` | `forged_word_damage` | `high` |
| 3 | `WORTHd_intent_digest` | `forged_intent_digest` | `forged_word_damage` | `high` |
| 3 | `WORTHd_mutation_program_for_digest_test` | `forged_mutation_program_for_digest_test` | `forged_word_damage` | `high` |
| 3 | `WORTHd_output_artifact_for_digest_test` | `forged_output_artifact_for_digest_test` | `forged_word_damage` | `high` |
| 3 | `WORTHd_parent` | `forged_parent` | `forged_word_damage` | `high` |
| 3 | `WORTHd_policy_digest` | `forged_policy_digest` | `forged_word_damage` | `high` |
| 3 | `WORTHd_propagation` | `forged_propagation` | `forged_word_damage` | `high` |
| 3 | `WORTHd_rejected` | `forged_rejected` | `forged_word_damage` | `high` |
| 3 | `WORTHd_request` | `forged_request` | `forged_word_damage` | `high` |
| 3 | `WORTHd_revalidation` | `forged_revalidation` | `forged_word_damage` | `high` |
| 3 | `WORTHd_row_with_basis` | `forged_row_with_basis` | `forged_word_damage` | `high` |
| 3 | `WORTHd_staged_effect` | `forged_staged_effect` | `forged_word_damage` | `high` |
| 3 | `WORTHd_timeout` | `forged_timeout` | `forged_word_damage` | `high` |
| 3 | `WORTHry` | `forgery` | `forgery_word_damage` | `high` |
| 3 | `WORTHtting` | `forgetting` | `forget_word_damage` | `high` |
| 3 | `__WORTHSignal.outputProjection.callbackLabel.1` | `__WorthSignal.outputProjection.callbackLabel.1` | `type_or_brand_case` | `high` |
| 3 | `__WORTHSignal.outputProjection.panelExplicit.2` | `__WorthSignal.outputProjection.panelExplicit.2` | `type_or_brand_case` | `high` |
| 2 | `BasisLifecycleProofShapeViolation::OperationLaneWORTHry` | `BasisLifecycleProofShapeViolation::OperationLaneForgery` | `forgery_word_damage` | `high` |
| 2 | `OperationLaneWORTHry` | `OperationLaneForgery` | `forgery_word_damage` | `high` |
| 2 | `PolicyNarrowingSupportStatus::BlockedOnWORTHStore` | `PolicyNarrowingSupportStatus::BlockedOnWorthStore` | `enum_variant_case` | `high` |
| 2 | `Self::BlockedOnWORTHStore` | `Self::BlockedOnWorthStore` | `enum_variant_case` | `high` |
| 2 | `WORTH-router:$` | `worth-router:$` | `protocol_slug_case_reviewed` | `high` |
| 2 | `WORTH-router:url:` | `worth-router:url:` | `protocol_slug_case_reviewed` | `high` |
| 2 | `WORTH.domain-capability.fixture` | `worth.domain-capability.fixture` | `canonical_namespace_case_reviewed` | `high` |
| 2 | `WORTH.query.writeback` | `worth.query.writeback` | `canonical_namespace_case_reviewed` | `high` |
| 2 | `WORTH.relational.harness.aspect-snapshot.v1` | `worth.relational.harness.aspect-snapshot.v1` | `canonical_namespace_case_reviewed` | `high` |
| 2 | `WORTH.relational.invariant.diagnostic_mask.v1` | `worth.relational.invariant.diagnostic_mask.v1` | `canonical_namespace_case_reviewed` | `high` |
| 2 | `WORTH.relational.merge.correspondence_witness.v1` | `worth.relational.merge.correspondence_witness.v1` | `canonical_namespace_case_reviewed` | `high` |
| 2 | `WORTH.relational.merge.schema_reconciliation_witness.v1` | `worth.relational.merge.schema_reconciliation_witness.v1` | `canonical_namespace_case_reviewed` | `high` |
| 2 | `WORTH.relational.merge.strategy_witness.v1` | `worth.relational.merge.strategy_witness.v1` | `canonical_namespace_case_reviewed` | `high` |
| 2 | `WORTH.runtime.bridge.causal-envelope-identity.v1:` | `worth.runtime.bridge.causal-envelope-identity.v1:` | `canonical_namespace_case_reviewed` | `high` |
| 2 | `WORTHFormValueType` | `WorthFormValueType` | `type_case` | `high` |
| 2 | `WORTHProofReadmissionDigest:` | `WorthProofReadmissionDigest:` | `type_case` | `high` |
| 2 | `WORTHSignal.hostCapabilityPlan` | `WorthSignal.hostCapabilityPlan` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.hostClockHandle` | `WorthSignal.hostClockHandle` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.hostClockRegistration` | `WorthSignal.hostClockRegistration` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.hostOnlineHandle` | `WorthSignal.hostOnlineHandle` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.hostOnlineRegistration` | `WorthSignal.hostOnlineRegistration` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.hostPersistenceHandle` | `WorthSignal.hostPersistenceHandle` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.hostPersistenceRegistration` | `WorthSignal.hostPersistenceRegistration` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.hostViewportHandle` | `WorthSignal.hostViewportHandle` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.hostViewportRegistration` | `WorthSignal.hostViewportRegistration` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.hostVisibilityHandle` | `WorthSignal.hostVisibilityHandle` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.hostVisibilityRegistration` | `WorthSignal.hostVisibilityRegistration` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignal.rawSignals` | `WorthSignal.rawSignals` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignalJsError::from_compute_callback_failure` | `WorthSignalJsError::from_compute_callback_failure` | `rust_type_case` | `high` |
| 2 | `WORTHSignalResourceProcessingResultBrand` | `WorthSignalResourceProcessingResultBrand` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignalResourceUploadResultBrand` | `WorthSignalResourceUploadResultBrand` | `type_or_brand_case` | `high` |
| 2 | `WORTHSignalRouteOutcomeBrand` | `WorthSignalRouteOutcomeBrand` | `type_or_brand_case` | `high` |
| 2 | `WORTH_GRAPH_AUTHORITY_QUERY_CONTRACT` | `WORTH_GRAPH_AUTHORITY_QUERY_CONTRACT` | `const_or_env_keep` | `keep` |
| 2 | `WORTH_SIGNAL_UPDATE_PERF_BASELINE` | `WORTH_SIGNAL_UPDATE_PERF_BASELINE` | `screaming_const_keep` | `keep` |
| 2 | `WORTHd-profile` | `forged-profile` | `forged_word_damage` | `high` |
| 2 | `WORTHd-summary-digest` | `forged-summary-digest` | `forged_word_damage` | `high` |
| 2 | `WORTHd-target-basis` | `forged-target-basis` | `forged_word_damage` | `high` |
| 2 | `WORTHd-witness-digest` | `forged-witness-digest` | `forged_word_damage` | `high` |
| 2 | `WORTHd.semantic_name` | `forged.semantic_name` | `forged_word_damage` | `high` |
| 2 | `WORTHd_anchor` | `forged_anchor` | `forged_word_damage` | `high` |
| 2 | `WORTHd_closeout:` | `forged_closeout:` | `forged_word_damage` | `high` |
| 2 | `WORTHd_digest_result.is_err` | `forged_digest_result.is_err` | `forged_word_damage` | `high` |
| 2 | `WORTHd_digest_result:` | `forged_digest_result:` | `forged_word_damage` | `high` |
| 2 | `WORTHd_equivalence:` | `forged_equivalence:` | `forged_word_damage` | `high` |
| 2 | `WORTHd_err` | `forged_err` | `forged_word_damage` | `high` |
| 2 | `WORTHd_matrix:` | `forged_matrix:` | `forged_word_damage` | `high` |
| 2 | `WORTHd_posture` | `forged_posture` | `forged_word_damage` | `high` |
| 2 | `WORTHd_row_with_recomputed_witness` | `forged_row_with_recomputed_witness` | `forged_word_damage` | `high` |
| 2 | `WORTHd_surface` | `forged_surface` | `forged_word_damage` | `high` |
| 2 | `WORTHd_surface_payload` | `forged_surface_payload` | `forged_word_damage` | `high` |
| 2 | `WORTHd_truth` | `forged_truth` | `forged_word_damage` | `high` |
| 2 | `WORTHd_witness` | `forged_witness` | `forged_word_damage` | `high` |
| 2 | `WORTHd_witness_result.is_err` | `forged_witness_result.is_err` | `forged_word_damage` | `high` |
| 2 | `WORTHd_witness_result:` | `forged_witness_result:` | `forged_word_damage` | `high` |
| 2 | `WorthQueryDeclarationEntryLowerOwnerCrate::WORTHRelational` | `WorthQueryDeclarationEntryLowerOwnerCrate::WorthRelational` | `enum_variant_case` | `high` |
| 2 | `WorthQueryDeclarationEntryLowerOwnerCrate::WORTHRuntimeBridge` | `WorthQueryDeclarationEntryLowerOwnerCrate::WorthRuntimeBridge` | `enum_variant_case` | `high` |
| 2 | `__WORTHInvalidRoute__:` | `__WorthInvalidRoute__:` | `typescript_error_sentinel_case_reviewed` | `high` |
| 2 | `__WORTHSignal.$` | `__WorthSignal.$` | `type_or_brand_case` | `high` |
| 2 | `__WORTHSignal.host.$` | `__WorthSignal.host.$` | `type_or_brand_case` | `high` |
| 2 | `__WORTHSignal.host.visibility.` | `__WorthSignal.host.visibility.` | `type_or_brand_case` | `high` |
| 2 | `__WORTHSignal.input.1` | `__WorthSignal.input.1` | `type_or_brand_case` | `high` |
| 2 | `__WORTHSignalActiveRuntimeCallbackReader` | `__WorthSignalActiveRuntimeCallbackReader` | `type_or_brand_case` | `high` |
| 2 | `__WORTHSignalActiveRuntimeCallbackReads` | `__WorthSignalActiveRuntimeCallbackReads` | `type_or_brand_case` | `high` |
| 2 | `__WORTHSignalScoped.$` | `__WorthSignalScoped.$` | `type_or_brand_case` | `high` |
| 2 | `application/vnd.WORTH.relational.harness.aspect-snapshot.v1` | `application/vnd.worth.relational.harness.aspect-snapshot.v1` | `media_type_namespace_case_reviewed` | `high` |
| 2 | `capture.__WORTHSignalCallbackCapture` | `capture.__WorthSignalCallbackCapture` | `type_or_brand_case` | `high` |
| 2 | `createWORTHdAuthorityArtifact` | `createForgedAuthorityArtifact` | `camel_identifier_forged_case_reviewed` | `high` |
| 2 | `fire-and-WORTHt` | `fire-and-forget` | `forget_word_damage` | `high` |
| 2 | `resourceEffectAuthorityGlobal.__WORTHResourceEffectAuthorityRegistry` | `resourceEffectAuthorityGlobal.__WorthResourceEffectAuthorityRegistry` | `internal_js_sentinel_case_reviewed` | `high` |
| 2 | `resourceModuleGlobal.__WORTHCachedResourceModuleCleanupInstalled` | `resourceModuleGlobal.__WorthCachedResourceModuleCleanupInstalled` | `internal_js_cache_sentinel_case_reviewed` | `high` |
| 2 | `resourceModuleGlobal.__WORTHCachedResourceModuleLoad` | `resourceModuleGlobal.__WorthCachedResourceModuleLoad` | `internal_js_cache_sentinel_case_reviewed` | `high` |
| 2 | `resourceModuleGlobal.__WORTHCachedResourceModuleTempDirs` | `resourceModuleGlobal.__WorthCachedResourceModuleTempDirs` | `internal_js_cache_sentinel_case_reviewed` | `high` |
| 2 | `signalsModuleGlobal.__WORTHCachedSignalsModuleCleanupInstalled` | `signalsModuleGlobal.__WorthCachedSignalsModuleCleanupInstalled` | `internal_js_cache_sentinel_case_reviewed` | `high` |
| 2 | `signalsModuleGlobal.__WORTHCachedSignalsModuleLoads` | `signalsModuleGlobal.__WorthCachedSignalsModuleLoads` | `internal_js_cache_sentinel_case_reviewed` | `high` |
| 2 | `signalsModuleGlobal.__WORTHCachedSignalsModuleTempDirs` | `signalsModuleGlobal.__WorthCachedSignalsModuleTempDirs` | `internal_js_cache_sentinel_case_reviewed` | `high` |
| 2 | `workerRuntimeAfterWORTHProofReadmission` | `workerRuntimeAfterWorthProofReadmission` | `type_or_camel_identifier_case_reviewed` | `high` |
| 1 | `.WORTH-checkpoint` | `.worth-checkpoint` | `protocol_slug_case_reviewed` | `high` |
| 1 | `.WORTH-segment` | `.worth-segment` | `protocol_slug_case_reviewed` | `high` |
| 1 | `/WORTH-worker-bridge:signal-readback-packet:/` | `/worth-worker-bridge:signal-readback-packet:/` | `protocol_slug_case_reviewed` | `high` |
| 1 | `PolicyExecutionSeamSupportStatus::BlockedOnWORTHStore` | `PolicyExecutionSeamSupportStatus::BlockedOnWorthStore` | `enum_variant_case` | `high` |
| 1 | `Self::OperationLaneWORTHry` | `Self::OperationLaneForgery` | `forgery_word_damage` | `high` |
| 1 | `Self::WORTHRelational` | `Self::WorthRelational` | `enum_variant_case` | `high` |
| 1 | `Self::WORTHRuntimeBridge` | `Self::WorthRuntimeBridge` | `enum_variant_case` | `high` |
| 1 | `Self::WORTHSignal` | `Self::WorthSignal` | `type_or_brand_case` | `high` |
| 1 | `WORTH-feature` | `worth-feature` | `protocol_slug_case_reviewed` | `high` |
| 1 | `WORTH-native` | `worth-native` | `protocol_slug_case_reviewed` | `high` |
| 1 | `WORTH-resource-external-v0` | `worth-resource-external-v0` | `protocol_slug_case_reviewed` | `high` |
| 1 | `WORTH-router:projected-outlet-contract:` | `worth-router:projected-outlet-contract:` | `protocol_slug_case_reviewed` | `high` |
| 1 | `WORTH-router:projected-outlet-route-occupant:` | `worth-router:projected-outlet-route-occupant:` | `protocol_slug_case_reviewed` | `high` |
| 1 | `WORTH-router:projected-outlet-stack:` | `worth-router:projected-outlet-stack:` | `protocol_slug_case_reviewed` | `high` |
| 1 | `WORTH-router:projected-route-composition:` | `worth-router:projected-route-composition:` | `protocol_slug_case_reviewed` | `high` |
| 1 | `WORTH-segment` | `worth-segment` | `protocol_slug_case_reviewed` | `high` |

## Remaining Medium/Review Rows

None. Every formerly ambiguous row now has an explicit high-confidence or keep decision.
