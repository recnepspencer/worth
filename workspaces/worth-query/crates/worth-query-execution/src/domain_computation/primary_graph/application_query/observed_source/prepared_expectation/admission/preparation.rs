//! Source facts can be prepared only from the validation owner's shared loan.
use super::*;
impl<Schema: worth_query_installation::facade::ApplicationSchema, Operation, Input, Scope>
    CheckedSourceAdmission<'_, Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph::application_query::observed_source::prepared_expectation) fn prepare<
        Binding,
    >(
        self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        source: WorthQueryObservedSource<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>,
        input: &Input,
    ) -> Result<PreparedObservedSourceExpectation, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
            Operation = Operation,
            Input = Input,
        >,
    {
        let Self { runtime, admission } = self;

        let identity = source.idempotency_identity().bytes();
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
        let control = SourceFactExpansionControl::for_phase(
            runtime,
            phase,
            Some(admission.publication_request()),
            Binding::IDENTITY,
        )?;
        let prepared_facts =
            source.admit_fact_materialization(checked.layout, Binding::IDENTITY, control)?;
        let partition_identity = source.partition_identity();
        let facts = prepared_facts.into_facts()?;
        let context = source.retain_expression_binding();
        let super::WorthQueryApplicationBasisSelectionIdentity::Product(observed_product) =
            source.selection
        else {
            unreachable!("validated product source retains its selected observation")
        };
        Ok(PreparedObservedSourceExpectation {
            context,
            admission_identity: admission.admission_identity(),
            selected_product: checked.selected_product,
            observed_product,
            bound: WorthQueryBoundSourceExpectation {
                kind: WorthQuerySourceExpectationKind::Row,
                identity,
                partition_identity,
            },
            facts,
        })
    }
}
