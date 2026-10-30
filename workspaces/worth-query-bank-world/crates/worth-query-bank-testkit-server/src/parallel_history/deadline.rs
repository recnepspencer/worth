//! Dispatch-time deadlines run at every posture through the real Bank doors.
pub use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial as Admission,
        WorthQueryManagedComputationInterruption as Interruption,
    },
    application_entry::WorthQueryApplicationOutputDemandDenial as DemandDenial,
    primary_graph::{
        advancement_observations_on_this_thread_for_test as observations,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionObservation, WorthQueryExecutionPlacementForTest as Placement,
        WorthQueryOutputDemandDenialKind as Kind,
    },
};

pub fn zero_bill() {
    let records = observations();
    assert_eq!(
        records.len(),
        1,
        "one deadline-stopped Bank advancement is observed"
    );
    assert_eq!(
        (
            records[0].report.charged_work(),
            records[0].report.charged_span()
        ),
        (0, 0),
        "a present Bank deadline dispatches and charges no work"
    );
    assert_eq!(
        records[0].applied.len(),
        0,
        "a present Bank deadline applies no commit or publication"
    );
    assert_eq!(
        records[0].workers, None,
        "deadline refusal precedes worker dispatch"
    );
}

pub fn assessment(
    width: usize,
    action: impl FnOnce() -> Result<
        bank_server::BankApprovedPaymentAssessmentProgress,
        bank_server::BankApprovedPaymentWorkflowError,
    >,
) {
    observations();
    let Err(bank_server::BankApprovedPaymentWorkflowError::AssessmentDemand(cause)) = action()
    else {
        panic!("expired assessment keeps the demand's typed refusal")
    };
    let DemandDenial::Demand(cause) = *cause else {
        panic!("expired assessment keeps its advancement refusal")
    };
    assert_eq!(
        cause.kind(),
        Kind::ExecutionRequest(Admission::Interrupted(Interruption::DeadlineExceeded)),
        "assessment deadline kind is asserted by value"
    );
    zero_bill();
    println!("BANK_DEADLINE:assessment:width={width}:work=0:span=0:commits=0:DeadlineExceeded");
}
