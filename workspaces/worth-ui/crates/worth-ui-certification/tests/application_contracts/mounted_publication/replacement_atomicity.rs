use worth_ui::facade::app::{
    WorthUiMountedApplicationReplacementOutcome, WorthUiMountedReplacementPreparationOutcome,
    WorthUiPreparedApplicationGenerationIdentity, WorthUiPreparedMountedApplicationReplacement,
};
use worth_ui::facade::graph::UiGraphGeneration;
use worth_ui_runtime::facade::application::WorthUiOrdinaryPlanAvailability;
use worth_ui_runtime::facade::mounted::{
    UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationMode, UiMountedFrameIdentity,
    UiMountedFramePublicationReceipt, UiMountedFrameRequest, UiMountedFrameRetentionBudget,
    UiMountedFrameRetentionBudgetInput, UiMountedRetentionClassBudget, UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiMountedPublicationCertificationExt;
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiMountedIdentityCertificationExt,
};

use super::{published, replacement_workspace, stage_replacement};
use crate::mounted_application_lifecycle::in_flight_presentation_world::{
    mounted_session, prepared,
};
use crate::mounted_application_lifecycle::known_empty_surface_world::{
    first_node, mounted_application_with_host_and_retention_budget, profile,
};
use crate::mounted_host_protocol::scripted_host::{
    presented_completion, ScriptedPresentationHost, ScriptedSurfaceCompletion,
};

struct PredecessorTruth {
    application_generation: WorthUiPreparedApplicationGenerationIdentity,
    graph_generation: UiGraphGeneration,
    ordinary_plan: WorthUiOrdinaryPlanAvailability,
    publication: Option<UiMountedFramePublicationReceipt>,
    mounted_frame: Option<UiMountedFrameIdentity>,
}

impl PredecessorTruth {
    fn capture(session: &worth_ui::facade::app::WorthUiActiveApplicationSession) -> Self {
        Self {
            application_generation: session.generation_identity().clone(),
            graph_generation: session.graph().generation(),
            ordinary_plan: session.ordinary_plan_availability(),
            publication: session.current_mounted_publication().cloned(),
            mounted_frame: session.inspect_mounted_identity().current_frame(),
        }
    }

    fn assert_unchanged(&self, session: &worth_ui::facade::app::WorthUiActiveApplicationSession) {
        assert_eq!(
            session.generation_identity(),
            &self.application_generation,
            "replacement failure cannot publish candidate application authority"
        );
        assert_eq!(
            session.graph().generation(),
            self.graph_generation,
            "replacement failure cannot publish candidate graph authority"
        );
        assert_eq!(
            session.ordinary_plan_availability(),
            self.ordinary_plan,
            "replacement failure cannot exchange the active plan"
        );
        assert_eq!(
            session.current_mounted_publication(),
            self.publication.as_ref(),
            "replacement failure cannot replace the mounted publication"
        );
        assert_eq!(
            session.inspect_mounted_identity().current_frame(),
            self.mounted_frame,
            "replacement failure cannot commit candidate mounted identity"
        );
    }
}

#[test]
fn admission_and_adapter_rejection_preserve_the_complete_predecessor_tuple() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "replacement-denial-atomicity", 1);
    host.push_presented();
    let predecessor_frame = prepared(&mut session);
    let predecessor = published(session.present_prepared_mounted_frame(
        predecessor_frame,
        UiPresentationDeadline::at_tick(10),
        0,
    ));
    let truth = PredecessorTruth::capture(&session);
    let workspace = replacement_workspace("replacement-denial-atomicity");

    let replacement = prepared_replacement(&workspace, &mut session);
    let denial = match replacement.present(UiPresentationDeadline::at_tick(0), 1) {
        WorthUiMountedApplicationReplacementOutcome::AdmissionDenied(denial) => denial,
        _ => panic!("expired replacement presentation must deny admission"),
    };
    drop(denial);
    truth.assert_unchanged(&session);

    let replacement = prepared_replacement(&workspace, &mut session);
    host.push_rejected();
    let rejection = match replacement.present(UiPresentationDeadline::at_tick(20), 2) {
        WorthUiMountedApplicationReplacementOutcome::RejectedBeforeEffects(rejection) => rejection,
        _ => panic!("adapter rejection must return the exact prepared replacement"),
    };
    assert_eq!(rejection.rejections().len(), 1);
    let retry = rejection.into_replacement();
    drop(retry);
    truth.assert_unchanged(&session);
    assert_eq!(session.current_mounted_publication(), Some(&predecessor));
    workspace.close();
}

#[test]
fn pending_replacement_stays_typed_until_terminal_atomic_publication() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "replacement-completion-retry", 1);
    host.push_presented();
    let predecessor_frame = prepared(&mut session);
    let predecessor = published(session.present_prepared_mounted_frame(
        predecessor_frame,
        UiPresentationDeadline::at_tick(10),
        0,
    ));
    let predecessor_generation = session.generation_identity().clone();
    let workspace = replacement_workspace("replacement-completion-retry");
    let replacement = prepared_replacement(&workspace, &mut session);
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending, presented_completion()],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );

    let in_flight = match replacement.present(UiPresentationDeadline::at_tick(20), 1) {
        WorthUiMountedApplicationReplacementOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("scripted replacement must remain in flight"),
    };
    let in_flight = match in_flight.complete(2) {
        WorthUiMountedApplicationReplacementOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("pending host completion must preserve the typed replacement transaction"),
    };
    let (application, mounted) = match in_flight.complete(3) {
        WorthUiMountedApplicationReplacementOutcome::Published {
            application,
            mounted,
        } => (application, mounted),
        _ => panic!("the returned in-flight replacement must publish on exact completion"),
    };

    assert_eq!(application.prior_generation(), &predecessor_generation);
    assert_eq!(
        application.active_generation(),
        session.generation_identity()
    );
    assert_eq!(mounted.predecessor(), Some(predecessor.frame()));
    assert_eq!(session.current_mounted_publication(), Some(&mounted));
    workspace.close();
}

#[test]
fn indeterminate_replacement_preserves_predecessor_application_and_publication() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "replacement-indeterminate", 1);
    host.push_presented();
    let predecessor_frame = prepared(&mut session);
    let predecessor = published(session.present_prepared_mounted_frame(
        predecessor_frame,
        UiPresentationDeadline::at_tick(10),
        0,
    ));
    let truth = PredecessorTruth::capture(&session);
    let workspace = replacement_workspace("replacement-indeterminate");
    let replacement = prepared_replacement(&workspace, &mut session);
    host.push_presentation(
        worth_ui_runtime::facade::mounted::UiHostSurfacePresentationOutcome::PresentationIndeterminate,
    );

    let frame = match replacement.present(UiPresentationDeadline::at_tick(20), 1) {
        WorthUiMountedApplicationReplacementOutcome::PresentationIndeterminate(frame) => frame,
        _ => panic!("indeterminate host effects must not publish either successor"),
    };
    drop(frame);
    truth.assert_unchanged(&session);
    assert_eq!(session.current_mounted_publication(), Some(&predecessor));
    workspace.close();
}

#[test]
fn retention_denial_precedes_host_effects_and_preserves_active_truth() {
    let host = ScriptedPresentationHost::default();
    let one_byte = UiMountedRetentionClassBudget::new(1, 1);
    let budget = UiMountedFrameRetentionBudget::new(UiMountedFrameRetentionBudgetInput {
        current: one_byte,
        in_flight: UiMountedRetentionClassBudget::new(8, 128 * 1024 * 1024),
        observation_basis: UiMountedRetentionClassBudget::new(8, 128 * 1024 * 1024),
        predecessor_inspection: UiMountedRetentionClassBudget::new(8, 128 * 1024 * 1024),
        diagnostic: UiMountedRetentionClassBudget::new(0, 0),
        visual_snapshot: UiMountedRetentionClassBudget::new(0, 0),
        visual_overlay: UiMountedRetentionClassBudget::new(0, 0),
        expired_identity_limit: 64,
    });
    let mut session = mounted_application_with_host_and_retention_budget(
        "replacement-retention-atomicity",
        host.clone(),
        budget,
    )
    .launch()
    .expect("the real filesystem-authored application launches");
    let node = first_node(&session);
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    session.mount_instance(node, surface).unwrap();
    let truth = PredecessorTruth::capture(&session);
    let workspace = replacement_workspace("replacement-retention-atomicity");
    let replacement = prepared_replacement(&workspace, &mut session);

    let denial = match replacement.present(UiPresentationDeadline::at_tick(20), 1) {
        WorthUiMountedApplicationReplacementOutcome::RetentionDenied(denial) => denial,
        _ => panic!("one-byte current retention budget must deny replacement"),
    };
    drop(denial);
    truth.assert_unchanged(&session);
    assert_eq!(
        host.presentation_calls(),
        0,
        "retention denial must occur before adapter effects"
    );
    workspace.close();
}

fn prepared_replacement<'session>(
    workspace: &crate::filesystem_contract_workspace::FilesystemContractWorkspace,
    session: &'session mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> Box<WorthUiPreparedMountedApplicationReplacement<'session>> {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    let (pending, catalog, boundary) = stage_replacement(workspace, session);
    match session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .unwrap()
    {
        WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("changed filesystem meaning requires mounted replacement")
        }
    }
}
