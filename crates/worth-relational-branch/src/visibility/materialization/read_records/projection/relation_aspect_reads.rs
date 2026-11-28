//! Borrow a selected relation's aspect or field directly from its exact root.
use super::{RelationalBorrowedRecordReadDenial as Denial, VisibilityProjectionView};
use crate::execution::request_stage::checkpoint;
use crate::identity::data::{KindId, RelationId};
use crate::schema::data::LoweredAspectContractBinding;
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_foundational::facade::{
    AspectKey, AspectValue, ContractValidatedAspectValue, ContractValidatedAspectValueView,
    FieldKey,
};

impl VisibilityProjectionView {
    /// The outer option identifies the live relation. The callback receives
    /// an absent value when the exact relation has no payload for this aspect.
    /// Only the selected Head region and its symbols are admitted.
    pub fn read_relation_aspect<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        relation: RelationId,
        aspect: &AspectKey,
        project: impl FnOnce(Option<&ContractValidatedAspectValue>) -> T,
    ) -> Result<Option<T>, Denial> {
        self.with_relation_aspect(execution, relation, aspect, |_, value, _| {
            Ok(project(value))
        })
    }

    /// Borrow one authored scalar or struct field from the exact relation.
    pub fn read_relation_aspect_field<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        relation: RelationId,
        aspect: &AspectKey,
        field: &FieldKey,
        project: impl FnOnce(Option<&AspectValue>) -> T,
    ) -> Result<Option<T>, Denial> {
        self.with_relation_aspect(execution, relation, aspect, |kind, value, context| {
            let selected = match value.map(ContractValidatedAspectValue::view) {
                Some(ContractValidatedAspectValueView::Struct(fields)) => {
                    admit_lookup(context, fields.len(), field.as_str())?;
                    fields.get(field)
                }
                Some(ContractValidatedAspectValueView::Scalar(value)) => self
                    .relation_scalar_field_matches(context, kind, aspect, field)?
                    .then_some(value),
                None => None,
            };
            Ok(project(selected))
        })
    }

    fn with_relation_aspect<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        relation: RelationId,
        aspect: &AspectKey,
        project: impl FnOnce(
            KindId,
            Option<&ContractValidatedAspectValue>,
            &mut MapKernelContext<'_, '_>,
        ) -> Result<T, Denial>,
    ) -> Result<Option<T>, Denial> {
        let probe_work = self
            .exact_relation_state_read_work_bound()
            .ok_or(Denial::ExactBasisRequired)?;
        execution
            .run_native_with_checkpoint(|context| {
                admit(context, probe_work)?;
                self.read_exact_relation_state(execution, relation, |metadata, state| {
                    if metadata.relation_id != relation {
                        return Ok(None);
                    }
                    let value = if let Some(state) = state {
                        admit_lookup(context, state.aspects().len(), aspect.as_str())?;
                        state.get(aspect)
                    } else {
                        None
                    };
                    project(metadata.kind_id, value, context).map(Some)
                })?
                .transpose()
                .map(Option::flatten)
            })
            .map_err(|stop| Denial::Execution(stop.into()))?
            .0
    }

    fn relation_scalar_field_matches(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        kind: KindId,
        aspect: &AspectKey,
        field: &FieldKey,
    ) -> Result<bool, Denial> {
        let selected =
            self.lookup_relation_aspect_binding(kind, aspect, |work| admit(context, work))?;
        admit(context, 1 + field.as_str().len() as u64)?;
        Ok(selected.is_some_and(|binding| binding.targets_relation_scalar_field(field)))
    }

    pub(super) fn lookup_relation_aspect_binding<E>(
        &self,
        kind: KindId,
        aspect: &AspectKey,
        mut admit: impl FnMut(u64) -> Result<(), E>,
    ) -> Result<Option<&LoweredAspectContractBinding>, E> {
        let Some(root) = self.basis.root() else {
            return Ok(None);
        };
        admit(super::field_revisions::btree_navigation_work(
            root.schema_authority().relation_aspect_plan_count(),
        ))?;
        let Some(plan) = self.relation_aspect_plan(kind) else {
            return Ok(None);
        };
        for binding in &plan.executable_bindings {
            admit(1 + aspect.as_str().len() as u64)?;
            if binding.aspect_key() == aspect {
                return Ok(Some(binding));
            }
        }
        Ok(None)
    }
}

fn admit_lookup(
    context: &mut MapKernelContext<'_, '_>,
    entries: usize,
    key: &str,
) -> Result<(), Denial> {
    let units =
        super::field_revisions::btree_navigation_work(entries).saturating_mul(1 + key.len() as u64);
    admit(context, units)
}

fn admit(context: &mut MapKernelContext<'_, '_>, work: u64) -> Result<(), Denial> {
    checkpoint(context, work)
        .map_err(|stop| Denial::Execution(crate::execution::denial::kernel_stop(stop)))
}
