//! The recorded installation relates A/B DATA without retaining N's live Home.
use worth_foundational::facade::{
    ApplicationHomeBinding, HomeProgramTransitionInstallationBinding,
};
use worth_proof::{AdmittedHomeProgramTransitionSuccessor, HomeProgramTransitionAdmissionDenial};
use worth_relational::facade::runtime::RelationalHomeProgramContractDenial;
use worth_relational::facade::runtime::{
    PreparedRelationalHomeProgramContracts, RelationalInitialSchemaInstallationReceipt,
};
type Home = worth_store_application::WorldReadmittedApplicationHome<(), (), (), (), ()>;
type Installation =
    AdmittedHomeProgramTransitionSuccessor<HomeProgramTransitionInstallationBinding>;

pub(super) fn matching_preparation(
    home: &Home,
    source: &worth_runtime_world::facade::ProductBranchObservation,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> PreparedRelationalHomeProgramContracts {
    home.relational()
        .runtime()
        .prepare_home_program_contracts(
            request,
            source.basis().relational_basis(),
            super::super::target::registry(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap()
        .with_receiver_configuration(
            request,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE).resolve(),
        )
        .unwrap()
}

pub(super) fn admit_exact_target(
    home: &Home,
    actual: &PreparedRelationalHomeProgramContracts,
    matching: &PreparedRelationalHomeProgramContracts,
    receipt: &RelationalInitialSchemaInstallationReceipt,
    proof: &Installation,
) {
    let native = home.relational().runtime();
    let before = native.history().historical_latest_commit();
    native
        .admit_installed_home_program_target(actual, receipt, proof)
        .unwrap();
    assert!(
        matches!(
            native.admit_installed_home_program_target(matching, receipt, proof),
            Err(RelationalHomeProgramContractDenial::TargetCatalogMismatch)
        ),
        "equal catalog DATA cannot replace the actual installed preparation"
    );
    assert_eq!(native.history().historical_latest_commit(), before);
}

pub(super) fn current(home: &Home, original: &ApplicationHomeBinding) -> Installation {
    let proof = home
        .home_program_transition_installation_binding()
        .unwrap()
        .unwrap();
    home.relational()
        .runtime()
        .admit_home_program_transition_installation(&proof)
        .unwrap();
    assert_eq!(
        proof.binding().origin(),
        [original.package(), original.schema()]
    );
    let installed = home.binding();
    assert_eq!(
        proof.binding().successor(),
        [
            installed.package(),
            installed.schema(),
            installed.selected().revision()
        ]
    );
    proof
}

pub(super) fn reopened(home: &Home, earlier: Installation) {
    let proof = home
        .home_program_transition_installation_binding()
        .unwrap()
        .unwrap();
    assert_eq!(
        proof.binding(),
        earlier.binding(),
        "the original installation meaning survives the same Home's reopen unchanged"
    );
    let native = home.relational().runtime();
    let before = native.history().historical_latest_commit();
    native
        .admit_home_program_transition_installation(&proof)
        .unwrap();
    assert!(matches!(
        native.admit_home_program_transition_installation(&earlier),
        Err(RelationalHomeProgramContractDenial::Admission(
            HomeProgramTransitionAdmissionDenial::StaleIncarnation
        ))
    ));
    assert_eq!(native.history().historical_latest_commit(), before);
}
