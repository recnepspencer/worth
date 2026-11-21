//! Physical support of the actual compiled Source acquisition, including absences.
//! These coordinates describe admitted reads; they grant no Source authority.
mod commitment;
mod facts;
use super::occurrences::MaterialDenial;
use crate::domain_computation::primary_graph::{
    application_query::derived_view::ViewDependency, source_facts::SourceFactsAllocation,
};
pub use facts::AppliedSourceAcquisitionSupport;
use std::{collections::BTreeSet, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::DerivedRelationData;

pub struct SourceProjectionSupport
{
    dependencies: BTreeSet<ViewDependency>,
    _allocation: Arc<SourceFactsAllocation>,
}
impl SourceProjectionSupport {
    pub fn new(execution: &ExecutionRequest<'_, '_>) -> Result<Self, MaterialDenial> {
        Ok(Self {
            dependencies: BTreeSet::new(),
            _allocation: SourceFactsAllocation::open(execution)?,
        })
    }
    pub fn membership(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        read: &DerivedRelationData,
    ) -> Result<(), MaterialDenial> {
        for &(entity, _) in read.entity_dependencies() {
            self.record(execution, &ViewDependency::Entity(entity))?;
        }
        // Empty adjacency observations are retained too: later membership can
        // change even when this acquisition returned no Source occurrence.
        for edge in read.adjacency_dependencies() {
            self.record(
                execution,
                &ViewDependency::Adjacency(edge.anchor, edge.relation, u8::from(!edge.outgoing)),
            )?;
        }
        Ok(())
    }
    pub fn row(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        dependencies: &BTreeSet<ViewDependency>,
    ) -> Result<(), MaterialDenial> {
        for dependency in dependencies {
            self.record(execution, dependency)?;
        }
        Ok(())
    }
    fn record(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        dependency: &ViewDependency,
    ) -> Result<(), MaterialDenial> {
        execution.consult().map_err(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::from,
        )?;
        if !self.dependencies.contains(dependency) {
            self._allocation
                .claim_tree::<ViewDependency>(execution, self.dependencies.len())?;
            self._allocation
                .claim(execution, dependency.retained_bytes() as u64)?;
            self.dependencies.insert(dependency.clone());
        }
        Ok(())
    }
}
