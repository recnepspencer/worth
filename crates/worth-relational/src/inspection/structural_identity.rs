use crate::inspection::data::{
    InspectionDegradation, InspectionRecordClass, InspectionScope, StructuralIdentityComparison,
    StructuralIdentityComparisonVerdict, StructuralIdentityEvidence,
    StructuralIdentityQueryRequest,
};
use crate::transactions::data::RecordRef;

use super::access::{InspectionAccess, PartitionScopeFilter};

impl<'runtime> InspectionAccess<'runtime> {
    pub fn structural_identity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: InspectionScope,
        target: RecordRef,
    ) -> Result<Option<StructuralIdentityEvidence>, crate::execution::RelationalExecutionDenialCause>
    {
        self.count_structural_identity_lookup();
        let version_id = self.scope_version_id(&scope);
        Ok(match target {
            RecordRef::Entity(entity_id) => {
                let Some(read) =
                    self.scoped_authoritative_entity_record(execution, &scope, entity_id)?
                else {
                    return Ok(None);
                };
                let (lineage_id, structural_fingerprint) =
                    self.entity_structural_sidecars(entity_id);
                let mut degradations = Vec::new();
                if structural_fingerprint.is_none() {
                    degradations.push(InspectionDegradation::MissingStructuralFingerprint);
                }
                if lineage_id.is_none() {
                    degradations.push(InspectionDegradation::MissingLineageIdentity);
                }
                Some(StructuralIdentityEvidence {
                    target,
                    record_class: InspectionRecordClass::Entity,
                    kind_id: read.kind.kind_id,
                    storage_identity: RecordRef::Entity(entity_id),
                    lineage_id,
                    structural_fingerprint,
                    observed_version: version_id,
                    lifecycle: read.lifecycle,
                    origin: self.scope_origin(&scope),
                    access_path: self.scope_access_path(&scope, version_id),
                    availability: self.scope_availability(&scope, version_id),
                    degradations,
                })
            }
            RecordRef::Relation(relation_id) => {
                let Some(read) =
                    self.scoped_authoritative_relation_record(execution, &scope, relation_id)?
                else {
                    return Ok(None);
                };
                Some(StructuralIdentityEvidence {
                    target,
                    record_class: InspectionRecordClass::Relation,
                    kind_id: read.kind.kind_id,
                    storage_identity: RecordRef::Relation(relation_id),
                    lineage_id: None,
                    structural_fingerprint: None,
                    observed_version: version_id,
                    lifecycle: read.lifecycle,
                    origin: self.scope_origin(&scope),
                    access_path: self.scope_access_path(&scope, version_id),
                    availability: self.scope_availability(&scope, version_id),
                    degradations: vec![
                        InspectionDegradation::MissingStructuralFingerprint,
                        InspectionDegradation::MissingLineageIdentity,
                    ],
                })
            }
        })
    }

    pub fn compare_structural_identity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: InspectionScope,
        left: RecordRef,
        right: RecordRef,
    ) -> Result<StructuralIdentityComparison, crate::execution::RelationalExecutionDenialCause>
    {
        let left_evidence = self.structural_identity(execution, scope.clone(), left)?;
        let right_evidence = self.structural_identity(execution, scope, right)?;
        let verdict = match (&left_evidence, &right_evidence) {
            (Some(left), Some(right)) => {
                match (left.structural_fingerprint, right.structural_fingerprint) {
                    (Some(left), Some(right))
                        if left.family == right.family && left.value == right.value =>
                    {
                        StructuralIdentityComparisonVerdict::EqualByFingerprint
                    }
                    (Some(left), Some(right)) if left.family == right.family => {
                        StructuralIdentityComparisonVerdict::NotEqualByFingerprint
                    }
                    (Some(_), Some(_)) => {
                        StructuralIdentityComparisonVerdict::IncomparableFingerprintFamilyMismatch
                    }
                    _ => StructuralIdentityComparisonVerdict::IncomparableMissingFingerprint,
                }
            }
            _ => StructuralIdentityComparisonVerdict::IncomparableMissingFingerprint,
        };
        Ok(StructuralIdentityComparison {
            left: left_evidence,
            right: right_evidence,
            verdict,
        })
    }

    pub fn query_structural_identity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &StructuralIdentityQueryRequest,
    ) -> Result<Vec<StructuralIdentityEvidence>, crate::execution::RelationalExecutionDenialCause>
    {
        self.count_structural_identity_query_scan();
        let version_id = self.scope_version_id(&request.scope);
        let Some(read_view) = self.read_view_for_scope(execution, &request.scope)? else {
            return Ok(Vec::new());
        };
        let partition_scope = PartitionScopeFilter::from_scope(request.partition_scope.as_ref());
        let mut evidence = Vec::new();
        for record in read_view
            .entities()
            .iter()
            .filter(|record| partition_scope.allows(record.entity_id.partition_id))
        {
            if let Some(item) = self.structural_identity(
                execution,
                InspectionScope::Version(version_id),
                RecordRef::Entity(record.entity_id),
            )? && item
                .structural_fingerprint
                .is_some_and(|fingerprint| fingerprint.family == request.fingerprint_family)
            {
                evidence.push(item);
            }
        }
        Ok(evidence)
    }
}
