//! Exact original identity and declared key retained for native inverse validation.
mod candidate;
#[cfg(test)]
mod candidate_tests;
use super::{WorthQueryCorrectionDerivationDenial as Denial, WorthQuerySelectedCorrectionSource};
pub(in crate::domain_computation::primary_graph) use candidate::AuthoredCorrectionCandidateDenial;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_query_declaration::facade::{
    application_aftermath::{ApplicationCorrectionBinding, DeclaredAuthoredCorrection},
    application_operation::ApplicationMutationOutputPosture as Posture,
    application_schema::{ApplicationSchema, ApplicationSchemaMember},
};
use worth_relational::facade::identity::EntityId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PreparedAuthoredIdentity {
    declaration: DeclaredAuthoredCorrection,
    original: EntityId,
    locator: AspectFieldLocator,
    key: AspectValue,
}

impl PreparedAuthoredIdentity {
    pub(super) fn retirement_anchor(&self) -> Option<EntityId> {
        (self.declaration.parts().source_posture == Posture::Create).then_some(self.original)
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
        if parts.source_posture != Posture::Retire {
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

    pub(super) fn prepare<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>(
        source: &WorthQuerySelectedCorrectionSource<'_, S>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<Self>, Denial> {
        let Some(identity) = C::authored_identity() else {
            return Ok(None);
        };
        let declaration = identity.into_declaration();
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
        let evidence = source.occurrence();
        let outputs = evidence.correspondence();
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
            })
            .ok_or(Denial::UnsupportedSourceMeaning)?;
        let locator = AspectFieldLocator::new(
            LocatorAuthority::Planned,
            AspectKey::new(aspect).ok_or(Denial::UnsupportedSourceMeaning)?,
            CanonicalFieldPath::single(
                FieldKey::new(field).ok_or(Denial::UnsupportedSourceMeaning)?,
            ),
        );
        let key = match parts.source_posture {
            Posture::Create => evidence
                .field(request, original, &locator)
                .map_err(|cause| {
                    Denial::SourceOutput(
                        super::material::WorthQueryCorrectionOutputProjectionDenial::Stopped(cause),
                    )
                })?
                .ok_or(Denial::MissingPreImage)?
                .clone(),
            Posture::Retire => evidence
                .preimage_key(original, entity, &locator)
                .ok_or(Denial::MissingPreImage)?
                .clone(),
            Posture::Preserve => return Err(Denial::UnsupportedSourceMeaning),
        };
        Ok(Some(Self {
            declaration,
            original,
            locator,
            key,
        }))
    }
}
