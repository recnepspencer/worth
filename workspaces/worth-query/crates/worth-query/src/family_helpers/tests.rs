use std::marker::PhantomData;

use crate::application::{
    WorthQueryBridgeContinuationAuthority, WorthQueryCapabilityFamily,
    WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationCanonicalEntry, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityContract,
    WorthQueryDeclarationRouteContract, WorthQueryDeclarationSignalCompatibilityContract,
    WorthQueryDeclarationSignalCompatibilityInput, WorthQueryDeclarationSignalExecutionFamily,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryNeighborhoodCapableGrouping, WorthQuerySignalCompatiblePosture,
};
use crate::domain_capabilities::{
    WorthQueryExplanationContributionAuthoring, WorthQuerySupportContributionAuthoring,
    WorthQueryWorkflowContributionAuthoring,
};
use crate::family_helpers::{
    WorthQueryGeometryActiveFaceSelectionHelperFamily,
    WorthQueryGeometryMaterialAttachmentHelperFamily, WorthQueryGeometryMaterialAttachmentInput,
};
use crate::ordinary_outcome::{
    WorthQueryOrdinaryContributionComposedCheckedTopologyKind, WorthQueryOrdinaryOutcome,
};
use crate::signal_compatibility_orchestration::{
    WorthQuerySignalCompatibilityOrchestration, WorthQuerySignalCompatibilityOrchestrationInput,
    WorthQuerySignalCompatibilityOrchestrationOutcome,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GeometryDomain;

impl WorthQueryDomainEntryMarker for GeometryDomain {
    fn domain_key(&self) -> &'static str {
        "test.family_helpers.geometry"
    }

    fn display_name(&self) -> &'static str {
        "GeometryFamilyHelpersDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GeometryWorld(&'static str);

impl WorthQueryDomainOperatingContext<GeometryDomain> for GeometryWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::PreviewSession]
    }

    fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        &[
            WorthQueryConfigSectionFamily::Query,
            WorthQueryConfigSectionFamily::RuntimeBridge,
            WorthQueryConfigSectionFamily::Signal,
        ]
    }

    fn context_identity(
        &self,
    ) -> crate::application::WorthQueryDomainOperatingContextIdentityDeclaration {
        let value = { format!("family-helpers-world-{}", self.0) };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GeometryFamily;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for GeometryFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "GeometryFamily"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(
            &["selection.active_face"],
            &["selection.material_preview"],
            &[],
            &[],
            &[],
        )
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::relational_and_bridge()
    }

    fn bridge_continuation_contract(
    ) -> Option<crate::application::WorthQueryDeclarationBridgeContinuationContract> {
        Some(crate::application::WorthQueryDeclarationBridgeContinuationContract::preview_session())
    }

    fn signal_compatibility_contract() -> Option<WorthQueryDeclarationSignalCompatibilityContract> {
        Some(WorthQueryDeclarationSignalCompatibilityContract::runtime_derived_execution())
    }
}

impl WorthQueryGeometryActiveFaceSelectionHelperFamily<GeometryDomain> for GeometryFamily {}

impl WorthQueryGeometryMaterialAttachmentHelperFamily<GeometryDomain> for GeometryFamily {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeometryInput {
    id: &'static str,
    _marker: PhantomData<GeometryFamily>,
}

impl GeometryInput {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<GeometryDomain> for GeometryInput {
    type Family = GeometryFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }
}

fn admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &'static str,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>
{
    crate::application::domain_test_support::installed_declaration_context_with_contributions(
        request_execution, GeometryDomain,
        GeometryWorld(world),
        [crate::application::domain_test_support::family::<
            GeometryDomain,
            GeometryFamily,
        >()],
        [
            crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability,
            crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::ExplanationInspection,
            crate::application::WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview,
        ],
    )
}

fn preview_request() -> crate::application::WorthQueryDeclarationBridgeContinuationRequest {
    crate::application::WorthQueryDeclarationBridgeContinuationRequest::new(
        crate::application::WorthQueryDeclarationBridgeContinuationMode::PreviewSession,
        crate::application::WorthQueryDeclarationBridgeTruthContext::Preview,
    )
}

fn runtime_route_request() -> crate::application::WorthQueryDeclarationBridgeContinuationRequest {
    crate::application::WorthQueryDeclarationBridgeContinuationRequest::new(
        crate::application::WorthQueryDeclarationBridgeContinuationMode::RuntimeRoute,
        crate::application::WorthQueryDeclarationBridgeTruthContext::Current,
    )
}

fn progressed(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        GeometryDomain,
        GeometryWorld,
    >,
    id: &'static str,
) -> crate::application::WorthQueryAdmittedDeclarationProgression<GeometryDomain, GeometryInput> {
    match handle.declare_review_and_progress(GeometryInput::new(id)) {
        Ok(value) => value,
        Err(_) => panic!("expected progressed geometry selection"),
    }
}

#[test]
fn preview_helper_matches_generic_signal_orchestration_path() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let helper_progressed = match handle
        .geometry_helpers()
        .progress_active_face_selection(GeometryInput::new("face-a"))
    {
        Ok(value) => value,
        Err(_) => panic!("expected helper progression to succeed"),
    };
    let generic_progressed = progressed(&handle, "face-a");

    let helper = handle
        .geometry_helpers()
        .prepare_preview_for_active_face_selection_checked(helper_progressed);
    let generic = handle.orchestrate_signal_compatibility_checked(
        generic_signal_orchestration_input(&handle, generic_progressed)
            .with_bridge_request(preview_request()),
    );

    assert_eq!(
        helper.orchestration_digest(),
        generic.orchestration_digest()
    );
    assert_eq!(helper.linked_artifacts(), generic.linked_artifacts());
}

#[path = "tests/truth_view_helpers_keep_current_and_historical_meaning_distinct.rs"]
mod truth_view_helpers_keep_current_and_historical_meaning_distinct;

#[test]
fn material_attachment_helper_matches_generic_composed_path() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let helper_input = WorthQueryGeometryMaterialAttachmentInput::new(GeometryInput::new("face-d"))
        .with_support_contribution(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "geometry.trace",
                "track selection to material attachment",
            ),
        )
        .with_explanation_contribution(
            WorthQueryExplanationContributionAuthoring::requires_context(
                "geometry.material.context",
                "material attachment needs explicit context from the active face selection",
            ),
        )
        .with_workflow_contribution(WorthQueryWorkflowContributionAuthoring::preview_only(
            "geometry.preview",
            "preview material attachment before promotion",
        ));
    let generic_input =
        crate::contribution_composed_orchestration::WorthQueryContributionComposedOrchestrationInput::new(
            GeometryInput::new("face-d"),
        )
        .with_contribution(crate::contribution_composed_orchestration::WorthQueryContributionIntent::support(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "geometry.trace",
                "track selection to material attachment",
            ),
        ))
        .with_contribution(crate::contribution_composed_orchestration::WorthQueryContributionIntent::explanation(
            WorthQueryExplanationContributionAuthoring::requires_context(
                "geometry.material.context",
                "material attachment needs explicit context from the active face selection",
            ),
        ))
        .with_contribution(crate::contribution_composed_orchestration::WorthQueryContributionIntent::workflow(
            WorthQueryWorkflowContributionAuthoring::preview_only(
                "geometry.preview",
                "preview material attachment before promotion",
            ),
        ));

    let helper = handle
        .geometry_helpers()
        .orchestrate_material_attachment_for_active_face_selection_proof(helper_input);
    let generic = handle.orchestrate_declaration_with_contributions_proof(generic_input);

    assert_eq!(helper.request_digest(), generic.request_digest());
    assert_eq!(helper.linked_artifacts(), generic.linked_artifacts());
    assert_eq!(helper.contribution_digest(), generic.contribution_digest());
}

#[test]
fn helper_wrong_world_matches_generic_path() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = admitted_handle(request_execution, "left");
    let right = admitted_handle(request_execution, "right");
    let helper = right
        .geometry_helpers()
        .prepare_runtime_route_for_active_face_selection_outcome(progressed(&left, "face-e"));
    let generic = right.orchestrate_signal_compatibility_outcome(
        generic_signal_orchestration_input(&right, progressed(&left, "face-e"))
            .with_bridge_request(runtime_route_request()),
    );

    match (helper, generic) {
        (
            WorthQueryOrdinaryOutcome::WrongWorld(left_posture),
            WorthQueryOrdinaryOutcome::WrongWorld(right_posture),
        ) => {
            assert_eq!(left_posture.reason(), right_posture.reason());
        }
        _ => panic!("expected helper and generic paths to preserve wrong-world posture"),
    }
}

#[test]
fn material_attachment_helper_preserves_contribution_checked_topology() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let outcome = handle
        .geometry_helpers()
        .orchestrate_material_attachment_for_active_face_selection_outcome(
            WorthQueryGeometryMaterialAttachmentInput::new(GeometryInput::new("face-f"))
                .with_support_contribution(
                    WorthQuerySupportContributionAuthoring::declaration_traceability(
                        "geometry.traceability",
                        "prove material attachment request lineage",
                    ),
                ),
        );

    match outcome {
        WorthQueryOrdinaryOutcome::Bound(_) => {}
        WorthQueryOrdinaryOutcome::Denied(posture) => {
            panic!(
                "expected bound material-attachment helper, got topology {:?}",
                posture.checked_topology().contribution_composed_kind()
            );
        }
        WorthQueryOrdinaryOutcome::Unsupported(posture) => {
            assert_ne!(
                posture.checked_topology().contribution_composed_kind(),
                Some(WorthQueryOrdinaryContributionComposedCheckedTopologyKind::ContributionDenied)
            );
        }
        _ => {}
    }
}

fn generic_signal_orchestration_input(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        GeometryDomain,
        GeometryWorld,
    >,
    progressed: crate::application::WorthQueryAdmittedDeclarationProgression<
        GeometryDomain,
        GeometryInput,
    >,
) -> WorthQuerySignalCompatibilityOrchestrationInput<GeometryDomain, GeometryInput> {
    let envelope_checked = handle.orchestrate_envelope_from_progressed_checked(progressed);
    WorthQuerySignalCompatibilityOrchestrationInput::new(
        WorthQueryDeclarationSignalCompatibilityInput::envelope_checked(envelope_checked),
    )
}
