//! Public source preparation admits DATA before the same receiver's one replay.
use super::*;
use worth_foundational::facade::{
    HomeProgramTransitionSchemaCarrierDigest, HomeProgramTransitionSchemaDigest,
};
use worth_relational::facade::durability::{
    RefusedPreparedRelationalHomeRecovery as Refusal,
    RelationalHomeRecoveryPreparationDenial as Denial,
};
use worth_testkit_relational::initial_input_source;

#[test]
fn prepared_home_schema_data_preserves_the_original_source_and_requires_its_receiving_installation()
{
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (source, medium) = initial_input_source::initialized(execution);
    create_entity(execution, &source, "before-preparation");
    let original = current(execution, &source);
    let offered = medium.offered().len();
    let mut receiver = durable_runtime(&medium);
    let prepared = receiver
        .durability()
        .prepare_home_recovery_source(
            execution,
            initial_input_source::journal_home_input(&medium, execution).unwrap(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    assert!(receiver.history().historical_latest_commit().is_none());
    assert_eq!(
        prepared.initial_schema_data(execution).unwrap(),
        &source.config().schema.registry,
    );
    let (_, basis) = source
        .owner_component_services()
        .basis_port()
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let contracts = source
        .prepare_home_program_contracts(
            execution,
            &basis,
            source.config().schema.registry.clone(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
    let target = contracts.target_meaning(execution, 0).unwrap();
    assert_eq!(
        prepared
            .schema_data(execution, target.carrier(), target.schema())
            .unwrap(),
        prepared.initial_schema_data(execution).unwrap(),
    );
    assert!(matches!(
        prepared.schema_data(
            execution,
            target.carrier(),
            HomeProgramTransitionSchemaDigest::from_recorded([42; 32]),
        ),
        Err(Denial::SchemaAuthorityMismatch)
    ));
    assert!(matches!(
        prepared.schema_data(
            execution,
            HomeProgramTransitionSchemaCarrierDigest::from_recorded([42; 32]),
            target.schema(),
        ),
        Err(Denial::SchemaCarrierMissing)
    ));
    let receipt = receiver
        .prepare_initial_schema_installation()
        .unwrap()
        .install(initial_input_source::additions())
        .unwrap();
    let receiver_identity = receiver.runtime_instance_id();
    let mut foreign = durable_runtime(&medium);
    let refused = foreign
        .durability_recovery()
        .recover_prepared_home_source(execution, prepared, &receipt, None)
        .unwrap_err();
    let prepared = match refused {
        Refusal::Preparation { prepared, cause } => {
            assert!(matches!(cause, Denial::ForeignReceiver));
            prepared
        }
        Refusal::Recovery(cause) => panic!("receiver must refuse before reconstruction: {cause:?}"),
        Refusal::Transition(cause) => panic!("no transition was offered: {cause:?}"),
    };
    assert!(foreign.history().historical_latest_commit().is_none());
    assert_eq!(medium.offered().len(), offered);
    let (outcome, recovered) = receiver
        .durability_recovery()
        .recover_prepared_home_source(execution, prepared, &receipt, None)
        .unwrap();
    assert_eq!(outcome.recovered_commits, 1);
    assert_ne!(receiver.runtime_instance_id(), receiver_identity);
    assert_eq!(medium.offered().len(), offered);
    assert_eq!(current(execution, &source), original);
    let readmitted = recovered
        .readmit_home_initial_schema_installation(execution, &receiver, receipt)
        .unwrap();
    assert_eq!(
        readmitted.runtime_instance_id(),
        receiver.runtime_instance_id()
    );
    assert_eq!(
        receiver.config().schema.registry,
        source.config().schema.registry
    );
    drop((readmitted, recovered, contracts, basis));
    receiver.prepare_close().unwrap().finish();
    foreign.prepare_close().unwrap().finish();
    source.prepare_close().unwrap().finish();
}
