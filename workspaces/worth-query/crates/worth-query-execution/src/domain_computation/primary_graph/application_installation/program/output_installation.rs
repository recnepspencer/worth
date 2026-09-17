use crate::domain_computation::primary_graph::WorthQueryApplicationRequiredOutputConnection;
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationNoOutputGraph, ApplicationOutputGraph,
    ApplicationProgramOutputShape,
};

/// Execution capability supplied only by the declared program output shapes.
///
/// This is public because it constrains the public program constructor. The
/// declaration crate seals the output-shape vocabulary, so application callers
/// cannot manufacture another installation posture.
#[doc(hidden)]
pub trait WorthQueryProgramOutputInstallation<Schema>:
    ApplicationProgramOutputShape<Schema>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    fn install_required_source(bindings: &mut std::collections::BTreeSet<std::any::TypeId>);
    fn required_source_operation() -> Option<std::any::TypeId>;
}

impl<Schema> WorthQueryProgramOutputInstallation<Schema> for ApplicationNoOutputGraph
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    fn install_required_source(_: &mut std::collections::BTreeSet<std::any::TypeId>) {}
    fn required_source_operation() -> Option<std::any::TypeId> {
        None
    }
}

impl<Schema, Root, Dependents> WorthQueryProgramOutputInstallation<Schema>
    for ApplicationOutputGraph<Root, Dependents>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Root: ApplicationConnectionShape<Schema>,
    Dependents:
        worth_query_declaration::facade::application_program::ApplicationOutputChildrenShape<
            Schema,
            Root::TargetFeature,
        >,
    Root::Binding: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    fn install_required_source(bindings: &mut std::collections::BTreeSet<std::any::TypeId>) {
        bindings.insert(std::any::TypeId::of::<
            <Root::Binding as WorthQueryApplicationRequiredOutputConnection<Schema>>::Source,
        >());
    }
    fn required_source_operation() -> Option<std::any::TypeId> {
        Some(std::any::TypeId::of::<<<Root::Binding as WorthQueryApplicationRequiredOutputConnection<Schema>>::Source as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::Operation>())
    }
}
