//! Ordinary public read attempts retain decoder refusal until fact completion.
pub use super::{host, operation::InputBinding, schema::*};
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_query_host::facade::{
    primary_graph::*, worth_query_operation, worth_query_operation_reads,
};

worth_query_operation!(pub ObserveCanonical for AdmissionSchema, input InputBinding);
worth_query_operation_reads!(ObserveCanonical => [DocumentValueField]);

pub struct Observed {
    pub second_refused: bool,
    pub completed: Result<(), WorthQueryApplicationAttemptDenial>,
}

pub fn observe_twice(host: &host::Host) -> Observed {
    super::opaque::reset();
    let runtime = host.runtime();
    let request = request_scope();
    let external = authenticate_operator(host.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let binding = host
                .installed_schema()
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::reference(),
                    "root".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = host
                .installed_schema()
                .installed_operation(ObserveCanonical::reference())
                .unwrap();
            let admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &root,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let mut attempt = runtime
                .begin_application_read_attempt(&execution, admission)
                .unwrap();
            let first = attempt
                .observe_field(&execution, &root, DocumentValueField::reference())
                .unwrap();
            assert_eq!(
                first.0,
                super::value(),
                "ordinary observations decode the supported nondefault source"
            );
            let second_refused =
                match attempt.observe_field(&execution, &root, DocumentValueField::reference()) {
                    Ok(value) => {
                        assert_eq!(value.0, super::value());
                        false
                    }
                    Err(_) => true, // Deliberately ignore the read error and try to complete facts.
                };
            let completed = attempt
                .complete(
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .map(drop);
            Observed {
                second_refused,
                completed,
            }
        })
        .expect("ordinary read attempt borrows the real request advancement")
}
