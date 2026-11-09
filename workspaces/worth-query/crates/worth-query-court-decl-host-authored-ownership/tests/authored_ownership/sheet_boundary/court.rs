//! A one-stage square-to-triangle edit must be current at its own settlement.
use super::super::cold_owned_revision::{create, settle, RequiredProgram, Root};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
#[test]
fn a_trimmed_sheet_revision_is_current_after_one_owned_publication() {
    assert_eq!(trim(), 1);
}
fn trim() -> usize {
    type S = Model<69, heterogeneous::Absent, sheet_boundary::Owned>;
    let host = correction_courts::install_definition::<S, RequiredProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    coupled::producer::handler::reset_contacts();
    let before = coupled::producer::handler::contacts();
    let outcome = request
        .advanced()
        .mutate(parameter_inverse::Edit { id: 10, value: 1 })
        .without_source()
        .idempotency(&"trim-sheet".to_owned())
        .execute_performed::<RequiredProgram, Root<S>>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let performed = match outcome {
        WorthQueryApplicationPerformedMutationOutcome::Performed(performed) => performed,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
            panic!("source must perform: {outcome:?}")
        }
        WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(failure) => {
            panic!("required output denial: {:?}", failure.denial())
        }
        _ => panic!("source has not reached its performed owner"),
    };
    let mut started = performed
        .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
    for _ in 0..64 {
        match started
            .required_output_mut()
            .advance(&host, &request)
            .unwrap()
        {
            WorthQueryApplicationProgramOutputProgress::Settled(output) => {
                let current = request.retain_current_read().unwrap();
                request
                    .at(&current)
                    .require_current_program_output(&output)
                    .unwrap();
                return coupled::producer::handler::contacts() - before;
            }
            WorthQueryApplicationProgramOutputProgress::Pending
            | WorthQueryApplicationProgramOutputProgress::Progressed => {}
        }
    }
    panic!("trim must settle once");
}
