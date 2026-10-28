//! Disclosed occurrences, not dependency footprints or endpoint reconstruction, select subjects.
use worth_query_declaration::facade::application_query::{
    ApplicationHistoryTarget, ApplicationQueryResultTraversalDirection,
};
use worth_query_installation::facade::{
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledGraphRelation,
};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    history::RelationalRecordHistorySubject,
    runtime::{RelationalRuntime, VisibilityProjectionView},
    transactions::RecordRef,
};

use super::super::super::projection::{
    WorthQueryApplicationProjectedRelation, WorthQueryApplicationProjectionNode,
};
use super::super::{denial::WorthQueryApplicationHistoryDenial as Denial, request_stage};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

pub(super) struct CaptureWalk<'source, 'request, 'authority> {
    pub(super) execution: worth_execution::ExecutionRequest<'request, 'authority>,
    pub(super) runtime: &'source RelationalRuntime,
    pub(super) projection: &'source VisibilityProjectionView,
    pub(super) layout: &'source WorthQueryPrimaryGraphLayout,
    pub(super) contract: &'source WorthQueryInstalledGraphReadContract,
    pub(super) target: &'source ApplicationHistoryTarget,
    pub(super) basis: &'source AdmittedRelationalBranchBasis,
    pub(super) subjects: worth_execution::ExecutionRequestArrayBuilder<
        'request,
        'authority,
        RelationalRecordHistorySubject,
    >,
    pub(super) maximum_visits: usize,
    pub(super) visits: usize,
}

impl CaptureWalk<'_, '_, '_> {
    pub(super) fn nodes(
        &mut self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        nodes: &[WorthQueryApplicationProjectionNode],
        entity_name: &str,
    ) -> Result<(), Denial> {
        for node in nodes {
            self.visit(context)?;
            if matches!(self.target, ApplicationHistoryTarget::Entity { entity } if entity == entity_name)
            {
                self.entity(context, node, entity_name)?;
            }
            let contract = self.contract;
            for relation in node.relations() {
                request_stage::checkpoint(context, 1)?;
                let mut installed = None;
                for candidate in contract.relations() {
                    request_stage::checkpoint(context, 1)?;
                    if candidate.result_path() == relation.result_path() {
                        installed = Some(candidate);
                        break;
                    }
                }
                let installed = installed.ok_or(Denial::SubjectMismatch)?;
                if self.selects_relation(relation, installed) {
                    for child in relation.rows() {
                        self.relation(context, node, child, installed)?;
                    }
                }
                self.nodes(context, relation.rows(), installed.child_entity())?;
            }
        }
        Ok(())
    }

    fn selects_relation(
        &self,
        projected: &WorthQueryApplicationProjectedRelation,
        installed: &WorthQueryInstalledGraphRelation,
    ) -> bool {
        let ApplicationHistoryTarget::Relation {
            query,
            slot,
            relation,
            from,
            to,
            direction,
            cardinality,
            output_name,
        } = self.target
        else {
            return false;
        };
        projected.slot_key().query_identity() == *query
            && projected.slot_key().slot_identity() == *slot
            && installed.relation() == relation
            && installed.from() == from
            && installed.to() == to
            && installed.direction() == *direction
            && installed.cardinality() == *cardinality
            && installed.output_name() == output_name
    }

    fn entity(
        &mut self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        node: &WorthQueryApplicationProjectionNode,
        name: &str,
    ) -> Result<(), Denial> {
        self.visit(context)?;
        let bound = self
            .projection
            .exact_entity_state_read_work_bound()
            .ok_or(Denial::SubjectMismatch)?;
        request_stage::checkpoint(context, bound)?;
        let metadata = self
            .projection
            .read_exact_entity_state(&self.execution,node.entity_id(), |metadata, _| metadata)
            .map_err(|cause| match cause { worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired=>Denial::SubjectMismatch, worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::Execution(cause)=>Denial::Native(worth_relational::facade::history::RelationalRecordHistoryDenial::Storage(cause)), })?
            .ok_or(Denial::SubjectMismatch)?;
        if metadata.entity_id != node.entity_id()
            || self.layout.entity_kind(name) != Some(metadata.kind_id)
        {
            return Err(Denial::SubjectMismatch);
        }
        self.subject(context, RecordRef::Entity(metadata.entity_id))
    }

    fn relation(
        &mut self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        parent: &WorthQueryApplicationProjectionNode,
        child: &WorthQueryApplicationProjectionNode,
        installed: &WorthQueryInstalledGraphRelation,
    ) -> Result<(), Denial> {
        let relation = child.incoming_relation().ok_or(Denial::SubjectMismatch)?;
        self.visit(context)?;
        let bound = self
            .projection
            .exact_relation_metadata_read_work_bound()
            .ok_or(Denial::SubjectMismatch)?;
        request_stage::checkpoint(context, bound)?;
        let metadata = self
            .projection
            .read_exact_relation_metadata(&self.execution,relation)
            .map_err(|cause| match cause { worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired=>Denial::SubjectMismatch, worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::Execution(cause)=>Denial::Native(worth_relational::facade::history::RelationalRecordHistoryDenial::Storage(cause)), })?
            .ok_or(Denial::SubjectMismatch)?;
        let layout = self
            .layout
            .relation(installed.relation())
            .ok_or(Denial::SubjectMismatch)?;
        let (source, target) = match installed.direction() {
            ApplicationQueryResultTraversalDirection::Forward => {
                (parent.entity_id(), child.entity_id())
            }
            ApplicationQueryResultTraversalDirection::Reverse => {
                (child.entity_id(), parent.entity_id())
            }
        };
        if metadata.relation_id != relation
            || metadata.kind_id != layout.kind
            || metadata.source != source
            || metadata.target != target
            || self.layout.entity_kind(installed.from()) != Some(layout.from)
            || self.layout.entity_kind(installed.to()) != Some(layout.to)
        {
            return Err(Denial::SubjectMismatch);
        }
        self.subject(context, RecordRef::Relation(metadata.relation_id))
    }

    fn subject(
        &mut self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        record: RecordRef,
    ) -> Result<(), Denial> {
        for subject in self.subjects.elements() {
            request_stage::checkpoint(context, 1)?;
            if subject.record() == &record {
                return Ok(());
            }
        }
        if self.subjects.len() == self.subjects.element_count() {
            return Err(Denial::CaptureLimitExceeded);
        }
        self.visit(context)?;
        let subject = self
            .runtime
            .retain_record_history_subject(self.execution, self.basis, record)
            .map_err(Denial::Native)?;
        self.subjects.push(subject)?;
        Ok(())
    }

    fn visit(
        &mut self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        if self.visits == self.maximum_visits {
            return Err(Denial::CaptureLimitExceeded);
        }
        request_stage::checkpoint(context, 1)?;
        self.visits += 1;
        Ok(())
    }
}
