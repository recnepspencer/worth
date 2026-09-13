use super::{
    ImportPublicationAction, ImportPublicationModel, ImportPublicationModelDenial,
    ImportPublicationState,
};

#[test]
fn legal_publication_progression_records_the_exact_actions() {
    let mut model = admitted_layout_model();
    model.admit_publication_readiness().unwrap();
    model.complete_publication(true).unwrap();

    assert_eq!(model.state(), ImportPublicationState::PublicationDurable);
    assert_eq!(
        model.actions().collect::<Vec<_>>(),
        vec![
            ImportPublicationAction::RawDeclarationObserved,
            ImportPublicationAction::CurrentScopeReadmitted,
            ImportPublicationAction::RecoveredArtifactAdmitted,
            ImportPublicationAction::LayoutMaterializationAdmitted,
            ImportPublicationAction::PublicationPending,
            ImportPublicationAction::PublicationDurable,
        ]
    );
}

#[test]
fn crash_before_publication_returns_to_materialized_without_durability() {
    let mut model = admitted_layout_model();
    model.admit_publication_readiness().unwrap();
    model.crash();

    assert_eq!(model.state(), ImportPublicationState::LayoutMaterialized);
    let actions = model.actions().collect::<Vec<_>>();
    assert_eq!(
        actions.last(),
        Some(&ImportPublicationAction::CrashBeforePublication)
    );
    assert!(!actions.contains(&ImportPublicationAction::PublicationDurable));
}

#[test]
fn publication_requires_the_exact_physical_outcome() {
    let mut model = admitted_layout_model();
    model.admit_publication_readiness().unwrap();

    assert_eq!(
        model.complete_publication(false),
        Err(ImportPublicationModelDenial::ExactPhysicalPublicationRequired)
    );
    assert_eq!(model.state(), ImportPublicationState::PublicationDenied);
    assert_eq!(
        model.actions().last(),
        Some(ImportPublicationAction::PublicationDenied)
    );
}

#[test]
fn owner_frontiers_cannot_be_skipped() {
    let mut raw = ImportPublicationModel::from_raw_declaration();
    assert_eq!(
        raw.admit_recovered_artifact(),
        Err(ImportPublicationModelDenial::CurrentScopeReadmissionRequired)
    );
    assert_eq!(
        raw.admit_layout_materialization(),
        Err(ImportPublicationModelDenial::RecoveredArtifactAdmissionRequired)
    );
    assert_eq!(
        raw.admit_publication_readiness(),
        Err(ImportPublicationModelDenial::LayoutMaterializationRequired)
    );
    assert_eq!(
        raw.complete_publication(true),
        Err(ImportPublicationModelDenial::PublicationReadinessRequired)
    );
}

fn admitted_layout_model() -> ImportPublicationModel {
    let mut model = ImportPublicationModel::from_raw_declaration();
    model.readmit_current_scope();
    model.admit_recovered_artifact().unwrap();
    model.admit_layout_materialization().unwrap();
    model
}
