//! Scalar and borrowed relation aspect reads from the immutable pin.
use super::RelationalPinnedRead;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectKey, AspectValue, ContractValidatedAspectValue, FieldKey};
use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::{
    RelationalBorrowedRecordReadDenial, RelationalFieldSupportComparisonDenial,
};
use worth_relational_contracts::internal_port::identity::data::record_ids::RelationId;

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    pub fn read_relation_aspect_version(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        relation: RelationId,
        aspect: &AspectKey,
    ) -> Result<Option<Option<u64>>, RelationalBorrowedRecordReadDenial> {
        self.project_read()
            .read_relation_aspect_version(execution, relation, aspect)
    }

    pub fn read_relation_aspect<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        relation: RelationId,
        aspect: &AspectKey,
        project: impl FnOnce(Option<&ContractValidatedAspectValue>) -> T,
    ) -> Result<Option<T>, RelationalBorrowedRecordReadDenial> {
        self.project_read()
            .read_relation_aspect(execution, relation, aspect, project)
    }

    pub fn read_relation_aspect_field<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        relation: RelationId,
        aspect: &AspectKey,
        field: &FieldKey,
        project: impl FnOnce(Option<&AspectValue>) -> T,
    ) -> Result<Option<T>, RelationalBorrowedRecordReadDenial> {
        self.project_read()
            .read_relation_aspect_field(execution, relation, aspect, field, project)
    }

    /// Visit the paired roots' authored coordinates without copying field keys.
    pub fn relation_aspect_support_fields<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        target: &Self,
        relation: RelationId,
        aspect: &AspectKey,
        project: impl FnOnce(&mut dyn Iterator<Item = &FieldKey>) -> T,
    ) -> Result<T, RelationalFieldSupportComparisonDenial> {
        let source = self.project_read();
        let mut fields = source.relation_aspect_support_fields(
            execution,
            &target.project_read(),
            relation,
            aspect,
        )?;
        Ok(project(&mut fields))
    }

    pub fn relation_aspect_presence_scalar_field<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        target: &Self,
        relation: RelationId,
        aspect: &AspectKey,
        project: impl FnOnce(Option<&FieldKey>) -> T,
    ) -> Result<T, RelationalFieldSupportComparisonDenial> {
        let source = self.project_read();
        let field = source.relation_aspect_presence_scalar_field(
            execution,
            &target.project_read(),
            relation,
            aspect,
        )?;
        Ok(project(field))
    }
}
