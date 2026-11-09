//! Retained member truth selects Native's one correspondence projection.
use super::relation_inverse::{base, fail};
use super::*;
use crate::domain_computation::application_aftermath::retained_preimage::WorthQueryRetainedPreImageField;
use crate::domain_computation::application_aftermath::{
    WorthQueryRelationPreImageDenial as Kind, WorthQueryRetainedPreImage,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::WorthQueryProviderSessionFailure as Denial;
use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;
use worth_relational::facade::{
    identity::KindId,
    mvcc::{RecordedMemberRestoration, RecordedMemberRestorationRefusal as Refusal},
    transactions::RecordRef,
};

pub(super) struct MemberInverse<'a> {
    original: EntityId,
    kind: KindId,
    key: &'a WorthQueryRetainedPreImageField,
    fields: Vec<(
        &'a worth_foundational::facade::AspectFieldLocator,
        &'a worth_foundational::facade::AspectValue,
    )>,
    owner_kind: KindId,
    owner: EntityId,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryPreparedCorrectionSource {
    pub(super) fn member_inverse<'a>(
        &self,
        retained: &'a WorthQueryRetainedPreImage,
        layout: &WorthQueryPrimaryGraphLayout,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<MemberInverse<'a>>, Denial> {
        let Some(declaration) = self.authored_declaration() else {
            return Ok(None);
        };
        let Some(owner) = declaration.live_member_owner() else {
            return Ok(None);
        };
        let parts = declaration.parts();
        let ApplicationSchemaMember::Field {
            entity,
            aspect,
            field,
            ..
        } = parts.logical_key.as_ref()
        else {
            return Err(base("installed member key missing"));
        };
        let original = self
            .material
            .commit_evidence()
            .output_correspondence()
            .native_witness_roles()
            .find_map(|(role, posture, kind, id)| {
                (role == parts.source_role && posture == parts.source_posture && kind == entity)
                    .then_some(id)
            })
            .ok_or_else(|| base("retired member output missing"))?;
        let key = retained
            .fields()
            .iter()
            .find(|value| {
                value.target_record() == &RecordRef::Entity(original)
                    && value.locus().entity() == entity
                    && value.locus().aspect() == aspect
                    && value.locus().field() == field
            })
            .ok_or_else(|| fail(Kind::MissingObservation, vec![RecordRef::Entity(original)]))?;
        let owner_kind = layout
            .relation(owner.relation())
            .ok_or_else(|| base("installed owner relation missing"))?
            .kind;
        let owner = retained
            .relations()
            .iter()
            .find_map(|row| {
                let mutation = row.mutation();
                (mutation.kind() == owner_kind && mutation.after().is_none())
                    .then(|| mutation.before())
                    .flatten()
                    .and_then(|(from, to)| (from == original).then_some(to))
            })
            .ok_or_else(|| fail(Kind::MissingObservation, vec![RecordRef::Entity(original)]))?;
        let bytes = retained
            .fields()
            .len()
            .checked_mul(size_of::<(
                &worth_foundational::facade::AspectFieldLocator,
                &worth_foundational::facade::AspectValue,
            )>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| base("member preimage projection size overflow"))?;
        let memory = execution
            .allocation_policy()
            .reserve_memory(bytes)
            .map_err(Denial::allocation_denied)?;
        let mut fields = Vec::new();
        fields
            .try_reserve_exact(retained.fields().len())
            .map_err(|_| base("member preimage projection allocation failed"))?;
        execution
            .run_native_with_checkpoint(|context| {
                for value in retained.fields() {
                    crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                        context, 1,
                    )
                    .map_err(|cause| {
                        super::relation_inverse::request_stop(
                            worth_execution::WorkCeilingDenial::Stopped(cause),
                        )
                    })?;
                    if value.target_record() == &RecordRef::Entity(original) {
                        fields.push((value.locator(), value.value()));
                    }
                }
                Ok::<_, Denial>(())
            })
            .map_err(super::relation_inverse::request_stop)?
            .0?;
        Ok(Some(MemberInverse {
            original,
            kind: key.entity_kind(),
            key,
            fields,
            owner_kind,
            owner,
            _memory: memory,
        }))
    }
}
impl MemberInverse<'_> {
    pub(super) fn selector(&self) -> RecordedMemberRestoration<'_> {
        RecordedMemberRestoration {
            original: self.original,
            kind: self.kind,
            owner: (self.owner_kind, self.owner),
            key: (self.key.locator(), self.key.value()),
            fields: &self.fields,
        }
    }
}

pub(super) fn refused(refusal: Refusal) -> Denial {
    let (kind, member) = match refusal {
        Refusal::Incomplete(id) => (Kind::IncompleteInverse, id),
        Refusal::Repeated(id) => (Kind::RepeatedInverse, id),
        Refusal::Foreign(id) => (Kind::ForeignEndpoint, id),
        Refusal::Extra(id) => (Kind::ExtraInverse, id),
        Refusal::FieldMismatch(id) => (Kind::RestoredFieldMismatch, id),
    };
    fail(kind, vec![RecordRef::Entity(member)])
}
