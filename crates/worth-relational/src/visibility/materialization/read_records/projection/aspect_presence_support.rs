//! Resolve presence-changing authored field coordinates from paired exact roots.
use super::VisibilityProjectionView;
use crate::execution::request_stage::{checkpoint, optional_stage};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::{EntityId, KindId};
use crate::schema::data::LoweredAspectContractBinding;
use worth_foundational::facade::{AspectBinding, AspectKey, AspectShape, FieldKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Why paired exact roots cannot supply the presence-changing coordinate.
pub enum RelationalAspectPresenceSupportDenial {
    ForeignOwner,
    EntityMismatch,
    SourceUnavailable,
    IncompatibleBinding,
    Execution(Cause),
}

use RelationalAspectPresenceSupportDenial as Denial;

impl VisibilityProjectionView {
    /// Borrow the scalar EntityField that can initialize this aspect through
    /// UpdateFields lowering. Struct PatchFields preserves its container, so
    /// it needs no field coordinate in addition to Lifecycle/WholeAspect.
    /// Both roots must contain the exact full-GEN entity/kind and the same
    /// installed contract, shape and binding. This establishes no cancellation
    /// or presence equality; callers must still validate their observed facts.
    /// Work follows two slot/kind probes, installed binding lookup and contract
    /// comparison width. No record, value or coordinate is copied.
    pub fn entity_aspect_presence_scalar_field<'a>(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &Self,
        entity: EntityId,
        kind: KindId,
        aspect: &AspectKey,
    ) -> Result<Option<&'a FieldKey>, Denial> {
        if self.runtime_identity != target.runtime_identity {
            return Err(Denial::ForeignOwner);
        }
        if entity.generation_value() == 0 {
            return Err(Denial::EntityMismatch);
        }
        optional_stage(
            execution,
            |context, _memory| {
                for view in [self, target] {
                    admit(
                        context,
                        view.exact_entity_state_read_work_bound()
                            .ok_or(Denial::SourceUnavailable)?,
                    )?;
                    let metadata = view
                        .read_exact_entity_state(execution, entity, |metadata, _| metadata)
                        .map_err(|cause| match cause {
                            super::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
                                Denial::SourceUnavailable
                            }
                            super::RelationalBorrowedRecordReadDenial::Execution(cause) => {
                                Denial::Execution(cause)
                            }
                        })?
                        .ok_or(Denial::SourceUnavailable)?;
                    if metadata.entity_id != entity
                        || metadata.kind_id != kind
                        || metadata.lifecycle != crate::storage::data::RecordLifecycleState::Live
                    {
                        return Err(Denial::EntityMismatch);
                    }
                }
                let before = binding(self, kind, aspect, context)?;
                let after = binding(target, kind, aspect, context)?;
                let (
                    AspectBinding::EntityField { field },
                    AspectBinding::EntityField { field: current },
                ) = (&before.target, &after.target)
                else {
                    return Err(Denial::IncompatibleBinding);
                };
                let work = (before.contract.semantic_byte_width() as u64)
                    .checked_add(after.contract.semantic_byte_width() as u64)
                    .and_then(|work| work.checked_add(field.as_str().len() as u64))
                    .ok_or(Denial::Execution(Cause::WorkCounterOverflow))?;
                admit(context, work)?;
                if field != current || before.contract != after.contract {
                    return Err(Denial::IncompatibleBinding);
                }
                match before.contract.shape() {
                    AspectShape::Scalar(_) => Ok(Some(field)),
                    AspectShape::Struct(_) => Ok(None),
                    _ => Err(Denial::IncompatibleBinding),
                }
            },
            |_| 0,
        )
        .map_err(Denial::Execution)?
    }
}

fn binding<'a>(
    view: &'a VisibilityProjectionView,
    kind: KindId,
    aspect: &AspectKey,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<&'a LoweredAspectContractBinding, Denial> {
    let root = view.basis.root().ok_or(Denial::SourceUnavailable)?;
    admit(
        context,
        super::field_revisions::btree_navigation_work(
            root.schema_authority().entity_aspect_plan_count(),
        ),
    )?;
    let plan = view
        .entity_aspect_plan(kind)
        .ok_or(Denial::SourceUnavailable)?;
    for binding in &plan.executable_bindings {
        admit(context, (aspect.as_str().len() as u64).saturating_add(1))?;
        if binding.aspect_key() == aspect {
            return Ok(binding);
        }
    }
    Err(Denial::SourceUnavailable)
}

fn admit(context: &mut worth_execution::MapKernelContext<'_, '_>, work: u64) -> Result<(), Denial> {
    checkpoint(context, work)
        .map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))
}
