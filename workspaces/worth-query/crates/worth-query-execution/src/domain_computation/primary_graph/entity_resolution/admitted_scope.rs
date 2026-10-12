//! Selected retained-scope resolution through the admitted Native lookup.

use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupAdmissionStop, BoundedEntityFieldLookupDenialKind,
    BoundedIndexParityMode, DerivedIndexId,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::RelationalSnapshotProjectionAdmissionStop;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;

use super::{
    entity_denial, WorthQueryApplicationEntityIdentity, WorthQueryEntityResolutionDenial,
    WorthQueryEntityResolutionDenialKind as Kind, WorthQueryEntityResolutionSubject,
    WorthQueryPrincipalResolutionMode, WorthQueryResolvedEntity,
};
use crate::domain_computation::primary_graph::{
    index_currency::selected_index_is_current_admitted,
    output_lineage::invalidation::InvalidationEditAdmission, WorthQuerySelectedProductOperation,
};
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    {
        // Context retention copies its Arc and 80-byte schema binding identity.
        admission
            .charge_external_work(82)
            .map_err(|stop| admission_denial(stop, field))?;
        let application = self.application();
        let graph = application.runtime.primary_graph().ok_or_else(|| {
            entity_denial(
                Kind::Refused(ResolutionRefusal::PrimaryGraphNotInstalled),
                entity,
            )
        })?;
        let installed = graph.retain_entity_resolution_context();
        let lookup = graph.with_runtime_mut(execution, |relational| {
            selected_index_is_current_admitted(
                relational,
                self.product().relational_basis(),
                index,
                admission,
            )
            .map_err(|stop| admission_denial(stop, field))?
            .then_some(())
            .ok_or_else(|| {
                entity_denial(
                    Kind::Refused(ResolutionRefusal::EqualityIndexUnavailable),
                    field,
                )
            })?;
            let view = relational
                .read_truth()
                .project_snapshot_admitted(
                    self.application_basis().snapshot_handle(),
                    execution,
                    |work, bytes| {
                        admission.admit_read_scratch(bytes)?;
                        admission.charge_external_work(work)
                    },
                )
                .map_err(|stop| match stop {
                    RelationalSnapshotProjectionAdmissionStop::Execution(cause) => entity_denial(
                        native_denial_kind(BoundedEntityFieldLookupDenialKind::Execution(cause)),
                        field,
                    ),
                    RelationalSnapshotProjectionAdmissionStop::Admission(stop) => {
                        admission_denial(stop, field)
                    }
                    RelationalSnapshotProjectionAdmissionStop::AccountingOverflow => {
                        entity_denial(Kind::Stopped(NativeReadStop::WorkCounterOverflow), field)
                    }
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
                        |work, bytes| admission.charge_selected_index_read(work, bytes),
                    ),
                    BoundedIndexParityMode::Production,
                    execution,
                )
                .map_err(|stop| match stop {
                    BoundedEntityFieldLookupAdmissionStop::Lookup(denial) => {
                        entity_denial(native_denial_kind(denial.kind()), field)
                    }
                    BoundedEntityFieldLookupAdmissionStop::Admission(stop) => {
                        admission_denial(stop, field)
                    }
                    BoundedEntityFieldLookupAdmissionStop::AccountingOverflow => {
                        entity_denial(Kind::Stopped(NativeReadStop::WorkCounterOverflow), field)
                    }
                    BoundedEntityFieldLookupAdmissionStop::ExactBasisRequired => entity_denial(
                        Kind::Refused(ResolutionRefusal::ForeignResolutionTruth),
                        field,
                    ),
                })
        })?;
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
        admission
            .charge_external_work(80)
            .map_err(|stop| admission_denial(stop, field))?;
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

fn admission_denial(
    stop: CompanionPreflightStop,
    subject: &str,
) -> WorthQueryEntityResolutionDenial {
    super::preflight::denial(stop, subject)
}

fn native_denial_kind(kind: BoundedEntityFieldLookupDenialKind) -> Kind {
    super::map_index_denial(kind, "selected entity lookup").kind()
}
