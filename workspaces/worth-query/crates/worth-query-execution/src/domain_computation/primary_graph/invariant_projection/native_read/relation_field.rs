//! Pinned relation payload access has one native field body.
use super::*;
use worth_execution::{MapKernelStop, WorkCeilingDenial};
use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_relational::facade::{identity::RelationId, storage::RecordLifecycleState};
impl NativeRead<'_, '_> {
    pub fn relation_field(
        &mut self,
        relation_id: RelationId,
        from: NativeIdentity<'_>,
        to: NativeIdentity<'_>,
        field: ColumnIdentity,
        endpoints: (&str, &str),
        decode: &mut dyn FieldDecoder,
    ) -> Option<()> {
        if self.stopped.is_some() || self.decoder.stopped() {
            return None;
        }
        let layout = self.layout.relation(field.entity())?;
        if !self.local(from, endpoints.0)
            || !self.local(to, endpoints.1)
            || from.kind != layout.from
            || to.kind != layout.to
        {
            return None;
        }
        let locator =
            self.layout
                .relation_field_locator(field.entity(), field.aspect(), field.field())?;
        let [key] = locator.field_path().fields() else {
            return None;
        };
        let projection = self.native.projection(self.snapshot)?;
        let work = projection.exact_relation_state_read_work_bound()?;
        let work = usize::try_from(work).ok()?;
        if !self.budget.consume(work) {
            return None;
        }
        let decoder = self.decoder;
        let (value, _) = match self.execution.run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(work as u64)?;
            } else {
                context.poll()?;
            }
            let observed = projection
                .read_exact_relation_state(self.execution,relation_id, |record, state| {
                    if record.relation_id != relation_id
                        || record.kind_id != layout.kind
                        || record.lifecycle != RecordLifecycleState::Live
                        || record.source != from.entity_id
                        || record.target != to.entity_id
                    {
                        return None;
                    }
                    let value = state?.get(locator.aspect().aspect_key())?;
                    let ContractValidatedAspectValueView::Struct(value) = value.view() else {
                        return None;
                    };
                    let raw = value.get(key)?;
                    decode.decode_relation(decoder, field.entity(), field.aspect(), field.field(), raw);
                    Some(())
                })
                .map_err(|cause| match cause {
                    worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => {self.stopped.get_or_insert(cause);},
                    worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {},
                }).ok().flatten();
            Ok::<_, MapKernelStop>(observed)
        }) {
            Ok(value) => value,
            Err(cause) => {
                self.stopped.get_or_insert(cause.into());
                decode.discard();
                return None;
            }
        };
        match value {
            Ok(value) => {
                self.work.record_fields(1);
                self.scope.record_relation(from.entity_id, to.entity_id);
                value.flatten()
            }
            Err(cause) => {
                self.stopped
                    .get_or_insert(WorkCeilingDenial::Stopped(cause).into());
                decode.discard();
                None
            }
        }
    }
}
