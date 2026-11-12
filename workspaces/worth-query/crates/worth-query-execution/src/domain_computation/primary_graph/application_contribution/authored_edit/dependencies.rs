//! Installed graph and Query contracts supply envelope dependency declarations.
use super::*;
use worth_query_installation::facade::{
    WorthQueryInstalledGraphReadContract, WorthQueryOperationReadTouchOverlapIndex,
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredQueryRead {
    pub identity: String,
    pub contract: WorthQueryInstalledGraphReadContract,
}
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph::application_contribution) struct ProducerDependencyScopes
{
    pub identity: String,
    pub family: &'static str,
    pub contract: WorthQueryOperationReadTouchOverlapIndex,
    pub queries: Vec<DeclaredQueryRead>,
}
pub(in crate::domain_computation::primary_graph::application_contribution) fn managed_reads<
    S: ApplicationSchema,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    references: worth_query_installation::facade::WorthQueryOperationManagedIndexReadView<'_>,
) -> Result<Vec<DeclaredQueryRead>, WorthQueryOutputDemandDenial> {
    references
        .iter()
        .map(|reference| {
            let contract = application
                .installed_schema
                .declared_query_read_contract(reference)
                .ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::ForeignSource,
                        format!(
                            "declared Query dependency {} is not installed",
                            reference.parts().binding.as_str()
                        ),
                    )
                })?;
            Ok(DeclaredQueryRead {
                identity: reference.parts().binding.as_str().into(),
                contract,
            })
        })
        .collect()
}
impl<S: ApplicationSchema> CapturedAuthoredEditStage<S> {
    pub fn with_required_output_dependencies<B, D, E>(
        mut self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<Self, WorthQueryOutputDemandDenial>
    where
        B: ApplicationMutationBinding<S>,
        D: WorthQueryApplicationOutputDemand<S>,
        E: super::super::producer::synchronous_outputs::SynchronousOutputEdges<S, D>,
    {
        let mut families = declared_families::<S, B>();
        families
            .push(crate::equality_type_id::EqualityTypeId::of::<D::OutputFamily>().into_external());
        E::declared_families(&mut |family| {
            if !families.contains(&family) {
                families.push(family);
            }
        });
        self.producers = application
            .installed_producers
            .declared_dependency_scopes(application, &families)?;
        Ok(self)
    }
}

struct DeclaredFamilies(Vec<std::any::TypeId>);
fn declared_families<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
) -> Vec<std::any::TypeId> {
    let mut visitor = DeclaredFamilies(Vec::new());
    B::visit_coupled_outputs(&mut visitor);
    visitor.0
}
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    worth_query_declaration::facade::application_program::ApplicationCoupledOutputVisitor<
        Schema,
        Binding,
    > for DeclaredFamilies
{
    fn required<Connection>(&mut self)
    where Connection: worth_query_declaration::facade::application_program::WorthQueryApplicationRequiredOutputConnection<Schema>, Connection::Demand: Send, Binding: worth_query_declaration::facade::application_program::WorthQueryApplicationRequiredOutputSource<Schema, Connection>,
        <DeclaredSource<Schema, Connection> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Input: worth_query_declaration::facade::application_query::ApplicationQueryIntent<Schema, Binding = DeclaredSource<Schema, Connection>>,
        <DeclaredSource<Schema, Connection> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ScopeBinding: worth_query_declaration::facade::application_query::ApplicationQueryScopeResolution<Schema, <DeclaredSource<Schema, Connection> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
{
        self.0.push(crate::equality_type_id::EqualityTypeId::of::<<<Connection as worth_query_declaration::facade::application_program::WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily>().into_external());
    }
}
type DeclaredSource<S, C> = <<<C as worth_query_declaration::facade::application_program::WorthQueryApplicationRequiredOutputConnection<S>>::Demand as WorthQueryApplicationOutputDemand<S>>::OutputFamily as worth_query_declaration::facade::application_program::WorthQueryProducerOutputFamily<S>>::Source;
