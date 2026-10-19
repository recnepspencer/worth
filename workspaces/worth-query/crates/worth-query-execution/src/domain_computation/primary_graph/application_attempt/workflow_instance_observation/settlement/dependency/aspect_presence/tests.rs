//! A reconstructed presence obligation moves even when its field stays absent.
use super::*;
use crate::domain_computation::primary_graph::workflow::evidence_dependency::WorkflowEvidenceDependencyKind;
use crate::domain_computation::primary_graph::{
    application_attempt::observe_aspect_presence,
    tests::{application_attempt::resolved_account, fixture},
};
use worth_foundational::facade::{
    ContractValidationInput, PortableAspectContractBasis, PortableAspectPatchOperation,
    PortableRecordAspectPatch, StructAspectValue,
};
use worth_relational::facade::transactions::{
    ApplyEntityAspectPatchIntent, EntityMutationIntent, MutationIntent, WorkerIntentBatch,
};

#[test]
fn optional_aspect_initialization_reconstructed_presence_detects_present_empty() {
    let world = fixture::installed_authorization_world(true);
    let request = fixture::live_scope();
    world.application.with_host_advancement(|phase| {
        let host = phase.test_host_request();
        let execution = host.execution_request_for(world.application.product_runtime()).unwrap();
        let account = resolved_account(&host, &world, "unrelated", &request);
        let entity = account.entity_id();
        let kind = account.entity_kind();
        let graph = world.application.runtime.primary_graph().unwrap();
        let field = fixture::AccountAnnotation::reference();
        let locator = graph.layout.field_locator(field.entity(), field.aspect(), field.field()).unwrap();
        let aspect = locator.aspect().aspect_key().clone();
        let contract = graph.layout.aspect_contract(field.entity(), &aspect).unwrap();
        let before = world.selected_product(&host);
        let handle = graph.integration_handle();
        let present = handle.with_runtime(&execution, |runtime| observe_aspect_presence(
            runtime, &execution, before.application_basis().snapshot_handle(), entity, kind, &aspect).unwrap().unwrap());
        assert!(!present);
        let tag = if present { WorkflowEvidenceDependencyKind::AspectPresent } else { WorkflowEvidenceDependencyKind::AspectAbsent };
        assert_eq!(tag.encode(), "aspect-absent");
        let readonly = WorthQueryApplicationObservedFact::AbsentField {
            entity_id: entity, kind, locator: locator.clone(),
        };
        let decoded_tag = WorkflowEvidenceDependencyKind::decode(tag.encode()).unwrap();
        let reconstructed = decode(entity, Some(aspect.as_str().to_owned()),
            Some(u64::from(kind.as_u32())), matches!(decoded_tag, WorkflowEvidenceDependencyKind::AspectPresent)).unwrap();
        handle.with_runtime(&execution, |runtime| {
            assert!(reconstructed.remains_equal_in(runtime, &execution, before.application_basis().snapshot_handle()).unwrap());
        });
        fixture::publish_relational_mutation(&execution, &world,
            WorkerIntentBatch::new("materialize-empty-annotations").push(
                MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: entity,
                    aspect_patch: PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
                        basis: PortableAspectContractBasis::from_contract(contract),
                        value: ContractValidationInput::Struct(StructAspectValue::new([]).unwrap()),
                    }]),
                }))));
        let after = world.selected_product(&host);
        handle.with_runtime(&execution, |runtime| {
            let snapshot = after.application_basis().snapshot_handle();
            assert!(!reconstructed.remains_equal_in(runtime, &execution, snapshot).unwrap());
            assert!(readonly.remains_equal_in(runtime, &execution, snapshot).unwrap(),
                "a read-only field keeps its narrower dependency");
            assert!(matches!(
                crate::domain_computation::primary_graph::application_attempt::observation::observe_field(
                    runtime, &execution, snapshot, entity, kind, locator).unwrap(),
                Some(crate::domain_computation::primary_graph::application_attempt::observation::WorthQueryApplicationFieldObservation::Absent)
            ), "field absence alone cannot stand in for aspect absence");
        });
        assert!(decode(entity, None, Some(u64::from(kind.as_u32())), false).is_err());
        assert!(decode(entity, Some(aspect.as_str().to_owned()), None, false).is_err());
    }).unwrap();
}
