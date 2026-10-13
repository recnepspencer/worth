use super::{
    sealed, ApplicationDemandOutputGraph, ApplicationDiscoveredOutputGraph, ApplicationOutputGraph,
};

/// A discovered root cannot enter the single required-root lane.
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_program::{
///     ApplicationDiscoveredOutputGraph, ApplicationOutputLeaf, ApplicationRequiredOutputRoot,
/// };
/// fn required<Root: ApplicationRequiredOutputRoot>() {}
/// fn main() { required::<ApplicationDiscoveredOutputGraph<(), ApplicationOutputLeaf>>(); }
/// ```
pub trait ApplicationRequiredOutputRoot:
    ApplicationFixedOutputRoot + sealed::RequiredRootKind
{
}

/// A statically selected installed root, either required or explicitly demanded.
/// Demand-only roots cannot acquire performed/prepared source authority.
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_program::{
///     ApplicationDemandOutputGraph, ApplicationOutputLeaf, ApplicationRequiredOutputRoot,
/// };
/// fn required<Root: ApplicationRequiredOutputRoot>() {}
/// fn main() { required::<ApplicationDemandOutputGraph<(), ApplicationOutputLeaf>>(); }
/// ```
///
/// ```
/// use worth_query_declaration::facade::application_program::{
///     ApplicationDemandOutputGraph, ApplicationOutputGraph, ApplicationOutputLeaf,
///     ApplicationFixedOutputRoot,
/// };
/// fn fixed<Root: ApplicationFixedOutputRoot>() {}
/// fn main() {
///     fixed::<ApplicationDemandOutputGraph<(), ApplicationOutputLeaf>>();
///     fixed::<ApplicationOutputGraph<(), ApplicationOutputLeaf>>();
/// }
/// ```
pub trait ApplicationFixedOutputRoot: sealed::FixedRootKind {}
impl<C, D> sealed::FixedRootKind for ApplicationOutputGraph<C, D> {}
impl<C, D> ApplicationFixedOutputRoot for ApplicationOutputGraph<C, D> {}
impl<C, D> sealed::FixedRootKind for ApplicationDemandOutputGraph<C, D> {}
impl<C, D> ApplicationFixedOutputRoot for ApplicationDemandOutputGraph<C, D> {}

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
