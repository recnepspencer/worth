//! Translate only owner-validated World lineage into descriptive Native axes.
use crate::durability::{
    RecoveredCustodyDisposition, WorldCheckpointImage, WorldDurableMediumDenial as Medium,
    WorldReadmissionDenial as Denial, WorldRecord,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{CanonicalDigestId, WorldOriginalFate};
use worth_relational::facade::{
    durability::{
        RelationalWorldCatalogDenial, RelationalWorldEffectCoordinates as Coordinates,
        RelationalWorldOriginalDescription, RelationalWorldSourceCatalog,
        RelationalWorldSourceDescription as Source,
    },
    history::CommitId,
    publication::PatchStreamPosition,
};

pub(in crate::durability) fn prepare(
    image: &WorldCheckpointImage,
    request: &ExecutionRequest<'_, '_>,
    selected_checkpoint: Option<Arc<RelationalWorldSourceCatalog>>,
) -> Result<Arc<RelationalWorldSourceCatalog>, Denial> {
    let mut sources = ExecutionOrderedMap::default();
    for origin in image.branch_origins() {
        let WorldRecord::Branch {
            head, components, ..
        } = &origin.record().record
        else {
            unreachable!("admitted branch origin")
        };
        sources
            .insert_cloned_native_for_request(
                &head.commit,
                &Source::Recorded(components.relational.clone()),
                *request,
            )
            .map_err(|cause| Denial::Medium(Medium::Ordered(cause)))?;
    }
    let mut published = ExecutionOrderedMap::default();
    for occurrence in image.occurrences.values() {
        let source = match occurrence.relational() {
            Some(evidence) => {
                let coordinates = Coordinates::from_recorded(
                    CommitId(evidence.commit),
                    PatchStreamPosition(evidence.position),
                    CanonicalDigestId::new(evidence.digest),
                );
                published
                    .insert_cloned_native_for_request(
                        &occurrence.open().association.attempt(),
                        &coordinates,
                        *request,
                    )
                    .map_err(|cause| Denial::Medium(Medium::Ordered(cause)))?;
                Source::Published(coordinates)
            }
            None => sources
                .get(&occurrence.open().expected.commit)
                .ok_or(Denial::MissingPredecessor)?
                .clone(),
        };
        sources
            .insert_cloned_native_for_request(&occurrence.open().commit, &source, *request)
            .map_err(|cause| Denial::Medium(Medium::Ordered(cause)))?;
    }
    let mut originals = ExecutionRequestArrayBuilder::allocate(
        *request,
        image
            .opens
            .values()
            .filter(|open| open.component_intent.changes_relational())
            .count(),
    )
    .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
    for open in image.opens.values() {
        if !open.component_intent.changes_relational() {
            continue;
        }
        let source = sources
            .get(&open.expected.commit)
            .ok_or(Denial::MissingPredecessor)?
            .clone();
        let disposition = image.dispositions.get(&open.association.attempt());
        let (fate, effect, receipt) =
            if let Some(effect) = published.get(&open.association.attempt()) {
                (WorldOriginalFate::Published, Some(*effect), None)
            } else {
                match disposition.map(|value| value.disposition()) {
                    None => (WorldOriginalFate::Open, None, None),
                    Some(RecoveredCustodyDisposition::Continued) => {
                        (WorldOriginalFate::Continued, None, None)
                    }
                    Some(RecoveredCustodyDisposition::Released(evidence)) => (
                        WorldOriginalFate::Released,
                        evidence.as_ref().map(|e| {
                            Coordinates::from_recorded(
                                e.receipt().commit_id,
                                PatchStreamPosition(e.position()),
                                CanonicalDigestId::new(*e.digest()),
                            )
                        }),
                        evidence.as_ref().map(|e| e.receipt()),
                    ),
                }
            };
        let association =
            super::super::encode_association(request, &open.association).map_err(Denial::Medium)?;
        let caller = CanonicalDigestId::new(Sha256::digest(association.elements()).into());
        originals
            .push(
                RelationalWorldOriginalDescription::from_recorded(
                    request, caller, source, fate, effect, receipt,
                )
                .map_err(catalog_denial)?,
            )
            .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
    }
    let mut heads = ExecutionRequestArrayBuilder::allocate(*request, image.heads.len())
        .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
    for head in image.heads.values() {
        heads
            .push(
                sources
                    .get(&head.fields().2)
                    .ok_or(Denial::MissingPredecessor)?
                    .clone(),
            )
            .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
    }
    RelationalWorldSourceCatalog::from_recorded(
        request,
        originals
            .seal()
            .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?,
        heads
            .seal()
            .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?,
        selected_checkpoint,
    )
    .map_err(catalog_denial)
}
fn catalog_denial(cause: RelationalWorldCatalogDenial) -> Denial {
    Denial::Medium(match cause {
        RelationalWorldCatalogDenial::Request(cause) => Medium::Request(cause),
        RelationalWorldCatalogDenial::CapacityOverflow => Medium::Unavailable,
    })
}
