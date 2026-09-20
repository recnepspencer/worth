use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationRequiredOutputConnection,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationDiscoveredOutputGraph, ApplicationOutputGraph,
    ApplicationOutputGraphShape, ApplicationProgramOutputRootsShape, ApplicationProgramOutputs,
};

type RootConnectionRef<Schema, Root> =
    <Root as ApplicationOutputGraphShape<Schema>>::RootConnection;
type RootConnection<Schema, Root> =
    <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding;

pub trait WorthQueryApplicationProgramRoots<Schema>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    fn append_required_bindings(bindings: &mut std::collections::BTreeSet<std::any::TypeId>);
}

impl<Schema, Root, Dependents> WorthQueryApplicationProgramRoots<Schema>
    for ApplicationOutputGraph<Root, Dependents>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Self: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Self>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    fn append_required_bindings(bindings: &mut std::collections::BTreeSet<std::any::TypeId>) {
        bindings.insert(std::any::TypeId::of::<
            <RootConnection<Schema, Self> as WorthQueryApplicationRequiredOutputConnection<
                Schema,
            >>::Source,
        >());
    }
}

impl<Schema, Root, Dependents> WorthQueryApplicationProgramRoots<Schema>
    for ApplicationDiscoveredOutputGraph<Root, Dependents>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Self: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Self>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    fn append_required_bindings(bindings: &mut std::collections::BTreeSet<std::any::TypeId>) {
        bindings.insert(std::any::TypeId::of::<
            <RootConnection<Schema, Self> as WorthQueryApplicationDiscoveredOutputConnection<
                Schema,
            >>::Source,
        >());
    }
}

impl<Schema> WorthQueryApplicationProgramRoots<Schema>
    for worth_query_declaration::facade::application_program::ApplicationNoOutputGraph
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    fn append_required_bindings(_: &mut std::collections::BTreeSet<std::any::TypeId>) {}
}

impl<Schema, Left, Right> WorthQueryApplicationProgramRoots<Schema> for (Left, Right)
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Left: WorthQueryApplicationProgramRoots<Schema>,
    Right: WorthQueryApplicationProgramRoots<Schema>,
{
    fn append_required_bindings(bindings: &mut std::collections::BTreeSet<std::any::TypeId>) {
        Left::append_required_bindings(bindings);
        Right::append_required_bindings(bindings);
    }
}

impl<Schema, Roots> WorthQueryApplicationProgramRoots<Schema> for ApplicationProgramOutputs<Roots>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Roots: ApplicationProgramOutputRootsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    fn append_required_bindings(bindings: &mut std::collections::BTreeSet<std::any::TypeId>) {
        Roots::append_required_bindings(bindings);
    }
}
