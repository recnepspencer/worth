//! Source binding consumes the validation owner's unique admission loan.
use super::*;
impl<Schema: worth_query_installation::facade::ApplicationSchema, Operation, Input, Scope>
    CheckedMutableSourceAdmission<'_, Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph::application_query::observed_source::prepared_expectation) fn bind<Binding>(
        self,
        phase: &crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase<'_>,
        source: WorthQueryObservedSource<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>,
        identity: [u8; 32],
        input: &Input,
        kind: WorthQuerySourceExpectationKind,
    ) -> Result<WorthQueryPendingSourceExpectation<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>, WorthQuerySourceExpectationDenial>
    where Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
{
        let Self { runtime, admission } = self;

        let checked = runtime.checked_source_affinity::<Binding, Scope>(
            admission,
            &source,
            input,
            |parameters, expected| {
                parameters
                    .matches_expected(expected)
                    .map_err(|denial| parameter_denial(Binding::IDENTITY, denial))
            },
        )?;
        source.validate_fact_retention(
            checked.layout,
            checked.expected_query_identifier,
            SourceFactExpansionControl::for_phase(
                runtime,
                phase,
                Some(admission.publication_request()),
                Binding::IDENTITY,
            )?,
        )?;
        let bound = WorthQueryBoundSourceExpectation {
            kind,
            identity,
            partition_identity: source.partition_identity(),
        };
        admission.bind_source_partition(bound.partition_identity);
        admission.bind_source_expectation_kind(bound.kind);
        Ok(WorthQueryPendingSourceExpectation {
            admission_identity: admission.admission_identity(),
            selected_product: checked.selected_product,
            expected_query_identifier: checked.expected_query_identifier,
            bound,
            source,
        })
    }
}
