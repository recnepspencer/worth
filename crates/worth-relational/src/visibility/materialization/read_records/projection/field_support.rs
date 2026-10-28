//! Paired immutable-root support. This does not normalize revision currentness.
use super::VisibilityProjectionView;
use crate::execution::request_stage::{checkpoint, optional_stage};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::EntityId;
use worth_foundational::facade::{AspectFieldLocator, LocatorAuthority};

/// A foreign source, unsupported field, unavailable root/contract, or original
/// execution refusal prevents an exact paired field-support verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalFieldSupportComparisonDenial {
    ForeignOwner,
    UnsupportedLocator,
    SourceUnavailable,
    Execution(Cause),
}

impl VisibilityProjectionView {
    /// Compare the actual original/target field meaning, presence and full-GEN
    /// kind under their installed contracts. Both exact roots stay borrowed.
    /// This is not proof of cancellation, ancestry, current authority, or reuse.
    /// Work is the two declared-field probes, selected slot probes, contract
    /// lookup and compared value width; no record/value/scope is allocated.
    pub fn compare_entity_field_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &Self,
        entity: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<bool, RelationalFieldSupportComparisonDenial> {
        use RelationalFieldSupportComparisonDenial as Denial;
        if self.runtime_identity != target.runtime_identity {
            return Err(Denial::ForeignOwner);
        }
        if entity.generation_value() == 0
            || locator.aspect().authority() != LocatorAuthority::Authoritative
            || locator.field_path().fields().len() != 1
        {
            return Err(Denial::UnsupportedLocator);
        }
        optional_stage(execution, |context, _memory| {
            let mut compare = || -> Result<bool, Denial> {
                for view in [self, target] {
                    let revision = view.entity_field_revision_admitted(execution, entity, locator, |work| checkpoint(context, work))
                        .map_err(Denial::Execution)?.map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))?;
                    if revision.is_none() { return Err(Denial::SourceUnavailable); }
                    checkpoint(context, view.exact_entity_state_read_work_bound().ok_or(Denial::SourceUnavailable)?)
                        .map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))?;
                }
                self.read_exact_entity_state(execution, entity, |original, before| {
                    target.read_exact_entity_state(execution, entity, |current, after| {
                        if original != current { return Ok(false); }
                        if !same_installed_contract(self, target, original.kind_id, locator, context)? { return Ok(false); }
                        admit_field_lookup(before, locator, context)?;
                        admit_field_lookup(after, locator, context)?;
                        let left = super::query_locus_projection::authoritative_state_query_locus_value(before, locator);
                        let right = super::query_locus_projection::authoritative_state_query_locus_value(after, locator);
                        let work = left.map_or(1, |value| value.semantic_byte_width() as u64)
                            .checked_add(right.map_or(1, |value| value.semantic_byte_width() as u64))
                            .ok_or(Denial::Execution(Cause::WorkCounterOverflow))?;
                        checkpoint(context, work).map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))?;
                        Ok(left == right)
                    }).map_err(|cause| match cause {
                        super::RelationalBorrowedRecordReadDenial::Execution(cause) => Denial::Execution(cause),
                        super::RelationalBorrowedRecordReadDenial::ExactBasisRequired => Denial::SourceUnavailable,
                    })?.ok_or(Denial::SourceUnavailable)?
                }).map_err(|cause| match cause {
                        super::RelationalBorrowedRecordReadDenial::Execution(cause) => Denial::Execution(cause),
                        super::RelationalBorrowedRecordReadDenial::ExactBasisRequired => Denial::SourceUnavailable,
                    })?.ok_or(Denial::SourceUnavailable)?
            };
            compare()
        }, |_| 0).map_err(Denial::Execution)?
    }
}

fn same_installed_contract(
    original: &VisibilityProjectionView,
    target: &VisibilityProjectionView,
    kind: crate::identity::data::KindId,
    locator: &AspectFieldLocator,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<bool, RelationalFieldSupportComparisonDenial> {
    use RelationalFieldSupportComparisonDenial as Denial;
    let aspect = locator.aspect().aspect_key();
    let mut contract = |view: &VisibilityProjectionView| {
        let count = view
            .basis
            .root()
            .ok_or(Denial::SourceUnavailable)?
            .schema_authority()
            .entity_aspect_plan_count();
        checkpoint(
            context,
            super::field_revisions::btree_navigation_work(count),
        )
        .map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))?;
        let plan = view
            .entity_aspect_plan(kind)
            .ok_or(Denial::SourceUnavailable)?;
        for binding in &plan.executable_bindings {
            checkpoint(context, (aspect.as_str().len() as u64).saturating_add(1))
                .map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))?;
            if binding.aspect_key() == aspect {
                return Ok((binding.contract.identity(), binding.contract.revision()));
            }
        }
        Err(Denial::SourceUnavailable)
    };
    Ok(contract(original)? == contract(target)?)
}

// Pay both real aspect probes (presence here, then the sole value projection)
// and the optional struct-field descent before the borrowed lookup runs.
fn admit_field_lookup(
    state: Option<&worth_foundational::facade::AuthoritativeRecordAspectState>,
    locator: &AspectFieldLocator,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), RelationalFieldSupportComparisonDenial> {
    use worth_foundational::facade::ContractValidatedAspectValueView;
    use RelationalFieldSupportComparisonDenial as Denial;
    let Some(state) = state else {
        return Ok(());
    };
    let aspect = locator.aspect().aspect_key();
    let work = super::field_revisions::btree_navigation_work(state.aspects().len())
        .checked_mul(aspect.as_str().len() as u64 + 1)
        .and_then(|work| work.checked_mul(2))
        .ok_or(Denial::Execution(Cause::WorkCounterOverflow))?;
    checkpoint(context, work)
        .map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))?;
    let Some(value) = state.get(aspect) else {
        return Ok(());
    };
    if let ContractValidatedAspectValueView::Struct(value) = value.view() {
        let field = &locator.field_path().fields()[0];
        let work = super::field_revisions::btree_navigation_work(value.len())
            .checked_mul(field.as_str().len() as u64 + 1)
            .ok_or(Denial::Execution(Cause::WorkCounterOverflow))?;
        checkpoint(context, work)
            .map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))?;
    }
    Ok(())
}
