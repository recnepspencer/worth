use super::super::super::support::*;
use super::super::consumer_support::routing::ConsumerStopRoute;

pub(super) fn consumer_stop_route_key(route: &ConsumerStopRoute) -> String {
    match route {
        ConsumerStopRoute::Advancement(denial) => format!("advancement:{denial:?}"),
        ConsumerStopRoute::ForeignPhase(_) => "foreign_phase".to_owned(),
        ConsumerStopRoute::InstalledDomainAuthorityDenied(kind) => {
            format!("installed_domain_authority_denied:{kind:?}")
        }
        ConsumerStopRoute::MissingRuntimeComponent(component) => {
            format!(
                "missing_runtime_component:{}",
                missing_component_key(*component)
            )
        }
        ConsumerStopRoute::ExistingTruthAssertionDenied(kind) => {
            format!("existing_truth_assertion_denied:{kind:?}")
        }
        ConsumerStopRoute::ExistingTruthProbeDenied(kind) => {
            format!("existing_truth_probe_denied:{kind:?}")
        }
        ConsumerStopRoute::MutationBindingDenied(kind) => {
            format!("mutation_binding_denied:{kind:?}")
        }
        ConsumerStopRoute::MutationContinuityDenied(kind) => {
            format!("mutation_continuity_denied:{kind:?}")
        }
        ConsumerStopRoute::MutationContractDenied(kind) => {
            format!("mutation_contract_denied:{kind:?}")
        }
        ConsumerStopRoute::GraphCompositionDenied(kind) => {
            format!("graph_composition_denied:{kind:?}")
        }
        ConsumerStopRoute::MutationNamingDenied(kind) => {
            format!("mutation_naming_denied:{kind:?}")
        }
        ConsumerStopRoute::MutationTargetReferenceDenied(kind) => {
            format!("mutation_target_reference_denied:{kind:?}")
        }
        ConsumerStopRoute::ReadCompositionDenied(kind) => {
            format!("read_composition_denied:{kind:?}")
        }
        ConsumerStopRoute::WorkspaceDenied => "workspace_denied".to_string(),
        ConsumerStopRoute::ProgramDenied => "program_denied".to_string(),
        ConsumerStopRoute::RuntimeLookupDenied(kind) => {
            format!("runtime_lookup_denied:{kind:?}")
        }
        ConsumerStopRoute::MissingRuntimeArtifact(kind) => {
            format!("missing_runtime_artifact:{kind:?}")
        }
        ConsumerStopRoute::RuntimeDeclarationDenied(kind) => {
            format!("runtime_declaration_denied:{kind:?}")
        }
        ConsumerStopRoute::PreviewOperationEffectDenied(label_identity) => {
            format!(
                "preview_operation_effect_denied:{}",
                label_identity.as_str()
            )
        }
        ConsumerStopRoute::UnsupportedAuthorityRequirement(requirement) => {
            format!("unsupported_authority_requirement:{}", requirement.as_str())
        }
        ConsumerStopRoute::ExistingTruthAssertionRequiresAuthorityLane(required_lane) => {
            format!(
                "existing_truth_assertion_requires_authority_lane:{}",
                required_lane.as_str()
            )
        }
        ConsumerStopRoute::IntentCommitDenied => "intent_commit_denied".to_string(),
        ConsumerStopRoute::IntentExecutionRoutingFailed(kind) => {
            format!("intent_execution_routing_failed:{kind:?}")
        }
        ConsumerStopRoute::EffectPolicyDenied => "effect_policy_denied".to_string(),
        ConsumerStopRoute::SharedReadStaleBasis => "shared_read_stale_basis".to_string(),
        ConsumerStopRoute::JournalReplayDenied(kind) => {
            format!("journal_replay_denied:{}", kind.as_str())
        }
        ConsumerStopRoute::PreviewPromotionDenied(kind) => {
            format!("preview_promotion_denied:{kind:?}")
        }
        ConsumerStopRoute::FamilyAdmissionDenied {
            family,
            status,
            teaching_posture,
        } => format!(
            "family_admission_denied:{}:{}:{}",
            family.as_str(),
            status.as_str(),
            teaching_posture
                .map(WorthQueryRuntimeFamilyTeachingPosture::as_str)
                .unwrap_or("none")
        ),
        ConsumerStopRoute::SessionLabelCollision(authority_lane) => {
            format!("session_label_collision:{}", authority_lane.as_str())
        }
    }
}

fn missing_component_key(component: WorthQueryRuntimeMissingComponent) -> &'static str {
    match component {
        WorthQueryRuntimeMissingComponent::Backend => "backend",
        WorthQueryRuntimeMissingComponent::RuntimeBridge => "runtime_bridge",
        WorthQueryRuntimeMissingComponent::SchemaAdapter => "schema_adapter",
        WorthQueryRuntimeMissingComponent::SnapshotIdentityAdapter => "snapshot_identity_adapter",
        WorthQueryRuntimeMissingComponent::SourceAdapter => "source_adapter",
        WorthQueryRuntimeMissingComponent::WriteAuthority => "write_authority",
        WorthQueryRuntimeMissingComponent::SignalSink => "signal_sink",
        WorthQueryRuntimeMissingComponent::SubscriptionActivation => "subscription_activation",
        WorthQueryRuntimeMissingComponent::PreviewBasis => "preview_basis",
        WorthQueryRuntimeMissingComponent::InspectorEvidence => "inspector_evidence",
        WorthQueryRuntimeMissingComponent::IntentAuthority => "intent_authority",
    }
}
