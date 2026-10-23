//! Plain and typed requests use the same real installed program records.
use super::*;
use worth_query_host::facade::{
    application_installation::{WorthQueryProgramOwner, WorthQuerySelectedProgramOwnerDenial},
    primary_graph::{
        WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchAdmissionRefusalKind,
    },
};

#[test]
fn plain_and_typed_requests_resolve_one_native_program_owner() {
    let application = host::install(model::authored_value()).unwrap();
    let foreign = host::install(model::authored_value()).unwrap();
    application
        .runtime()
        .with_host_advancement(|phase| {
            let selected = application
                .runtime()
                .on_branch(application.runtime().current_world())
                .select(&phase)
                .unwrap();
            let typed = application.selected_program_owner(&selected).unwrap();
            let plain = application
                .runtime()
                .selected_program_owner(&selected)
                .unwrap();
            assert_eq!(
                plain.owned_revision(),
                application.installed_program().revision()
            );
            assert_eq!(plain.owned_revision(), typed.owned_revision());
            assert!(plain.contains_action::<publish::PublishBinding>());
            assert!(plain.contains_action::<retire::Binding>());
            let edit_type = std::any::TypeId::of::<edit::EditBinding>();
            assert!(plain.owns_output_source(edit_type));
            assert_eq!(
                plain.owns_output_source(edit_type),
                typed.owns_output_source(edit_type)
            );
            let action = application
                .installed_program()
                .actions()
                .iter()
                .find(|action| action.mutation_binding_type() == Some(edit_type))
                .unwrap();
            let typed_action = typed
                .resolve_declared_action::<edit::EditBinding>(action)
                .unwrap();
            let plain_action = plain
                .resolve_declared_action::<edit::EditBinding>(action)
                .unwrap();
            assert!(
                std::ptr::eq(typed_action, plain_action),
                "both views borrow the original installed action allocation"
            );
            assert!(
                std::ptr::eq(plain_action, action),
                "selection cannot reconstruct an inventory from descriptive membership"
            );
            assert!(matches!(
                foreign.runtime().selected_program_owner(&selected),
                Err(WorthQuerySelectedProgramOwnerDenial::ProductSelection(
                    WorthQueryProductBranchAdmissionDenial::Refused(
                        WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner
                    )
                ))
            ));
        })
        .unwrap();
}
