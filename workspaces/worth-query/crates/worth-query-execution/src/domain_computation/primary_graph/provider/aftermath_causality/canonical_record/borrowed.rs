//! Compare the existing canonical grammar without owned row/receipt decoding.
use super::super::{lineage_fields, source_fields, WorthQueryAftermathCausalityLayout};
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCommittedAftermathCausality,
    WorthQueryRecordedCorrectionSource,
};
use crate::domain_computation::primary_graph::provider::correction_witness::{
    admit, run, CorrectionWitnessDenial as Denial,
};
use worth_execution::{ExecutionRequest, MapKernelContext, MapKernelStop};
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, AuthoritativeRecordAspectState,
    ContractValidatedAspectValueView, InternedString,
};
use worth_relational::facade::{
    runtime::RelationalRuntime, snapshots::SnapshotHandle, storage::RecordLifecycleState,
    transactions::RecordRef,
};

pub(in crate::domain_computation::primary_graph::provider::aftermath_causality) fn matches_reverse(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryAftermathCausalityLayout,
    execution: ExecutionRequest<'_, '_>,
    sealed: &WorthQueryCommittedAftermathCausality,
) -> Result<bool, Denial> {
    let RecordRef::Entity(entity) = sealed.fact_record() else {
        return Ok(false);
    };
    if entity.generation_value() == 0
        || sealed.role() != WorthQueryAftermathCausalRole::RecordedReverse
    {
        return Ok(false);
    }
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or(Denial::Unavailable)?;
    run(execution, |context| {
        admit(
            context,
            view.exact_entity_state_read_work_bound()
                .ok_or(Denial::Unavailable)?,
        )?;
        view.read_exact_entity_state(&execution, *entity, |metadata, state| {
            if metadata.entity_id != *entity
                || metadata.kind_id != layout.entity_kind
                || metadata.lifecycle != RecordLifecycleState::Live
            {
                return Ok(false);
            }
            let mut fields = Fields {
                state: state.ok_or(Denial::Unavailable)?,
                context,
            };
            let lineage = sealed.correction_lineage();
            Ok(fields.number(&layout.role_locator, sealed.role().code())?
                && fields.string(&layout.parent_branch_locator, &sealed.parent().branch_id.0)?
                && fields.number(&layout.parent_commit_locator, sealed.parent().commit_id.0)?
                && fields.coordinates(
                    source_fields::locators(layout),
                    Some(sealed.recorded_source()),
                )?
                && fields.coordinates(layout.lineage.root.each_ref(), Some(lineage.root()))?
                && fields.coordinates(layout.lineage.selected.each_ref(), None)?
                && fields.coordinates(layout.lineage.active.each_ref(), None)?
                && fields
                    .coordinates(layout.lineage.prior.each_ref(), lineage.prior_transition())?
                && fields.selector(&layout.lineage.reversed_root, lineage.root())?
                && fields.selector(&layout.lineage.reversed_source, sealed.recorded_source())?
                && fields.value(&layout.lineage.consumed_reversal)?.is_none())
        })
        .map_err(|cause| match cause {
            worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired => Denial::Unavailable,
            worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => Denial::NativeRead(cause),
        })?
        .ok_or(Denial::Unavailable)?
    })
}

struct Fields<'state, 'context, 'kernel, 'authority> {
    state: &'state AuthoritativeRecordAspectState,
    context: &'context mut MapKernelContext<'kernel, 'authority>,
}
impl<'state> Fields<'state, '_, '_, '_> {
    fn value(
        &mut self,
        locator: &AspectFieldLocator,
    ) -> Result<Option<&'state AspectValue>, Denial> {
        let [field] = locator.field_path().fields() else {
            return Err(Denial::Unavailable);
        };
        let aspect = locator.aspect().aspect_key();
        admit(
            self.context,
            navigation(self.state.aspects().len(), aspect.as_str().len())?,
        )?;
        let value = self.state.get(aspect).ok_or(Denial::Unavailable)?;
        let ContractValidatedAspectValueView::Struct(fields) = value.view() else {
            return Err(Denial::Unavailable);
        };
        admit(
            self.context,
            navigation(fields.len(), field.as_str().len())?,
        )?;
        Ok(fields.get(field))
    }
    fn number(&mut self, locator: &AspectFieldLocator, expected: u64) -> Result<bool, Denial> {
        Ok(matches!(self.value(locator)?, Some(AspectValue::UInt64(value)) if *value == expected))
    }
    fn string(&mut self, locator: &AspectFieldLocator, expected: &str) -> Result<bool, Denial> {
        let Some(AspectValue::String(InternedString::Raw(value))) = self.value(locator)? else {
            return Ok(false);
        };
        admit(self.context, value.len() as u64)?;
        admit(self.context, expected.len() as u64)?;
        Ok(value == expected)
    }
    fn coordinates(
        &mut self,
        fields: [&AspectFieldLocator; 4],
        expected: Option<&WorthQueryRecordedCorrectionSource>,
    ) -> Result<bool, Denial> {
        match expected {
            Some(source) => Ok(
                self.number(fields[0], source.world().owner_identity().get())?
                    && self.number(fields[1], source.world().ordinal())?
                    && self.string(fields[2], &source.relational().branch_id.0)?
                    && self.number(fields[3], source.relational().commit_id.0)?,
            ),
            None => {
                for field in fields {
                    if self.value(field)?.is_some() {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }
    fn selector(
        &mut self,
        locator: &AspectFieldLocator,
        source: &WorthQueryRecordedCorrectionSource,
    ) -> Result<bool, Denial> {
        let value = self.value(locator)?;
        lineage_fields::selector_matches(source, value, |work| admit(self.context, work))
    }
}

fn navigation(entries: usize, key_bytes: usize) -> Result<u64, Denial> {
    (key_bytes as u64)
        .checked_add(1)
        .and_then(|width| width.checked_mul((entries as u64).checked_add(1)?))
        .ok_or(Denial::Interrupted(MapKernelStop::WorkCounterOverflow))
}
