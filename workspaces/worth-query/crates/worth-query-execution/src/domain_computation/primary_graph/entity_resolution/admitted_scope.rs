//! Selected retained-scope resolution through the admitted Native lookup.

use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;
use worth_relational::facade::indexes::BoundedIndexParityMode;
use worth_relational::facade::indexes::DerivedIndexId;

use super::entity_denial;
use super::WorthQueryApplicationEntityIdentity;
use super::WorthQueryEntityResolutionDenial;
use super::WorthQueryEntityResolutionDenialKind as Kind;
use super::WorthQueryEntityResolutionSubject;
use super::WorthQueryPrincipalResolutionMode;
use super::WorthQueryResolvedEntity;
use crate::domain_computation::primary_graph::index_currency::selected_index_is_current;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// The installed selector has already been checked and its cloning
    /// admitted by resolve_retained_query_scope. This child owns the actual
    /// selected-index, snapshot and bounded Native lookup Work.
    pub(super) fn resolve_encoded_entity_admitted<Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: &str,
        field: &str,
        encoded: AspectValue,
        request: &WorthQueryRequestScope,
        index: DerivedIndexId,
        layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryFieldLayout,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    {
        self.resolve_encoded_entity_admitted_with_native_access(execution, entity, field, encoded, request, index, layout, &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire())
    }

    pub(super) fn resolve_encoded_entity_admitted_with_native_access<Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: &str,
        field: &str,
        encoded: AspectValue,
        request: &WorthQueryRequestScope,
        index: DerivedIndexId,
        layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryFieldLayout,

        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    {
        // Context retention copies its Arc and 80-byte schema binding identity.

        let application = self.application();
        let graph = application.runtime.primary_graph().ok_or_else(|| {
            entity_denial(
                Kind::Refused(ResolutionRefusal::PrimaryGraphNotInstalled),
                entity,
            )
        })?;
        let installed = graph.retain_entity_resolution_context();
        let lookup = native.with_runtime_mut(
            graph,
            execution,
            || {
                entity_denial(
                    Kind::Refused(ResolutionRefusal::ForeignResolutionTruth),
                    field,
                )
            },
            |relational| {
                selected_index_is_current(relational, self.product().relational_basis(), index)
                    .then_some(())
                    .ok_or_else(|| {
                        entity_denial(
                            Kind::Refused(ResolutionRefusal::EqualityIndexUnavailable),
                            field,
                        )
                    })?;
                let view = relational
                    .read_truth()
                    .project_snapshot_in_request(
                        self.application_basis().snapshot_handle(),
                        execution,
                    )
                    .map_err(|cause| {
                        entity_denial(
                            native_denial_kind(BoundedEntityFieldLookupDenialKind::Execution(
                                cause,
                            )),
                            field,
                        )
                    })?
                    .ok_or_else(|| {
                        entity_denial(
                            Kind::Refused(ResolutionRefusal::ForeignResolutionTruth),
                            field,
                        )
                    })?;
                relational
                    .index_access()
                    .execute_bounded_entity_field_lookup(
                        worth_relational::facade::indexes::BoundedEntityFieldSelectedLookup::new(
                            &view,
                            index,
                            layout.entity_kind,
                            &layout.locator,
                            &encoded,
                            2,
                        ),
                        BoundedIndexParityMode::Production,
                        execution,
                    )
                    .map_err(|denial| entity_denial(native_denial_kind(denial.kind()), field))
            },
        )?;
        if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
            return Err(entity_denial(
                Kind::Refused(ResolutionRefusal::AmbiguousEntity),
                field,
            ));
        }
        lookup
            .candidate_entity_ids()
            .first()
            .ok_or_else(|| entity_denial(Kind::Refused(ResolutionRefusal::UnknownEntity), field))?;

        let resolved = WorthQueryResolvedEntity::from_lookup(
            &installed,
            layout,
            WorthQueryEntityResolutionSubject::new(
                entity,
                encoded,
                WorthQueryPrincipalResolutionMode::Ordinary,
            ),
            &lookup,
        );
        super::admit_request(request, field)?;
        Ok(resolved.into_application_identity())
    }
}

fn native_denial_kind(kind: BoundedEntityFieldLookupDenialKind) -> Kind {
    super::map_index_denial(kind, "selected entity lookup").kind()
}
