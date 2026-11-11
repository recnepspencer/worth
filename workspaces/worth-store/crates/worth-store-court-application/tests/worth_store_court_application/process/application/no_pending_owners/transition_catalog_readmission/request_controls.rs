//! The authentic catalog grant cannot replace its caller's original controls.
use super::*;
use worth_relational::facade::{
    durability::{RecoveredRelationalRuntimeAuthority, RelationalRecoveryStop},
    runtime::{
        PreparedRelationalHomeProgramContracts, RelationalHomeProgramCatalogReadmissionSource,
        RelationalHomeProgramContractDenial, RelationalRuntime,
    },
    transactions::RelationalExecutionDenialCause as Cause,
};

pub(super) fn refuse(
    native: &RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    grant: &worth_proof::AdmittedHomeProgramTransitionCatalog<
        RelationalHomeProgramCatalogReadmissionSource<'_>,
    >,
    contracts: &PreparedRelationalHomeProgramContracts,
) {
    let memory = worth_execution::SerialMemoryBudget::new(Some(0));
    let request = worth_execution::SerialRequest::from_memory(
        memory.clone(),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let refused = authority
        .readmit_home_program_catalogs(
            &ExecutionRequest::serial(&request),
            native,
            grant,
            contracts,
        )
        .unwrap_err();
    match refused.reason() {
        Denial::TargetMeaning(RelationalHomeProgramContractDenial::SchemaImage(
            RelationalRecoveryStop::Request(cause),
        )) => {
            assert!(matches!(
                *cause,
                Cause::PolicyMemoryExhausted {
                    admitted: 0,
                    ancestor: 0,
                    ..
                }
            ));
        }
        other => panic!("the original zero budget must refuse target schema capture: {other:?}"),
    }
    assert!(!refused.retains_acquired_custody());
    drop(refused);
    assert_eq!(memory.charged_memory_bytes(), 0);
    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let memory = worth_execution::SerialMemoryBudget::new(None);
    let request =
        worth_execution::SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
    let refused = authority
        .readmit_home_program_catalogs(
            &ExecutionRequest::serial(&request),
            native,
            grant,
            contracts,
        )
        .unwrap_err();
    match refused.reason() {
        Denial::Request(cause) => assert_eq!(Cause::from(*cause), Cause::Cancelled),
        other => panic!("the original cancellation must stop at entry: {other:?}"),
    }
    assert!(!refused.retains_acquired_custody());
    drop(refused);
    assert_eq!(memory.charged_memory_bytes(), 0);
}
