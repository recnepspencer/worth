//! Borrowed owner access across the sealed Relational package boundary.
use super::*;

#[doc(hidden)]
pub trait StructuralRelationViewOwnerPort<'runtime>: Sized {
    fn new(
        state_view: InvariantStateView<'runtime>,
        execution: worth_execution::ExecutionRequest<'runtime, 'runtime>,
        work: super::super::CustomInvariantWorkMeter,
        access: std::sync::Arc<
            crate::contracts::validation::data::descriptor::CustomInvariantAccessContract,
        >,
    ) -> Self;
}

impl<'runtime> StructuralRelationViewOwnerPort<'runtime> for StructuralRelationView<'runtime> {
    fn new(
        state_view: InvariantStateView<'runtime>,
        execution: worth_execution::ExecutionRequest<'runtime, 'runtime>,
        work: super::super::CustomInvariantWorkMeter,
        access: std::sync::Arc<
            crate::contracts::validation::data::descriptor::CustomInvariantAccessContract,
        >,
    ) -> Self {
        Self::new(state_view, execution, work, access)
    }
}

#[doc(hidden)]
pub trait StructuralAspectStateViewOwnerPort<'runtime>: Sized {
    fn new(
        state_view: InvariantStateView<'runtime>,
        work: super::super::CustomInvariantWorkMeter,
        access: std::sync::Arc<
            crate::contracts::validation::data::descriptor::CustomInvariantAccessContract,
        >,
    ) -> Self;
}

impl<'runtime> StructuralAspectStateViewOwnerPort<'runtime>
    for StructuralAspectStateView<'runtime>
{
    fn new(
        state_view: InvariantStateView<'runtime>,
        work: super::super::CustomInvariantWorkMeter,
        access: std::sync::Arc<
            crate::contracts::validation::data::descriptor::CustomInvariantAccessContract,
        >,
    ) -> Self {
        Self::new(state_view, work, access)
    }
}

#[doc(hidden)]
pub trait StructuralRelationViewFixturePort<'runtime> {
    fn structural_source_for_test(
        &self,
    ) -> &Option<(
        worth_relational_branch::internal_port::schema::registry_custody::SchemaRegistryCustody,
        crate::validation::engine::InvariantSourceAffinity,
    )>;
}

impl<'runtime> StructuralRelationViewFixturePort<'runtime> for StructuralRelationView<'runtime> {
    fn structural_source_for_test(
        &self,
    ) -> &Option<(
        worth_relational_branch::internal_port::schema::registry_custody::SchemaRegistryCustody,
        crate::validation::engine::InvariantSourceAffinity,
    )> {
        &self.structural_source
    }
}
