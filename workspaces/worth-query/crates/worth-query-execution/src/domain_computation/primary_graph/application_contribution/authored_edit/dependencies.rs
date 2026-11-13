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
    pub family_type: std::any::TypeId,
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
        demand: D,
        position: worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
        order: worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
    ) -> Result<Self, WorthQueryOutputDemandDenial>
    where
        B: ApplicationMutationBinding<S>,
        D: WorthQueryApplicationOutputDemand<S> + Clone + Send + Sync,
        E: super::super::producer::synchronous_outputs::SynchronousOutputEdges<S, D> + 'static,
        super::super::producer::synchronous_outputs::source::SourceInput<S, D>: worth_query_declaration::facade::application_query::ApplicationQueryIntent<S, Binding = super::super::producer::synchronous_outputs::source::Source<S, D>>,
        <super::super::producer::synchronous_outputs::source::Source<S, D> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<S>>::ScopeBinding: worth_query_declaration::facade::application_query::ApplicationQueryScopeResolution<S, <super::super::producer::synchronous_outputs::source::Source<S, D> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<S>>::PrincipalIdentity>,
{
        let mut families = declared_families::<S, B>();
        let root = crate::equality_type_id::EqualityTypeId::of::<D::OutputFamily>().into_external();
        families.push(root);
        E::declared_families(root, &mut |parent, family| {
            if !families.contains(&family) {
                families.push(family);
            }
            self.producer_connections.push((parent, family));
        });
        self.producers = application
            .installed_producers
            .declared_dependency_scopes(application, &families)?;
        self.required = Some(std::sync::Arc::new(move |execution| {
            super::super::producer::synchronous_outputs::CapturedSynchronousOutputs::fixed::<S, D, E>(
                execution,
                position.identity(),
                demand.clone(),
                position.clone(),
                &order,
            )
        }));
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
