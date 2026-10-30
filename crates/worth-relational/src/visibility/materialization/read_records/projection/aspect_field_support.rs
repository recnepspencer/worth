//! Complete field-set support at two immutable roots, not revision equality.
use super::{RelationalFieldSupportComparisonDenial as Denial, VisibilityProjectionView};
use crate::execution::request_stage::{checkpoint, optional_stage};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::EntityId;
use crate::schema::data::LoweredAspectContractBinding;
use worth_foundational::facade::{AspectBinding, AspectKey, AspectShape, FieldKey};

impl VisibilityProjectionView {
    /// Actual installed coordinates that can author this entire aspect.
    /// Lifecycle and WholeAspect must also be selected. No coordinate is copied,
    /// and this descriptive roster grants neither history nor cancellation.
    pub fn entity_aspect_support_fields<'a>(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &Self,
        entity: EntityId,
        aspect: &AspectKey,
    ) -> Result<impl Iterator<Item = &'a FieldKey>, Denial> {
        let binding = optional_stage(
            execution,
            |context, _| paired_binding(self, execution, target, entity, aspect, context),
            |_| 0,
        )
        .map_err(Denial::Execution)??;
        let AspectBinding::EntityField { field } = &binding.target else {
            return Err(Denial::UnsupportedLocator);
        };
        let (scalar, structure) = match binding.contract.shape() {
            AspectShape::Scalar(_) => (Some(field), None),
            AspectShape::Struct(shape) => (None, Some(shape)),
            _ => return Err(Denial::UnsupportedLocator),
        };
        Ok(scalar.into_iter().chain(
            structure
                .into_iter()
                .flat_map(|shape| shape.fields().iter().map(|declaration| declaration.key())),
        ))
    }

    /// Compare full aspect presence/content with compatible installed meaning.
    /// The caller still proves selected ancestry, cancellation and original fact
    /// validity; equal content alone never establishes restored eligibility.
    pub fn compare_entity_aspect_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &Self,
        entity: EntityId,
        aspect: &AspectKey,
    ) -> Result<bool, Denial> {
        optional_stage(
            execution,
            |context, _| {
                let binding = paired_binding(self, execution, target, entity, aspect, context)?;
                for view in [self, target] {
                    admit(
                        context,
                        view.exact_entity_state_read_work_bound()
                            .ok_or(Denial::SourceUnavailable)?,
                    )?;
                }
                self.read_exact_entity_state(execution, entity, |_, before| {
                    target
                        .read_exact_entity_state(execution, entity, |_, after| {
                            equal_aspect_values(before, after, aspect, &binding.contract, context)
                        })
                        .map_err(read_denial)?
                        .ok_or(Denial::SourceUnavailable)?
                })
                .map_err(read_denial)?
                .ok_or(Denial::SourceUnavailable)?
            },
            |_| 0,
        )
        .map_err(Denial::Execution)?
    }
}

fn equal_aspect_values(
    before: Option<&worth_foundational::facade::AuthoritativeRecordAspectState>,
    after: Option<&worth_foundational::facade::AuthoritativeRecordAspectState>,
    aspect: &AspectKey,
    contract: &worth_foundational::facade::AspectContract,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<bool, Denial> {
    for state in [before, after].into_iter().flatten() {
        let work = super::field_revisions::btree_navigation_work(state.aspects().len())
            .checked_mul(aspect.as_str().len() as u64 + 1)
            .ok_or(Denial::Execution(Cause::WorkCounterOverflow))?;
        admit(context, work)?;
    }
    let left = before.and_then(|state| state.get(aspect));
    let right = after.and_then(|state| state.get(aspect));
    let width = left
        .map_or(1, |value| value.semantic_byte_width() as u64)
        .checked_add(right.map_or(1, |value| value.semantic_byte_width() as u64))
        .ok_or(Denial::Execution(Cause::WorkCounterOverflow))?;
    admit(context, width)?;
    if [left, right]
        .into_iter()
        .flatten()
        .any(|value| value.contract() != contract)
    {
        return Err(Denial::SourceUnavailable);
    }
    Ok(left == right)
}

fn paired_binding<'a>(
    original: &'a VisibilityProjectionView,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: &VisibilityProjectionView,
    entity: EntityId,
    aspect: &AspectKey,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<&'a LoweredAspectContractBinding, Denial> {
    if original.runtime_identity != target.runtime_identity {
        return Err(Denial::ForeignOwner);
    }
    if entity.generation_value() == 0 {
        return Err(Denial::UnsupportedLocator);
    }
    let mut metadata = None;
    for view in [original, target] {
        admit(
            context,
            view.exact_entity_state_read_work_bound()
                .ok_or(Denial::SourceUnavailable)?,
        )?;
        let current = view
            .read_exact_entity_state(execution, entity, |metadata, _| metadata)
            .map_err(read_denial)?
            .ok_or(Denial::SourceUnavailable)?;
        if current.entity_id != entity
            || current.lifecycle != crate::storage::data::RecordLifecycleState::Live
            || metadata.is_some_and(|before| before != current)
        {
            return Err(Denial::SourceUnavailable);
        }
        metadata = Some(current);
    }
    let kind = metadata.ok_or(Denial::SourceUnavailable)?.kind_id;
    let before = binding(original, kind, aspect, context)?;
    let after = binding(target, kind, aspect, context)?;
    let width = (before.contract.semantic_byte_width() as u64)
        .checked_add(after.contract.semantic_byte_width() as u64)
        .ok_or(Denial::Execution(Cause::WorkCounterOverflow))?;
    admit(context, width)?;
    if before != after || !matches!(&before.target, AspectBinding::EntityField { .. }) {
        return Err(Denial::SourceUnavailable);
    }
    if !matches!(
        before.contract.shape(),
        AspectShape::Scalar(_) | AspectShape::Struct(_)
    ) {
        return Err(Denial::UnsupportedLocator);
    }
    Ok(before)
}

fn binding<'a>(
    view: &'a VisibilityProjectionView,
    kind: crate::identity::data::KindId,
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
        admit(context, aspect.as_str().len() as u64 + 1)?;
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

fn read_denial(cause: super::RelationalBorrowedRecordReadDenial) -> Denial {
    match cause {
        super::RelationalBorrowedRecordReadDenial::ExactBasisRequired => Denial::SourceUnavailable,
        super::RelationalBorrowedRecordReadDenial::Execution(cause) => Denial::Execution(cause),
    }
}
