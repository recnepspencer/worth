use sha2::Digest;
use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{UiFontCollectionGeneration, UiQualifiedFontPackIdentity};

use super::{
    admission::{coverage_range_count, instantiate},
    application_pack::{qualify, UiPreflightedApplicationPack},
    UiApplicationFontPackDefinition, UiFontCollectionAdmissionCost,
    UiFontCollectionAdmissionDenial, UiFontFaceSource, UiGlobalFontCollection,
    UiQualifiedFontPackReceipt,
};

type PackTransition = Result<
    (
        UiGlobalFontCollection,
        UiQualifiedFontPackReceipt,
        UiFontCollectionAdmissionCost,
    ),
    UiFontCollectionAdmissionDenial,
>;

impl UiGlobalFontCollection {
    pub fn register_application_pack(
        &self,
        successor_generation: UiFontCollectionGeneration,
        definition: UiApplicationFontPackDefinition,
    ) -> PackTransition {
        self.transition_application_pack(successor_generation, None, Some(definition))
    }

    pub fn replace_application_pack(
        &self,
        predecessor: UiQualifiedFontPackIdentity,
        successor_generation: UiFontCollectionGeneration,
        definition: UiApplicationFontPackDefinition,
    ) -> PackTransition {
        self.transition_application_pack(successor_generation, Some(predecessor), Some(definition))
    }

    pub fn remove_application_pack(
        &self,
        predecessor: UiQualifiedFontPackIdentity,
        successor_generation: UiFontCollectionGeneration,
    ) -> Result<Self, UiFontCollectionAdmissionDenial> {
        let (collection, _, _) =
            self.transition_application_pack(successor_generation, Some(predecessor), None)?;
        Ok(collection)
    }

    fn transition_application_pack(
        &self,
        successor_generation: UiFontCollectionGeneration,
        predecessor: Option<UiQualifiedFontPackIdentity>,
        definition: Option<UiApplicationFontPackDefinition>,
    ) -> PackTransition {
        self.validate_successor_generation(successor_generation)?;
        if predecessor
            .is_some_and(|identity| !self.packs.iter().any(|pack| pack.identity() == identity))
        {
            return Err(UiFontCollectionAdmissionDenial::UnknownFontPack);
        }
        preflight_face_capacity(self, predecessor, definition.as_ref())?;
        let preflighted = definition
            .map(|definition| preflight_application_pack(self, predecessor, definition))
            .transpose()?;
        let application_bytes = preflighted.as_ref().map_or_else(
            || retained_application_bytes(self, predecessor),
            |pack| Ok(pack.application_bytes),
        )?;
        let bytes_hashed = preflighted.as_ref().map_or(0, |pack| pack.bytes_hashed);
        let qualified = preflighted
            .map(|definition| qualify(definition, successor_generation))
            .transpose()?;
        if qualified.as_ref().is_some_and(|candidate| {
            self.packs
                .iter()
                .any(|pack| pack.identity() == candidate.receipt.identity())
                && predecessor != Some(candidate.receipt.identity())
        }) {
            return Err(UiFontCollectionAdmissionDenial::DuplicateFontPack);
        }
        let mut sources = self
            .sources
            .iter()
            .filter(|source| source.pack.is_none() || source.pack != predecessor)
            .cloned()
            .collect::<Vec<_>>();
        if let Some(candidate) = &qualified {
            sources.extend(candidate.sources.iter().cloned());
        }
        let application_faces = sources
            .iter()
            .filter(|source| source.pack.is_some())
            .count();
        if application_faces > crate::UiGlobalTextProfile::MAX_APPLICATION_FONT_FACES {
            return Err(UiFontCollectionAdmissionDenial::ApplicationFaceCapacityExceeded);
        }
        debug_assert_eq!(application_bytes, unique_application_bytes(&sources)?);
        let faces = instantiate(&sources)?;
        let mut packs = self
            .packs
            .iter()
            .filter(|pack| Some(pack.identity()) != predecessor)
            .cloned()
            .collect::<Vec<_>>();
        let receipt = qualified
            .as_ref()
            .map(|candidate| candidate.receipt.clone())
            .unwrap_or_else(|| UiQualifiedFontPackReceipt {
                identity: predecessor.expect("remove names a predecessor"),
                collection_generation: successor_generation,
                families: Box::new([]),
                faces: Box::new([]),
            });
        if let Some(candidate) = qualified {
            packs.push(candidate.receipt);
        }
        let cost = UiFontCollectionAdmissionCost {
            faces_checked: u16::try_from(faces.len()).expect("profile capacity fits u16"),
            bytes_hashed: u64::try_from(bytes_hashed).expect("profile capacity fits u64"),
            shaper_data_built: u16::try_from(faces.len()).expect("profile capacity fits u16"),
            coverage_ranges_built: coverage_range_count(&faces)?,
        };
        self.advance_lineage(successor_generation)?;
        Ok((
            Self {
                generation: successor_generation,
                identity_digest: super::collection_identity(&sources),
                capacity_bound: super::UiFontCollectionCapacityBound::from_sources(&sources),
                lineage: std::sync::Arc::clone(&self.lineage),
                sources: sources.into_boxed_slice(),
                faces: faces.into_boxed_slice(),
                packs: packs.into_boxed_slice(),
                application_bytes,
            },
            receipt,
            cost,
        ))
    }
}

fn preflight_application_pack(
    collection: &UiGlobalFontCollection,
    predecessor: Option<UiQualifiedFontPackIdentity>,
    definition: UiApplicationFontPackDefinition,
) -> Result<UiPreflightedApplicationPack, UiFontCollectionAdmissionDenial> {
    use UiFontCollectionAdmissionDenial as Denial;
    let limit = crate::UiGlobalTextProfile::MAX_APPLICATION_FONT_BYTES;
    if definition.faces.iter().any(|face| face.bytes.len() > limit) {
        return Err(Denial::ApplicationFontByteCapacityExceeded);
    }
    let mut bytes_by_digest = retained_application_byte_map(collection, predecessor);
    let mut digests = Vec::with_capacity(definition.faces.len());
    let mut bytes_hashed = 0usize;
    for face in &definition.faces {
        bytes_hashed = bytes_hashed
            .checked_add(face.bytes.len())
            .ok_or(Denial::ApplicationFontByteCapacityExceeded)?;
        let digest: [u8; 32] = UiCountedSha256::digest(&face.bytes).into();
        bytes_by_digest.entry(digest).or_insert(face.bytes.len());
        if total_application_bytes(&bytes_by_digest)? > limit {
            return Err(Denial::ApplicationFontByteCapacityExceeded);
        }
        digests.push(digest);
    }
    Ok(UiPreflightedApplicationPack {
        definition,
        face_digests: digests.into_boxed_slice(),
        bytes_hashed,
        application_bytes: total_application_bytes(&bytes_by_digest)?,
    })
}

fn retained_application_bytes(
    collection: &UiGlobalFontCollection,
    predecessor: Option<UiQualifiedFontPackIdentity>,
) -> Result<usize, UiFontCollectionAdmissionDenial> {
    total_application_bytes(&retained_application_byte_map(collection, predecessor))
}

fn retained_application_byte_map(
    collection: &UiGlobalFontCollection,
    predecessor: Option<UiQualifiedFontPackIdentity>,
) -> std::collections::BTreeMap<[u8; 32], usize> {
    collection
        .sources
        .iter()
        .filter(|source| source.pack.is_some() && source.pack != predecessor)
        .map(|source| (source.identity.font_bytes_digest(), source.bytes.len()))
        .collect()
}

fn total_application_bytes(
    bytes_by_digest: &std::collections::BTreeMap<[u8; 32], usize>,
) -> Result<usize, UiFontCollectionAdmissionDenial> {
    bytes_by_digest.values().try_fold(0usize, |total, bytes| {
        total
            .checked_add(*bytes)
            .ok_or(UiFontCollectionAdmissionDenial::ApplicationFontByteCapacityExceeded)
    })
}

fn preflight_face_capacity(
    collection: &UiGlobalFontCollection,
    predecessor: Option<UiQualifiedFontPackIdentity>,
    definition: Option<&UiApplicationFontPackDefinition>,
) -> Result<(), UiFontCollectionAdmissionDenial> {
    let retained = collection
        .sources
        .iter()
        .filter(|source| source.pack.is_some() && source.pack != predecessor)
        .count();
    let incoming = definition.map_or(0, |definition| definition.faces.len());
    if retained.saturating_add(incoming) > crate::UiGlobalTextProfile::MAX_APPLICATION_FONT_FACES {
        Err(UiFontCollectionAdmissionDenial::ApplicationFaceCapacityExceeded)
    } else {
        Ok(())
    }
}

fn unique_application_bytes(
    sources: &[UiFontFaceSource],
) -> Result<usize, UiFontCollectionAdmissionDenial> {
    let mut bytes_by_digest = std::collections::BTreeMap::new();
    for source in sources.iter().filter(|source| source.pack.is_some()) {
        bytes_by_digest
            .entry(source.identity.font_bytes_digest())
            .or_insert(source.bytes.len());
    }
    total_application_bytes(&bytes_by_digest)
}
