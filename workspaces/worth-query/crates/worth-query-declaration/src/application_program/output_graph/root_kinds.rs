use super::{sealed, ApplicationDiscoveredOutputGraph, ApplicationOutputGraph};

/// A discovered root cannot enter the single required-root lane.
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_program::{
///     ApplicationDiscoveredOutputGraph, ApplicationOutputLeaf, ApplicationRequiredOutputRoot,
/// };
/// fn required<Root: ApplicationRequiredOutputRoot>() {}
/// fn main() { required::<ApplicationDiscoveredOutputGraph<(), ApplicationOutputLeaf>>(); }
/// ```
pub trait ApplicationRequiredOutputRoot: sealed::RequiredRootKind {}

/// A single required root cannot enter the discovered-root lane.
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_program::{
///     ApplicationOutputGraph, ApplicationOutputLeaf, ApplicationDiscoveredOutputRoot,
/// };
/// fn discovered<Root: ApplicationDiscoveredOutputRoot>() {}
/// fn main() { discovered::<ApplicationOutputGraph<(), ApplicationOutputLeaf>>(); }
/// ```
pub trait ApplicationDiscoveredOutputRoot: sealed::DiscoveredRootKind {}

impl<Connection, Dependents> sealed::RequiredRootKind
    for ApplicationOutputGraph<Connection, Dependents>
{
}
impl<Connection, Dependents> ApplicationRequiredOutputRoot
    for ApplicationOutputGraph<Connection, Dependents>
{
}
impl<Connection, Dependents> sealed::DiscoveredRootKind
    for ApplicationDiscoveredOutputGraph<Connection, Dependents>
{
}
impl<Connection, Dependents> ApplicationDiscoveredOutputRoot
    for ApplicationDiscoveredOutputGraph<Connection, Dependents>
{
}
