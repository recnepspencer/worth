//! Owner-derived custody obligations and the owner gates that enforce them.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::declaration::application_program::ApplicationSemanticChangeKind;
use worth_query_host::facade::domain::WorthQueryProgramCustodyInventoryKind;
use worth_query_host::facade::primary_graph::WorthQueryProgramCustodyDispositionKind;

use crate::bounded_dimension_model::host::{
    publish_on_first_program, publish_on_first_resource_program,
};
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::presented_request::set_dimension;
use crate::bounded_dimension_model::programs::{
    ChangedOperationDimensionProgram, RemovedOperationDimensionProgram, ResourceDimensionProgramP1,
};
use crate::bounded_dimension_model::settled_verdict::{settle, DimensionVerdict};

#[test]
fn removed_operation_derives_retire_disposition_and_closes_its_source_owner() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = host
        .supported_program::<RemovedOperationDimensionProgram>()
        .expect("the removal target is rostered")
        .owned_revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs
        .compare(&target)
        .expect("the owner must describe operation removal");
    let removed_operations = requirements
        .custody_inventory_requirements()
        .iter()
        .filter(|requirement| {
            requirement.kind() == WorthQueryProgramCustodyInventoryKind::OperationContinuation
        })
        .collect::<Vec<_>>();
    assert_eq!(removed_operations.len(), 2);
    assert!(removed_operations
        .iter()
        .all(|requirement| requirement.change() == ApplicationSemanticChangeKind::Removed));

    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("execution derives the disposition from installed owner truth");
    let expected_custody = prepared.custody().clone();
    let dispositions = expected_custody.dispositions();
    assert_eq!(dispositions.len(), removed_operations.len());
    for requirement in removed_operations {
        let disposition = dispositions
            .iter()
            .find(|disposition| disposition.requirement() == requirement)
            .expect("every removed operation needs a custody disposition");
        assert_eq!(
            disposition.kind(),
            WorthQueryProgramCustodyDispositionKind::RetireUneffectedContinuation
        );
    }

    let performed = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::Performed(performed) => performed,
        WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
            panic!("operation-removal adoption unexpectedly had no effect: {no_effect:?}")
        }
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            panic!("operation-removal adoption lost publication: {unpublished:?}")
        }
    };
    assert_eq!(performed.custody(), &expected_custody);
    assert_eq!(
        settle(set_dimension(&host, host.current_world(), 8, 0x9175_3c00)),
        DimensionVerdict::inactive(),
        "the removed operation cannot continue through its obsolete source-program owner"
    );
}

#[test]
fn changed_operation_requires_and_then_receives_fresh_target_admission() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target_owner = host
        .supported_program::<ChangedOperationDimensionProgram>()
        .expect("the changed-operation target is rostered");
    let target = target_owner.owned_revision().clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("comparison succeeds");
    let requirement = requirements
        .custody_inventory_requirements()
        .iter()
        .find(|requirement| {
            requirement.kind() == WorthQueryProgramCustodyInventoryKind::OperationContinuation
        })
        .expect("changed operation requires continuation disposition");
    assert_eq!(requirement.change(), ApplicationSemanticChangeKind::Changed);

    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("changed operation has an owner-derived disposition");
    assert!(prepared.custody().dispositions().iter().any(|disposition| {
        disposition.requirement() == requirement
            && disposition.kind() == WorthQueryProgramCustodyDispositionKind::FreshCurrentAdmission
    }));
    assert!(matches!(
        prepared.publish(),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));

    let adopted = host.current_world();
    assert_eq!(
        settle(set_dimension(&host, adopted, 9, 0x9175_3c01)),
        DimensionVerdict::inactive(),
        "the changed operation cannot continue through stale source admission"
    );
    assert_eq!(
        settle(set_dimension(&target_owner, adopted, 9, 0x9175_3c02)),
        DimensionVerdict::Performed(9),
        "the operation executes only through a fresh request admitted by the active target"
    );
}

#[test]
fn changed_resource_ceiling_derives_exact_source_reservation_disposition() {
    let host = publish_on_first_resource_program();
    let branch = host.current_world();
    let target = host
        .supported_program::<ResourceDimensionProgramP1>()
        .expect("the changed-resource target is rostered")
        .owned_revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("comparison succeeds");
    let resource = requirements
        .custody_inventory_requirements()
        .iter()
        .find(|requirement| {
            requirement.kind() == WorthQueryProgramCustodyInventoryKind::ResourceCustody
        })
        .expect("a changed resource ceiling requires resource custody");
    assert_eq!(resource.change(), ApplicationSemanticChangeKind::Changed);
    let resource_change = requirements
        .semantic_diff()
        .changes()
        .iter()
        .find(|change| change.subject() == resource.subject())
        .expect("the custody requirement retains its exact semantic change");
    assert_eq!(
        resource_change.source_meaning(),
        Some("work=1:8|bytes=4:4096")
    );
    assert_eq!(
        resource_change.target_meaning(),
        Some("work=2:16|bytes=4:8192")
    );

    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("the resource owner supplies an exact-source disposition");
    assert!(prepared.custody().dispositions().iter().any(|disposition| {
        disposition.requirement() == resource
            && disposition.kind()
                == WorthQueryProgramCustodyDispositionKind::RetainExactSourceReservation
    }));
    let performed = prepared.publish();
    assert!(matches!(
        performed,
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}
