//! Recorded reachability cannot substitute for the installed ownership declaration.
use super::*;
use worth_relational::facade::mvcc::RelationalOwnedGraphSelectionPreparationError as Error;

#[test]
fn graph_write_refuses_an_uninstalled_ownership_recipe() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let native = runtime();
    let model = create_entity(execution, &native, "model");
    refuse(execution, &native, model, &ownership(), &ownership());
}

#[test]
fn graph_write_refuses_turning_an_ordinary_reference_into_document_ownership() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let native = installed_runtime();
    let model = create_entity(execution, &native, "model");
    let foreign_model = create_entity(execution, &native, "foreign-model");
    let foreign_member = member(execution, &native, "foreign-member");
    create_relation(
        execution,
        &native,
        foreign_model,
        foreign_member,
        "owned-by-foreign",
    );
    create_relation_of_kind(
        execution,
        &native,
        model,
        foreign_member,
        "ordinary-reference",
        KindId(4),
    );
    let broadened = RelationalModelOwnership::new(vec![RelationalModelOwnershipDomain {
        root: KindId(1),
        relations: vec![KindId(2), KindId(4)],
        entities: vec![KindId(3)],
    }]);
    refuse(execution, &native, model, &broadened, &ownership());
    assert_eq!(
        native
            .observe_model_owner(
                execution,
                &native
                    .admit_branch_basis(&native.main_branch_identity())
                    .unwrap(),
                &ownership(),
                foreign_member
            )
            .unwrap(),
        Some(foreign_model)
    );
}

#[test]
fn graph_write_refuses_an_incomplete_global_ownership_recipe() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let native = installed_runtime();
    let model = create_entity(execution, &native, "model");
    let member = member(execution, &native, "member");
    create_relation(execution, &native, model, member, "owns");
    refuse(
        execution,
        &native,
        model,
        &ownership(),
        &RelationalModelOwnership::new(vec![]),
    );
}

fn refuse(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    model: EntityId,
    document: &RelationalModelOwnership,
    global: &RelationalModelOwnership,
) {
    let (_, basis) = native
        .observe_branch(&native.main_branch_identity())
        .unwrap();
    let graph = native
        .prepare_owned_model_graph_selection(execution, &basis, &basis, document, global, model)
        .unwrap();
    let error = native
        .preparation_port()
        .admit_owned_graph_selection(execution, begin(native), graph)
        .unwrap_err();
    assert!(
        matches!(error, Error::OwnershipNotInstalled { model: subject } if subject == model),
        "{error:?}"
    );
    assert_eq!(
        native
            .observe_branch(&native.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}
