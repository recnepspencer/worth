//! Authored relation field coordinates shared by two exact immutable roots.
use super::{RelationalFieldSupportComparisonDenial as Denial, VisibilityProjectionView};
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::execution::request_stage::{checkpoint, optional_stage};
use crate::identity::data::RelationId;
use crate::schema::data::LoweredAspectContractBinding;
use crate::storage::data::RecordLifecycleState;
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_foundational::facade::{AspectBinding, AspectKey, AspectShape, FieldKey};

impl VisibilityProjectionView {
    /// Authored coordinates of the installed scalar or struct relation aspect.
    /// Both roots must hold the same live relation, kind, and aspect contract.
    pub fn relation_aspect_support_fields<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        target: &Self,
        relation: RelationId,
        aspect: &AspectKey,
    ) -> Result<impl Iterator<Item = &'a FieldKey> + use<'a>, Denial> {
        let binding = self.paired_relation_aspect_binding(execution, target, relation, aspect)?;
        let (scalar, structure) = match (&binding.target, binding.contract.shape()) {
            (AspectBinding::RelationField { field }, AspectShape::Scalar(_)) => (Some(field), None),
            (AspectBinding::RelationField { .. }, AspectShape::Struct(shape)) => {
                (None, Some(shape))
            }
            _ => return Err(Denial::UnsupportedLocator),
        };
        Ok(scalar.into_iter().chain(
            structure
                .into_iter()
                .flat_map(|shape| shape.fields().iter().map(|field| field.key())),
        ))
    }

    /// Scalar field that may initialize presence. Struct field patches preserve
    /// presence, so they need no additional presence-changing coordinate.
    pub fn relation_aspect_presence_scalar_field<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        target: &Self,
        relation: RelationId,
        aspect: &AspectKey,
    ) -> Result<Option<&'a FieldKey>, Denial> {
        let binding = self.paired_relation_aspect_binding(execution, target, relation, aspect)?;
        match (&binding.target, binding.contract.shape()) {
            (AspectBinding::RelationField { field }, AspectShape::Scalar(_)) => Ok(Some(field)),
            (AspectBinding::RelationField { .. }, AspectShape::Struct(_)) => Ok(None),
            _ => Err(Denial::UnsupportedLocator),
        }
    }

    fn paired_relation_aspect_binding<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        target: &Self,
        relation: RelationId,
        aspect: &AspectKey,
    ) -> Result<&'a LoweredAspectContractBinding, Denial> {
        if self.runtime_identity != target.runtime_identity {
            return Err(Denial::ForeignOwner);
        }
        if relation.generation.is_zero() {
            return Err(Denial::UnsupportedLocator);
        }
        optional_stage(
            execution,
            |context, _| {
                let mut kind = None;
                for view in [self, target] {
                    admit(
                        context,
                        view.exact_relation_metadata_read_work_bound()
                            .ok_or(Denial::SourceUnavailable)?,
                    )?;
                    let current = view
                        .read_exact_relation_metadata(execution, relation)
                        .map_err(|denial| match denial {
                            super::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
                                Denial::SourceUnavailable
                            }
                            super::RelationalBorrowedRecordReadDenial::Execution(cause) => {
                                Denial::Execution(cause)
                            }
                        })?
                        .ok_or(Denial::SourceUnavailable)?;
                    if current.relation_id != relation
                        || current.lifecycle != RecordLifecycleState::Live
                        || kind.is_some_and(|before| before != current.kind_id)
                    {
                        return Err(Denial::SourceUnavailable);
                    }
                    kind = Some(current.kind_id);
                }
                let kind = kind.ok_or(Denial::SourceUnavailable)?;
                let before = self
                    .lookup_relation_aspect_binding(kind, aspect, |work| admit(context, work))?
                    .ok_or(Denial::SourceUnavailable)?;
                let after = target
                    .lookup_relation_aspect_binding(kind, aspect, |work| admit(context, work))?
                    .ok_or(Denial::SourceUnavailable)?;
                admit(
                    context,
                    (before.contract.semantic_byte_width() as u64)
                        .checked_add(after.contract.semantic_byte_width() as u64)
                        .and_then(|work| {
                            work.checked_add(before.target.clone_allocation_bytes() as u64)
                        })
                        .ok_or(Denial::Execution(Cause::WorkCounterOverflow))?,
                )?;
                if before != after {
                    return Err(Denial::SourceUnavailable);
                }
                Ok(before)
            },
            |_| 0,
        )
        .map_err(Denial::Execution)?
    }
}

fn admit(context: &mut MapKernelContext<'_, '_>, work: u64) -> Result<(), Denial> {
    checkpoint(context, work)
        .map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))
}
