use worth_proof::{Artifact, TransitionOutcome};

use crate::aspects::{AspectKey, AspectMask, DiagnosticMask, MutationMask, ProjectionMask};
use crate::CanonicalFieldPath;

use super::{
    prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact, CanonicalBasisValue,
    CanonicalDigestMaskMode, CanonicalDigestPreparationEntry, CanonicalizationRuleVersion,
    DigestPreparationReadyAspectMask, DigestPreparationReadyAspectMaskArtifact,
};

pub trait DigestPreparationMaskMode: sealed::DigestPreparationMaskModeSeal {
    fn digest_mask_mode() -> CanonicalDigestMaskMode;
}

impl DigestPreparationMaskMode for ProjectionMask {
    fn digest_mask_mode() -> CanonicalDigestMaskMode {
        CanonicalDigestMaskMode::Projection
    }
}

impl DigestPreparationMaskMode for MutationMask {
    fn digest_mask_mode() -> CanonicalDigestMaskMode {
        CanonicalDigestMaskMode::Mutation
    }
}

impl DigestPreparationMaskMode for DiagnosticMask {
    fn digest_mask_mode() -> CanonicalDigestMaskMode {
        CanonicalDigestMaskMode::Diagnostic
    }
}

mod sealed {
    use crate::aspects::{DiagnosticMask, MutationMask, ProjectionMask};

    pub trait DigestPreparationMaskModeSeal {}

    impl DigestPreparationMaskModeSeal for ProjectionMask {}
    impl DigestPreparationMaskModeSeal for MutationMask {}
    impl DigestPreparationMaskModeSeal for DiagnosticMask {}
}

pub fn prepare_aspect_mask_for_digest<Mode>(
    aspect_key: AspectKey,
    mask: AspectMask<Mode>,
) -> TransitionOutcome<DigestPreparationReadyAspectMaskArtifact<Mode>>
where
    Mode: DigestPreparationMaskMode,
{
    let basis = digest_basis_for_aspect_mask_core(
        &aspect_key,
        mask.is_whole_aspect(),
        mask.paths(),
        Mode::digest_mask_mode(),
    );

    TransitionOutcome::success(Artifact::new(DigestPreparationReadyAspectMask::new(
        aspect_key, mask, basis,
    )))
}

pub fn aspect_mask_digest_preparation_basis<Mode>(
    ready: &DigestPreparationReadyAspectMaskArtifact<Mode>,
) -> &[CanonicalDigestPreparationEntry] {
    ready.payload().basis()
}

pub fn prepare_aspect_mask_for_canonical_basis<Mode>(
    version: CanonicalizationRuleVersion,
    aspect_key: AspectKey,
    mask: AspectMask<Mode>,
) -> TransitionOutcome<CanonicalBasisReadyArtifact, super::CanonicalBasisConstructionDenial>
where
    Mode: DigestPreparationMaskMode,
{
    prepare_canonical_basis_sequence(
        version,
        CanonicalBasisDomain::AspectMask,
        canonical_basis_for_aspect_mask_core(
            &aspect_key,
            mask.is_whole_aspect(),
            mask.paths(),
            Mode::digest_mask_mode(),
        ),
    )
}

/// One compiled mask basis. `Mode` only selects [`CanonicalDigestMaskMode`].
#[inline(never)]
fn canonical_basis_for_aspect_mask_core(
    aspect_key: &AspectKey,
    whole: bool,
    paths: &[CanonicalFieldPath],
    mode: CanonicalDigestMaskMode,
) -> Vec<CanonicalBasisEntry> {
    digest_basis_for_aspect_mask_core(aspect_key, whole, paths, mode)
        .into_iter()
        .map(canonical_entry_for_mask_digest_entry)
        .collect()
}

/// One compiled digest-entry walk for a mask. The mode is a value, not a type.
#[inline(never)]
fn digest_basis_for_aspect_mask_core(
    aspect_key: &AspectKey,
    whole: bool,
    paths: &[CanonicalFieldPath],
    mode: CanonicalDigestMaskMode,
) -> Vec<CanonicalDigestPreparationEntry> {
    if whole {
        return vec![CanonicalDigestPreparationEntry::MaskWholeAspect {
            key: aspect_key.clone(),
            mode,
        }];
    }

    paths
        .iter()
        .cloned()
        .map(|path| CanonicalDigestPreparationEntry::MaskFieldPath {
            key: aspect_key.clone(),
            mode,
            path,
        })
        .collect()
}

fn canonical_entry_for_mask_digest_entry(
    entry: CanonicalDigestPreparationEntry,
) -> CanonicalBasisEntry {
    match entry {
        CanonicalDigestPreparationEntry::MaskWholeAspect { key, mode } => mask_entry(
            key.as_str(),
            format!("{}.whole", digest_mask_mode_name(mode)),
            CanonicalBasisValue::ExactText("whole".into()),
        ),
        CanonicalDigestPreparationEntry::MaskFieldPath { key, mode, path } => {
            let path_text = path
                .fields()
                .iter()
                .map(|field| field.as_str())
                .collect::<Vec<_>>()
                .join(".");
            mask_entry(
                key.as_str(),
                format!("{}.field.{path_text}", digest_mask_mode_name(mode)),
                CanonicalBasisValue::ExactText(path_text.into()),
            )
        }
        _ => unreachable!("mask canonical basis only consumes mask digest entries"),
    }
}

fn mask_entry(
    aspect_key: &str,
    locus: impl Into<String>,
    value: CanonicalBasisValue,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::AspectMask,
        CanonicalBasisLocus::Named(format!("{aspect_key}.{}", locus.into()).into()),
        CanonicalBasisEntryKind::Mask,
        value,
    )
}

fn digest_mask_mode_name(mode: CanonicalDigestMaskMode) -> &'static str {
    match mode {
        CanonicalDigestMaskMode::Projection => "projection",
        CanonicalDigestMaskMode::Mutation => "mutation",
        CanonicalDigestMaskMode::Diagnostic => "diagnostic",
    }
}
