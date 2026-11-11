//! Join Native tail metadata to the same owner-validated World journal source.
use crate::{ApplicationHomeOpenRefusal as Denial, ReadmittedApplicationHome};
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_foundational::facade::{
    CanonicalDigestId, DurableOwnerFamily, HomeProgramTransitionCaller, UncleanRecoveryUnsupported,
    WorldOriginalFate,
};
use worth_runtime_world::facade::{WorldDurableMediumDenial, WorldReadmissionDenial};

pub(super) fn scan(
    home: &ReadmittedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let Some(selection) = home
        .context
        .world_preparation
        .as_ref()
        .and_then(|prepared| prepared.selection.as_ref())
    else {
        return Ok(());
    };
    let mut open = ExecutionOrderedMap::<CanonicalDigestId, ()>::default();
    for original in selection.source().originals() {
        match original.fate() {
            WorldOriginalFate::Open => {
                open.insert_cloned_native_for_request(original.caller(), &(), *request)
                    .map_err(|cause| {
                        Denial::World(WorldReadmissionDenial::Medium(
                            WorldDurableMediumDenial::Ordered(cause),
                        ))
                    })?;
            }
            WorldOriginalFate::Published
            | WorldOriginalFate::Continued
            | WorldOriginalFate::Released => {}
        }
    }
    let covered = home.context.checkpoint.acknowledged_native()[0];
    let subjects = home
        .relational
        .runtime()
        .durability_authority()
        .discover_all_associated_effects(request)
        .map_err(|cause| Denial::World(WorldReadmissionDenial::NativeDiscovery(cause)))?;
    for subject in subjects {
        if subject.position().0 <= covered {
            continue;
        }
        let Some(caller) = subject.association() else {
            continue;
        };
        let bytes = if HomeProgramTransitionCaller::recognizes(caller.as_bytes()) {
            HomeProgramTransitionCaller::read(caller.as_bytes())
                .map_err(|_| Denial::World(WorldReadmissionDenial::MalformedNativeAssociation))?
                .original()
        } else {
            caller.as_bytes()
        };
        let key = CanonicalDigestId::new(Sha256::digest(bytes).into());
        if open.contains_key(&key) {
            return Err(Denial::UncleanRecoveryUnsupported(
                UncleanRecoveryUnsupported {
                    owner: DurableOwnerFamily::Relational,
                    effect: CanonicalDigestId::new(*subject.artifact_digest()),
                },
            ));
        }
    }
    Ok(())
}
