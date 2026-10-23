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
use worth_relational::facade::{identity::EntityId, transactions::RecordRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PreparedAuthoredIdentity {
    declaration: DeclaredAuthoredCorrection,
    original: EntityId,
    locator: AspectFieldLocator,
    key: AspectValue,
}

impl PreparedAuthoredIdentity {
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
        Ok(Some(Self {
            declaration,
            original,
            locator,
            key,
        }))
    }
}
