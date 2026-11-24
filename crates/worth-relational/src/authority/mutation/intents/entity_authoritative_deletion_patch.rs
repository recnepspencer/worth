use worth_foundational::facade::{
    aspects, AuthoritativeRecordAspectPatch, AuthoritativeRecordAspectState,
};
use worth_proof::TransitionOutcome;

use crate::schema::data::KindAspectContractDeclarations;
use crate::transactions::data::outcomes::conflict_class::EntityAuthoritativeAspectStateDenial;

pub(crate) fn plan_entity_authoritative_deletion_patch(
    authoritative_state: Option<&AuthoritativeRecordAspectState>,
    declarations: &KindAspectContractDeclarations,
) -> Result<Option<AuthoritativeRecordAspectPatch>, EntityAuthoritativeAspectStateDenial> {
    let Some(authoritative_state) = authoritative_state else {
        return Ok(None);
    };
    let cleared_aspects = authoritative_state
        .aspects()
        .entries()
        .map(|(aspect_key, _)| {
            declarations
                .aspects
                .iter()
                .find(|binding| binding.contract.key() == aspect_key)
                .map(|binding| binding.contract.clone())
                .ok_or_else(
                    || EntityAuthoritativeAspectStateDenial::MissingAspectContract {
                        aspect_key: aspect_key.clone(),
                    },
                )
        })
        .collect::<Result<Vec<_>, _>>()?;
    construct_deletion_whole_aspect_patch(cleared_aspects)
}

fn construct_deletion_whole_aspect_patch(
    clears: impl IntoIterator<Item = worth_foundational::facade::AspectContract>,
) -> Result<Option<AuthoritativeRecordAspectPatch>, EntityAuthoritativeAspectStateDenial> {
    let mut builder = aspects().patch().whole_aspect();
    for clear in clears {
        builder = builder.clear(clear);
    }

    match builder.finish() {
        TransitionOutcome::Success(patch) if patch.is_empty() => Ok(None),
        TransitionOutcome::Success(patch) => Ok(Some(patch)),
        TransitionOutcome::Denied(denial) => {
            Err(EntityAuthoritativeAspectStateDenial::PatchConstructionDenied { denial })
        }
    }
}
