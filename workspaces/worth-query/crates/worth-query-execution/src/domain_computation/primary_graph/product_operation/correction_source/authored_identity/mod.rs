//! Exact original identity and declared key retained for native inverse validation.
mod candidate;
#[cfg(test)]
mod candidate_tests;
mod live_owner_members;
use super::{WorthQueryCorrectionDerivationDenial as Denial, WorthQuerySelectedCorrectionSource};
pub(in crate::domain_computation::primary_graph) use candidate::AuthoredCorrectionCandidateDenial;
pub(super) use live_owner_members::PreparedAuthoredCorrection;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_query_declaration::facade::{
    application_aftermath::{ApplicationCorrectionBinding, DeclaredAuthoredCorrection},
    application_operation::ApplicationMutationOutputPosture as Posture,
    application_schema::{ApplicationSchema, ApplicationSchemaMember},
};
use worth_relational::facade::{identity::EntityId, transactions::RecordRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PreparedAuthoredIdentity {
    declaration: DeclaredAuthoredCorrection,
    original: EntityId,
    locator: AspectFieldLocator,
    key: AspectValue,
    co_anchors: Vec<PreparedAuthoredIdentity>,
}

impl PreparedAuthoredIdentity {
    pub(super) fn declaration(&self) -> &DeclaredAuthoredCorrection {
        &self.declaration
    }
    pub(super) fn original(&self) -> EntityId {
        self.original
    }
    pub(super) fn retirement_anchor(&self) -> Option<EntityId> {
        (self.declaration.parts().source_posture == Posture::Create).then_some(self.original)
    }

    pub(super) fn retirement_co_anchors(&self) -> Vec<EntityId> {
        self.co_anchors
            .iter()
            .filter_map(Self::retirement_anchor)
            .collect()
    }
    pub(super) fn restoration_co_anchors<'a>(
        &self,
        roles: impl Iterator<
                Item = (
                    &'a str,
                    Posture,
                    &'a str,
                    &'a worth_relational::facade::transactions::EntityReference,
                ),
            > + Clone,
    ) -> Vec<(
        EntityId,
        worth_relational::facade::transactions::CreatedEntityRef,
    )> {
        self.co_anchors
            .iter()
            .filter_map(|anchor| anchor.restoration_anchor(roles.clone()))
            .collect()
    }

    pub(super) fn restoration_anchor<'a>(
        &self,
        mut roles: impl Iterator<
            Item = (
                &'a str,
                Posture,
                &'a str,
                &'a worth_relational::facade::transactions::EntityReference,
            ),
        >,
    ) -> Option<(
        EntityId,
        worth_relational::facade::transactions::CreatedEntityRef,
    )> {
        let parts = self.declaration.parts();
        if parts.source_posture != Posture::Retire || self.declaration.live_member_owner().is_some()
        {
            return None;
        }
        roles.find_map(|(name, posture, _, target)| match target {
            worth_relational::facade::transactions::EntityReference::Created(target)
                if name == parts.inverse_role && posture == Posture::Create =>
            {
                Some((self.original, target.clone()))
            }
            _ => None,
        })
    }

    /// One exact leaf per witnessed role of a declared member family. The role
    /// suffix alone pairs a source member with its inverse role; work is one
    /// scan of the source's witnessed roles and one key read per member.
    fn prepare_members<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>(
        source: &WorthQuerySelectedCorrectionSource<'_, S>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        family: &DeclaredAuthoredCorrection,
        members: &mut Vec<Self>,
    ) -> Result<(), Denial> {
        let outputs = source
            .occurrence()
            .commit_evidence()
            .output_correspondence();
        for (name, _, _, _) in
            outputs
                .native_witness_roles()
                .filter(|(name, posture, entity, _)| {
                    family
                        .member_role_suffix(name, *posture, entity, false)
                        .is_some()
                })
        {
            let mut parts = family.parts().clone();
            let suffix = &name[parts.source_role.len()..];
            parts.inverse_role = format!("{}{suffix}", parts.inverse_role);
            parts.source_role = name.to_owned();
            let mut member = DeclaredAuthoredCorrection::from_untrusted_parts(parts);
            if let Some(owner) = family.live_member_owner() {
                member = member.with_live_member_owner(owner.clone());
            }
            if let Some(member) = Self::prepare_identity::<S, C>(source, request, member)? {
                members.push(member);
            }
        }
        Ok(())
    }

    fn prepare_identity<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>(
        source: &WorthQuerySelectedCorrectionSource<'_, S>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        declaration: DeclaredAuthoredCorrection,
    ) -> Result<Option<Self>, Denial> {
        let parts = declaration.parts();
        let ApplicationSchemaMember::Field {
            entity,
            aspect,
            field,
            ..
        } = parts.logical_key.as_ref()
        else {
            return Err(Denial::UnsupportedSourceMeaning);
        };
        let evidence = source.occurrence().commit_evidence();
        let outputs = evidence.output_correspondence();
        outputs
            .require_source_binding::<C::Source>()
            .map_err(|cause| {
                Denial::SourceOutput(
                    super::material::WorthQueryCorrectionOutputProjectionDenial::Output(cause),
                )
            })?;
        let original = outputs
            .native_witness_roles()
            .find_map(|(name, posture, kind, id)| {
                (name == parts.source_role && posture == parts.source_posture && kind == entity)
                    .then_some(id)
            });
        let Some(original) = original else {
            return if declaration.is_optional() {
                Ok(None)
            } else {
                Err(Denial::UnsupportedSourceMeaning)
            };
        };
        let locator = AspectFieldLocator::new(
            LocatorAuthority::Planned,
            AspectKey::new(aspect).ok_or(Denial::UnsupportedSourceMeaning)?,
            CanonicalFieldPath::single(
                FieldKey::new(field).ok_or(Denial::UnsupportedSourceMeaning)?,
            ),
        );
        let key = match parts.source_posture {
            Posture::Create => evidence
                .committed_changes()
                .committed_field_ref(request, original, &locator)
                .map_err(|cause| {
                    Denial::SourceOutput(
                        super::material::WorthQueryCorrectionOutputProjectionDenial::Stopped(cause),
                    )
                })?
                .ok_or(Denial::MissingPreImage)?
                .clone(),
            Posture::Retire => {
                let retained = evidence
                    .retained_preimage()
                    .ok_or(Denial::MissingPreImage)?;
                retained
                    .fields()
                    .iter()
                    .find(|value| {
                        value.target_record() == &RecordRef::Entity(original)
                            && value.locator() == &locator
                            && value.locus().entity() == entity
                    })
                    .ok_or(Denial::MissingPreImage)?
                    .value()
                    .clone()
            }
            Posture::Preserve => return Err(Denial::UnsupportedSourceMeaning),
        };
        let mut co_anchors = Vec::with_capacity(declaration.co_anchors().len());
        for declared in declaration.co_anchors() {
            if declared.member_owner().is_some() {
                Self::prepare_members::<S, C>(source, request, declared, &mut co_anchors)?;
                continue;
            }
            if let Some(anchor) = Self::prepare_identity::<S, C>(source, request, declared.clone())?
            {
                co_anchors.push(anchor);
            }
        }
        Ok(Some(Self {
            co_anchors,
            declaration,
            original,
            locator,
            key,
        }))
    }
}
