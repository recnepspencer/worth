//! Sealed test doors over this module's private members.
use super::*;

pub trait PreparedExpectationDoorsOps<Schema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    #[doc(hidden)]
    fn prepare_application_source_expectation_for_test<Binding, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            Scope,
        >,
        source: WorthQueryObservedSource<
            <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
        input: &Binding::Input,
    ) -> Result<PreparedObservedSourceExpectation, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        Self::prepare_application_source_expectation::<Binding, Scope>(
            self, phase, admission, source, input,
        )
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > PreparedExpectationDoorsOps<Schema> for HostRuntime
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
}
