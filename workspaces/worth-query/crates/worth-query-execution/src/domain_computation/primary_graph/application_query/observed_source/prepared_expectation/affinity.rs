use super::*;

impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    pub(super) fn checked_source_affinity<'a, Binding, Scope>(
        &'a self,
        admission: &WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            Scope,
        >,
        source: &WorthQueryObservedSource<
            <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
        input: &Binding::Input,
        compare_parameters: impl FnOnce(
            &WorthQueryRetainedObservedParameters,
            worth_query_declaration::facade::application_query::ApplicationQueryParameterSet<
                <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
            >,
        ) -> Result<bool, WorthQuerySourceExpectationDenial>,
    ) -> Result<CheckedSourceAffinity<'a>, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        let expected_query_identifier = <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::QUERY_IDENTIFIER
            .ok_or_else(|| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                    Binding::IDENTITY,
                )
            })?;
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::ForeignApplication,
                Binding::IDENTITY,
            )
        })?;
        let expected_query_identity = self
            .installed_schema
            .installed_query_identity_by_name(expected_query_identifier)
            .ok_or_else(|| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                    expected_query_identifier,
                )
            })?;
        let selected_product = admission
            .graph_work()
            .mutation_product()
            .map(|product| {
                crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                    product.observation(),
                )
            })
            .ok_or_else(|| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::ForeignBranch,
                    Binding::IDENTITY,
                )
            })?;
        let parameters_denial = || {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::SourceParametersMismatch,
                Binding::IDENTITY,
            )
        };
        if let Some(expected) =
            Binding::expected_source_parameters(input).map_err(|_| parameters_denial())?
        {
            if !compare_parameters(&source.parameters, expected)? {
                return Err(parameters_denial());
            }
        }
        source.validate_affinity(
            self.runtime.authority_identity().as_u64(),
            &self.installed_schema.binding_identity(),
            admission.graph_work_branch(),
            admission.scope_entity_id(),
            &selected_product,
            expected_query_identifier,
            expected_query_identity,
        )?;
        Ok(CheckedSourceAffinity {
            layout: &graph.layout,
            selected_product,
            expected_query_identifier,
        })
    }
}

pub(super) fn parameter_denial(
    subject: &str,
    denial: worth_query_admission::facade::application_query::WorthQueryApplicationQueryParameterDenial,
) -> WorthQuerySourceExpectationDenial {
    use worth_query_admission::facade::application_query::WorthQueryApplicationQueryParameterDenialKind as Kind;
    WorthQuerySourceExpectationDenial::new(
        match denial.kind() {
            Kind::CanonicalEntryBudgetExceeded | Kind::CanonicalEncodedByteBudgetExceeded => {
                WorthQuerySourceExpectationDenialKind::WorkBudgetExceeded
            }
            _ => WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
        },
        subject,
    )
}
