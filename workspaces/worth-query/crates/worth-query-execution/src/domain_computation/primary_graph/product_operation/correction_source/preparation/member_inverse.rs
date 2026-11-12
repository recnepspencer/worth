//! Retained member truth selects Native's one correspondence projection.
use super::relation_inverse::{base, fail, request_stop};
use super::*;
use crate::domain_computation::application_aftermath::retained_preimage::WorthQueryRetainedPreImageField;
use crate::domain_computation::application_aftermath::{
    WorthQueryRelationPreImageDenial as Kind, WorthQueryRetainedPreImage,
};
use crate::domain_computation::primary_graph::admitted_capture::checkpoint;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::WorthQueryProviderSessionFailure as Denial;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_aftermath::DeclaredAuthoredCorrection;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as Posture;
use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;
use worth_relational::facade::{
    identity::KindId,
    mvcc::{RecordedMemberRestoration, RecordedMemberRestorationRefusal as Refusal},
    transactions::RecordRef,
};

struct MemberInverse<'a> {
    original: EntityId,
    kind: KindId,
    key_field: (&'a str, &'a str),
    key: Option<&'a WorthQueryRetainedPreImageField>,
    owner_kind: KindId,
    owner: Option<EntityId>,
    fields: std::ops::Range<usize>,
}

/// Every member the source retired under a live owner, and every entity kind
/// its declared families may restore. Zero members is the exact inverse of a
/// source that retired none.
pub(super) struct MemberInverses<'a> {
    members: Vec<MemberInverse<'a>>,
    fields: Vec<(&'a AspectFieldLocator, &'a AspectValue)>,
    kinds: Vec<KindId>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryPreparedCorrectionSource {
    /// Runs only for an inverse whose declaration names live-owner member
    /// families. Work is one pass over the retained fields and one over the
    /// retained relation rows, each with a point lookup among the recorded
    /// members; it never reads a roster or a thread.
    pub(super) fn member_inverses<'a>(
        &'a self,
        retained: &'a WorthQueryRetainedPreImage,
        layout: &WorthQueryPrimaryGraphLayout,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<MemberInverses<'a>, Denial> {
        let mut inverses = MemberInverses {
            members: Vec::new(),
            fields: Vec::new(),
            kinds: Vec::new(),
            _memory: None,
        };
        let Some(prepared) = self.authored_identity.as_ref() else {
            return Ok(inverses);
        };
        let declaration = prepared.declaration();
        if declaration.live_member_owner().is_none() {
            return Ok(inverses);
        }
        let key_field = |declaration: &'a DeclaredAuthoredCorrection| match declaration
            .parts()
            .logical_key
            .as_ref()
        {
            ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                ..
            } => layout
                .entity_kind(entity)
                .map(|kind| (kind, (aspect.as_str(), field.as_str())))
                .ok_or_else(|| base("installed member family missing")),
            _ => Err(base("installed member key missing")),
        };
        for family in std::iter::once(declaration).chain(declaration.co_anchors()) {
            if family.parts().source_posture == Posture::Retire {
                let (kind, _) = key_field(family)?;
                if !inverses.kinds.contains(&kind) {
                    inverses.kinds.push(kind);
                }
            }
        }
        let recorded = prepared.live_members();
        let bytes = recorded
            .len()
            .checked_mul(
                size_of::<MemberInverse<'_>>()
                    + size_of::<(EntityId, usize)>()
                    + size_of::<RecordedMemberRestoration<'_>>(),
            )
            .and_then(|bytes| {
                retained
                    .fields()
                    .len()
                    .checked_mul(size_of::<(usize, &AspectFieldLocator, &AspectValue)>() * 2)
                    .and_then(|fields| bytes.checked_add(fields))
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| base("member preimage projection size overflow"))?;
        inverses._memory = execution
            .allocation_policy()
            .reserve_memory(bytes)
            .map_err(|cause| Denial::allocation_denied(cause, "member inverse preparation"))?;
        let failed = |_| base("member preimage projection allocation failed");
        inverses
            .members
            .try_reserve_exact(recorded.len())
            .map_err(failed)?;
        let mut index = std::collections::BTreeMap::new();
        for member in recorded {
            let declared = member.declaration();
            if declared.parts().source_posture != Posture::Retire {
                continue;
            }
            let owner = declared
                .live_member_owner()
                .ok_or_else(|| base("recorded member lost its live owner"))?;
            let (kind, key_field) = key_field(declared)?;
            index.insert(member.original(), inverses.members.len());
            inverses.members.push(MemberInverse {
                original: member.original(),
                kind,
                key_field,
                key: None,
                owner_kind: layout
                    .relation(owner.relation())
                    .ok_or_else(|| base("installed owner relation missing"))?
                    .kind,
                owner: None,
                fields: 0..0,
            });
        }
        if inverses.members.is_empty() {
            return Ok(inverses);
        }
        let mut staged = Vec::new();
        staged
            .try_reserve_exact(retained.fields().len())
            .map_err(failed)?;
        inverses
            .fields
            .try_reserve_exact(retained.fields().len())
            .map_err(failed)?;
        let members = &mut inverses.members;
        let fields = &mut inverses.fields;
        execution
            .run_native_with_checkpoint(|context| {
                let stop = |cause| request_stop(worth_execution::WorkCeilingDenial::Stopped(cause));
                for value in retained.fields() {
                    checkpoint(context, 1).map_err(stop)?;
                    let RecordRef::Entity(target) = value.target_record() else {
                        continue;
                    };
                    let Some(&member) = index.get(target) else {
                        continue;
                    };
                    let locus = value.locus();
                    if (locus.aspect(), locus.field()) == members[member].key_field {
                        members[member].key = Some(value);
                    }
                    staged.push((member, value.locator(), value.value()));
                }
                checkpoint(context, staged.len() as u64).map_err(stop)?;
                staged.sort_by_key(|(member, _, _)| *member);
                for (member, locator, value) in staged {
                    let end = fields.len() + 1;
                    let range = &mut members[member].fields;
                    if range.start == range.end {
                        range.start = end - 1;
                    }
                    range.end = end;
                    fields.push((locator, value));
                }
                for row in retained.relations() {
                    checkpoint(context, 1).map_err(stop)?;
                    let mutation = row.mutation();
                    let (Some((from, to)), None) = (mutation.before(), mutation.after()) else {
                        continue;
                    };
                    if let Some(&member) = index.get(&from)
                        && members[member].owner_kind == mutation.kind()
                        && members[member].owner.is_none()
                    {
                        members[member].owner = Some(to);
                    }
                }
                Ok::<_, Denial>(())
            })
            .map_err(request_stop)?
            .0?;
        if let Some(member) = inverses
            .members
            .iter()
            .find(|member| member.key.is_none() || member.owner.is_none())
        {
            return Err(fail(
                Kind::MissingObservation,
                vec![RecordRef::Entity(member.original)],
            ));
        }
        Ok(inverses)
    }
}
impl MemberInverses<'_> {
    /// Entity kinds whose every birth must be one recorded member.
    pub(super) fn kinds(&self) -> &[KindId] {
        &self.kinds
    }

    pub(super) fn selectors(&self) -> Vec<RecordedMemberRestoration<'_>> {
        self.members
            .iter()
            .filter_map(|member| {
                let key = member.key?;
                Some(RecordedMemberRestoration {
                    original: member.original,
                    kind: member.kind,
                    owner: (member.owner_kind, member.owner?),
                    key: (key.locator(), key.value()),
                    fields: &self.fields[member.fields.clone()],
                })
            })
            .collect()
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
