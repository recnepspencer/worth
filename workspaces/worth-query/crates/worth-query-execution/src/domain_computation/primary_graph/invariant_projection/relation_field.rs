//! Privileged pinned relation payload reads. This door is not forwarded to operation readers.
use super::{WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantRelation};
use worth_execution::{MapKernelStop, WorkCeilingDenial};
use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_query_declaration::facade::application_schema::fragment::RelationSelection;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::storage::RecordLifecycleState;

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    /// Read one actual field under bootstrap-minted broad invariant authority.
    /// This supplies neither ordinary query disclosure nor an operation's
    /// recorded relation-field decision read. Those readers expose no forwarding door.
    pub fn relation_field<Relation, From, To, Field>(
        &mut self,
        identity: &WorthQueryInvariantRelation<Schema, Relation, From, To>,
        field: Field,
    ) -> Option<Field::Value>
    where
        Field: RelationSelection<Entity = Relation, From = From, To = To>,
    {
        if self.native_read_stop.is_some() || self.value_decode.stopped() {
            return None;
        }
        let relation = field;
        let layout = self.layout.relation(relation.entity())?;
        if !self.identity_is_local(identity.from(), relation.from())
            || !self.identity_is_local(identity.to(), relation.to())
            || identity.from().kind != layout.from
            || identity.to().kind != layout.to
        {
            return None;
        }
        let locator =
            self.layout
                .relation_field_locator(relation.entity(), field.aspect(), field.field())?;
        let [key] = locator.field_path().fields() else {
            return None;
        };
        let projection = self.native.projection(self.snapshot)?;
        let work = projection.exact_relation_state_read_work_bound()?;
        let work = usize::try_from(work).ok()?;
        if !self.work_budget.consume(work) {
            return None;
        }
        let decoder = self.value_decode;
        let (value, _) = match self.execution.run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(work as u64)?;
            } else {
                context.poll()?;
            }
            let observed = projection
                .read_exact_relation_state(self.execution,identity.relation_id(), |record, state| {
                    if record.relation_id != identity.relation_id()
                        || record.kind_id != layout.kind
                        || record.lifecycle != RecordLifecycleState::Live
                        || record.source != identity.from().entity_id()
                        || record.target != identity.to().entity_id()
                    {
                        return None;
                    }
                    let value = state?.get(locator.aspect().aspect_key())?;
                    let ContractValidatedAspectValueView::Struct(value) = value.view() else {
                        return None;
                    };
                    let raw = value.get(key)?;
                    decoder.decode_relation::<Field::Binding>(
                        relation.entity(),
                        field.aspect(),
                        field.field(),
                        raw,
                    )
                })
                .map_err(|cause| match cause {
                    worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => {self.native_read_stop.get_or_insert(cause);},
                    worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {},
                }).ok().flatten();
            Ok::<_, MapKernelStop>(observed)
        }) {
            Ok(value) => value,
            Err(cause) => {
                self.native_read_stop.get_or_insert(cause.into());
                return None;
            }
        };
        match value {
            Ok(value) => {
                self.work.record_fields(1);
                self.realized_scope
                    .record_relation(identity.from().entity_id(), identity.to().entity_id());
                value.flatten()
            }
            Err(cause) => {
                self.native_read_stop
                    .get_or_insert(WorkCeilingDenial::Stopped(cause).into());
                None
            }
        }
    }
}
