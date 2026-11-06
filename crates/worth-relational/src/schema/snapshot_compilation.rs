//! One canonical descriptor assembly under configuration or the original request.
use super::{data::*, SchemaCompilationControl};
use crate::execution::RelationalExecutionDenialCause as Denial;
mod recipes;

impl RelationalSchemaRegistry {
    pub fn authority_snapshot(&self) -> SchemaAuthoritySnapshot {
        self.snapshot_for_compilation(SchemaCompilationControl::Configuration)
            .expect("configuration descriptor assembly has no request refusal")
    }

    pub(crate) fn snapshot_for_compilation(
        &self,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<SchemaAuthoritySnapshot, Denial> {
        let mut basis = self
            .entity_kinds
            .values()
            .map(|kind| (&kind.schema_id, kind.schema_version_id))
            .chain(
                self.relation_kinds
                    .values()
                    .map(|kind| (&kind.schema_id, kind.schema_version_id)),
            );
        let first = basis.next();
        let mut consistent = true;
        for candidate in basis {
            control.checkpoint()?;
            consistent &= Some(candidate) == first;
        }
        let (primary_schema_id, primary_schema_version_id) = match first.filter(|_| consistent) {
            Some((id, version)) => (Some(id.clone()), Some(version)),
            None => (None, None),
        };
        let mut entity_kinds = Vec::with_capacity(self.entity_kinds.len());
        for (kind_id, registration) in &self.entity_kinds {
            control.checkpoint()?;
            entity_kinds.push(SchemaAuthorityKindSnapshot {
                kind_id: *kind_id,
                kind_name: registration.kind_name.clone(),
                schema_id: registration.schema_id.clone(),
                schema_version_id: registration.schema_version_id,
                aspect_plan_revision: registration.aspect_contract_declarations.plan_revision,
            });
        }
        let mut relation_kinds = Vec::with_capacity(self.relation_kinds.len());
        for (kind_id, registration) in &self.relation_kinds {
            control.checkpoint()?;
            let declarations = &registration.relation_integrity;
            let relation_integrity_plan_revision =
                derive_relation_integrity_plan_revision_for_compilation(
                    control,
                    &declarations.endpoint_kind_contracts,
                    &declarations.cardinality_contracts,
                    &declarations.uniqueness_contracts,
                    &declarations.symmetry_contracts,
                    &declarations.endpoint_deletion_integrity_contracts,
                    &declarations.acyclicity_contracts,
                    &declarations.partition_isolation_contracts,
                    &declarations.connectivity_minimum_contracts,
                    declarations.aggregate_identity.as_ref(),
                )?;
            relation_kinds.push(SchemaAuthorityRelationSnapshot {
                kind_id: *kind_id,
                kind_name: registration.kind_name.clone(),
                schema_id: registration.schema_id.clone(),
                schema_version_id: registration.schema_version_id,
                aspect_plan_revision: registration.aspect_contract_declarations.plan_revision,
                relation_integrity_plan_revision,
                cross_context_policy: registration.cross_context_policy,
                cascade_delete_policy: registration.cascade_delete_policy,
            });
        }
        let mut derived_relations = Vec::with_capacity(self.derived_relations.len());
        for definition in self.derived_relations.values() {
            control.checkpoint()?;
            derived_relations.push(recipes::definition(&definition.parts, control)?);
        }
        let mut projected_member_domains = Vec::with_capacity(self.projected_member_domains.len());
        for domain in self.projected_member_domains.values() {
            control.checkpoint()?;
            projected_member_domains.push(recipes::domain(&domain.parts, control)?);
        }
        control.checkpoint()?;
        Ok(SchemaAuthoritySnapshot {
            content_formats: self.content_formats.snapshot(),
            primary_schema_id,
            primary_schema_version_id,
            entity_kinds,
            relation_kinds,
            derived_relations,
            projected_member_domains,
        })
    }
}
