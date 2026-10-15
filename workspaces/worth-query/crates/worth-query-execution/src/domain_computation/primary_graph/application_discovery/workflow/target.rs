//! Exact-data locator shared by original-intent discovery and indexed enumeration.
use super::{
    admission::{admit, WorthQueryWorkflowReadDenial as Denial},
    original_intent::OriginalWorkflowCandidate,
    view::WorthQueryWorkflowReadBasis,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        observe_adjacency, observe_field_value_borrowed, WorthQueryApplicationAdjacencyDirection,
    },
    output_lineage::invalidation::InvalidationEditAdmission,
    WorthQueryAdvancementPhase, WorthQuerySelectedProductOperation,
};
use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowReadBinding, ApplicationWorkflowSpec,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledWorkflowReadBinding};
use worth_relational::facade::identity::EntityId;
/// Opaque non-serializable locator, never a disclosure permit or a command authority.
/// ```
/// use worth_query_execution::facade::application_discovery::WorthQueryWorkflowReadTarget;
/// fn inspect(target:&WorthQueryWorkflowReadTarget) {let _=target.basis().selected_commit();}
/// ```
/// ```compile_fail,E0616
/// use worth_query_execution::facade::application_discovery::WorthQueryWorkflowReadTarget;
/// fn no_raw_parts(target:&WorthQueryWorkflowReadTarget) {let _=target.instance;}
/// ```
/// ```compile_fail,E0599
/// use worth_query_execution::facade::application_discovery::WorthQueryWorkflowReadTarget;
/// fn no_authority(target: WorthQueryWorkflowReadTarget) { target.advance(); }
/// ```
#[derive(Clone)]
pub struct WorthQueryWorkflowReadTarget {
    pub(super) basis: WorthQueryWorkflowReadBasis,
    pub(super) instance: EntityId,
    pub(super) lineage: EntityId,
    pub(super) subject: EntityId,
    pub(super) dependencies: std::sync::Arc<worth_execution::ExecutionArray<EntityId>>,
}
impl WorthQueryWorkflowReadTarget {
    pub(super) fn observed<S: ApplicationSchema>(
        data: &WorthQuerySelectedProductOperation<'_, S>,
        instance: EntityId,
        lineage: EntityId,
        subject: EntityId,
        dependencies: worth_execution::ExecutionArray<EntityId>,
    ) -> Self {
        Self {
            basis: WorthQueryWorkflowReadBasis::retained(data),
            instance,
            lineage,
            subject,
            dependencies: std::sync::Arc::new(dependencies),
        }
    }
    pub fn basis(&self) -> &WorthQueryWorkflowReadBasis {
        &self.basis
    }
    pub(super) fn admit<'d, 'r, S, W, B>(
        &self,
        current: &WorthQuerySelectedProductOperation<'_, S>,
        data: &'d WorthQuerySelectedProductOperation<'r, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledWorkflowReadBinding<'_, S, W, B>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        input: B::Input,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        (
            OriginalWorkflowCandidate<'d, 'r, S>,
            crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
                S,
                B::Capability,
                B::Operation,
                B::Input,
            >,
        ),
        Denial,
    >
    where
        S: ApplicationSchema,
        W: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationWorkflowReadBinding<S>,
    {
        // Fresh security precedes even locator affinity and existence-dependent reads.
        let access = admit(
            current, phase, binding, principal, input, request, admission,
        )?;
        if !self
            .basis
            .0
            .belongs_to_selected_occurrence(current.application(), data.product().observation())
            || !std::ptr::eq(current.application(), data.application())
            || current.product().branch_identity() != data.product().branch_identity()
            || self.basis.selected_commit() != data.product().selected_commit()
            || self.basis.branch_identity() != data.product().branch_identity()
            || access.resource_entity_id() != self.subject
        {
            return Err(Denial::PolicyMismatch);
        }
        let execution = phase
            .execution_request_for(&current.application().product_runtime)
            .map_err(Denial::Phase)?;
        let graph = current
            .application()
            .runtime
            .primary_graph()
            .ok_or(Denial::BasisUnavailable)?;
        graph
            .integration_handle()
            .with_runtime(&execution, |runtime| {
                let layout = graph.layout().workflow();
                let snapshot = data.application_basis().snapshot_handle();
                for (field, value) in [
                    (
                        &layout.instance.subject_partition,
                        self.subject.partition_value_u64(),
                    ),
                    (
                        &layout.instance.subject_slot,
                        self.subject.local_slot_value(),
                    ),
                    (
                        &layout.instance.subject_generation,
                        u64::from(self.subject.generation_value()),
                    ),
                ] {
                    if observe_field_value_borrowed(
                        runtime,
                        &execution,
                        snapshot,
                        self.instance,
                        layout.instance.entity_kind,
                        field,
                        |v| v == Some(&AspectValue::UInt64(value)),
                    )
                    .map_err(Denial::Relational)?
                        != Some(true)
                    {
                        return Err(Denial::Corrupt);
                    }
                }
                let relations = observe_adjacency(
                    runtime,
                    &execution,
                    snapshot,
                    layout.instance_lineage_relation,
                    self.instance,
                    WorthQueryApplicationAdjacencyDirection::Outgoing,
                    2,
                )
                .map_err(Denial::Execution)?
                .ok_or(Denial::BasisUnavailable)?;
                if !matches!(relations.as_slice(),[one] if one.to==self.lineage) {
                    return Err(Denial::Corrupt);
                }
                let pin = runtime
                    .read_truth()
                    .pin_read_capability(snapshot, &[], &execution)
                    .map_err(Denial::PinnedRead)?;
                if !super::text::inspect(
                    runtime,
                    &execution,
                    &pin,
                    self.lineage,
                    layout.lineage.entity_kind,
                    &layout.lineage.spec,
                    |text| text == Some(W::IDENTITY.as_str()),
                )? {
                    return Err(Denial::PolicyMismatch);
                }
                Ok(())
            })?;
        Ok((
            OriginalWorkflowCandidate {
                data,
                instance: self.instance,
                lineage: self.lineage,
                subject: self.subject,
            },
            access,
        ))
    }
}
