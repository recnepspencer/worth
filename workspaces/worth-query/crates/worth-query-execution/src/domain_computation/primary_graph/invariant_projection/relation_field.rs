//! Privileged pinned relation payload reads. This door is not forwarded to operation readers.
use super::{WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantRelation};
use worth_execution::{MapKernelStop, WorkCeilingDenial};
use worth_foundational::facade::{
    AspectContractRevision, AspectIdentity, AspectValue, ContractValidatedAspectValueView,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationFieldMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationReadableScalarValueBinding, ApplicationRelationFieldRef,
    ApplicationSchema,
};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::storage::RecordLifecycleState;

struct NamedRelationField<'a> {
    relation_id: RelationId,
    relation: &'a str,
    from_entity: &'a str,
    to_entity: &'a str,
    aspect: &'a str,
    field: &'a str,
    from_id: EntityId,
    to_id: EntityId,
    from_kind: KindId,
    to_kind: KindId,
    from_authority: u64,
    to_authority: u64,
    from_name: &'a str,
    to_name: &'a str,
    aspect_identity: AspectIdentity,
    contract_revision: AspectContractRevision,
}

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    /// Read one actual field under bootstrap-minted broad invariant authority.
    /// This supplies neither ordinary query disclosure nor an operation's
    /// recorded relation-field decision read. Those readers expose no forwarding door.
    pub fn relation_field<Relation, From, To, Aspect, Field, Write, Unit>(
        &mut self,
        identity: &WorthQueryInvariantRelation<Schema, Relation, From, To>,
        field: ApplicationRelationFieldRef<Schema, Relation, From, To, Aspect, Field, Write, Unit>,
    ) -> Option<Field::Value>
    where
        Aspect: ApplicationAspectMarkerIdentity<Schema, Relation>,
        Field: ApplicationFieldMarkerIdentity<Schema, Relation, Aspect>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Unit: ApplicationFieldUnit,
    {
        let relation = field.relation();
        let decoder = self.value_decode;
        let mut decoded = None;
        let visited = self.read_named_relation_field(
            &NamedRelationField {
                relation_id: identity.relation_id(),
                relation: relation.name(),
                from_entity: relation.from(),
                to_entity: relation.to(),
                aspect: field.aspect(),
                field: field.field(),
                from_id: identity.from().entity_id,
                to_id: identity.to().entity_id,
                from_kind: identity.from().kind,
                to_kind: identity.to().kind,
                from_authority: identity.from().authority_identity,
                to_authority: identity.to().authority_identity,
                from_name: identity.from().entity.as_ref(),
                to_name: identity.to().entity.as_ref(),
                aspect_identity: Aspect::ASPECT_IDENTITY,
                contract_revision: Aspect::CONTRACT_REVISION,
            },
            &mut |raw| {
                decoded = Some(decoder.decode_relation::<Field::Binding>(
                    relation.name(),
                    field.aspect(),
                    field.field(),
                    raw,
                ));
            },
        );
        visited.and_then(|_| decoded.flatten())
    }

    /// Locality stays ahead of the aspect-contract check. The decoded value
    /// stays with the shell; this function only reports that the visit ran.
    #[inline(never)]
    fn read_named_relation_field(
        &mut self,
        subject: &NamedRelationField<'_>,
        decode: &mut dyn FnMut(&AspectValue),
    ) -> Option<()> {
        if self.native_read_stop.is_some() || self.value_decode.stopped() {
            return None;
        }
        let layout = self.layout.relation(subject.relation)?;
        if subject.from_authority != self.authority_identity
            || subject.from_name != subject.from_entity
            || subject.to_authority != self.authority_identity
            || subject.to_name != subject.to_entity
            || subject.from_kind != layout.from
            || subject.to_kind != layout.to
        {
            return None;
        }
        let contract = self
            .layout
            .relation_aspect_contract(subject.relation, subject.aspect)?;
        if contract.identity() != subject.aspect_identity
            || contract.revision() != subject.contract_revision
        {
            return None;
        }
        let locator =
            self.layout
                .relation_field_locator(subject.relation, subject.aspect, subject.field)?;
        let [key] = locator.field_path().fields() else {
            return None;
        };
        let projection = self.runtime.read_truth().project_snapshot(self.snapshot)?;
        let work = projection.exact_relation_state_read_work_bound()?;
        let work = usize::try_from(work).ok()?;
        if !self.work_budget.consume(work) {
            return None;
        }
        let (value, _) = match self.execution.run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(work as u64)?;
            } else {
                context.poll()?;
            }
            let observed = projection
                .read_exact_relation_state(self.execution, subject.relation_id, |record, state| {
                    if record.relation_id != subject.relation_id
                        || record.kind_id != layout.kind
                        || record.lifecycle != RecordLifecycleState::Live
                        || record.source != subject.from_id
                        || record.target != subject.to_id
                    {
                        return None;
                    }
                    let value = state?.get(locator.aspect().aspect_key())?;
                    let ContractValidatedAspectValueView::Struct(value) = value.view() else {
                        return None;
                    };
                    let raw = value.get(key)?;
                    decode(raw);
                    Some(())
                })
                .map_err(|cause| match cause {
                    worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => {
                        self.native_read_stop.get_or_insert(cause);
                    }
                    worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {}
                })
                .ok()
                .flatten();
            Ok::<_, MapKernelStop>(observed)
        }) {
            Ok(value) => value,
            Err(cause) => {
                self.native_read_stop.get_or_insert(cause.into());
                return None;
            }
        };
        match value {
            Ok(_value) => {
                self.work.record_fields(1);
                self.realized_scope
                    .record_relation(subject.from_id, subject.to_id);
                Some(())
            }
            Err(cause) => {
                self.native_read_stop
                    .get_or_insert(WorkCeilingDenial::Stopped(cause).into());
                None
            }
        }
    }
}
