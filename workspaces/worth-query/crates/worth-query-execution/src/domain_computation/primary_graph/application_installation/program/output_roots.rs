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
    /// Visits declared bindings into a membership accumulator; no TypeId order exists.
    fn append_required_bindings(bindings: &mut impl FnMut(std::any::TypeId));
}

impl<Schema, Root, Dependents> WorthQueryApplicationProgramRoots<Schema>
    for ApplicationOutputGraph<Root, Dependents>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Self: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Self>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    fn append_required_bindings(bindings: &mut impl FnMut(std::any::TypeId)) {
        bindings(std::any::TypeId::of::<
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
    fn append_required_bindings(bindings: &mut impl FnMut(std::any::TypeId)) {
        bindings(std::any::TypeId::of::<
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
    fn append_required_bindings(_: &mut impl FnMut(std::any::TypeId)) {}
}

impl<Schema, Left, Right> WorthQueryApplicationProgramRoots<Schema> for (Left, Right)
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Left: WorthQueryApplicationProgramRoots<Schema>,
    Right: WorthQueryApplicationProgramRoots<Schema>,
{
    fn append_required_bindings(bindings: &mut impl FnMut(std::any::TypeId)) {
        Left::append_required_bindings(bindings);
        Right::append_required_bindings(bindings);
    }
}

impl<Schema, Roots> WorthQueryApplicationProgramRoots<Schema> for ApplicationProgramOutputs<Roots>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Roots: ApplicationProgramOutputRootsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    fn append_required_bindings(bindings: &mut impl FnMut(std::any::TypeId)) {
        Roots::append_required_bindings(bindings);
    }
}

/// Collect declaration visits, deduplicating by equality without walking the set.
pub(super) fn required_output_bindings<Schema, Roots>(
    actions: &[worth_query_declaration::facade::application_program::ApplicationActionDeclaration],
) -> Vec<crate::equality_type_id::EqualityTypeId>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Roots: WorthQueryApplicationProgramRoots<Schema>,
{
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    let mut append = |binding| {
        let binding = crate::equality_type_id::EqualityTypeId::from(binding);
        if seen.insert(binding) {
            result.push(binding);
        }
    };
    Roots::append_required_bindings(&mut append);
    for binding in actions.iter().filter_map(|action| {
        action
            .required_output_source()
            .then(|| action.mutation_binding_type())
            .flatten()
    }) {
        append(binding);
    }
    result
}
