use serde::{Deserialize, Serialize};
use worth_foundational::facade::{
    AspectKey, CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind,
    CanonicalBasisLocus, FieldKey, InternedString,
};

use super::value::NativeValue;
use crate::diagnostics::data::fields::native_serde::{
    field_path_from_native, field_path_to_native,
};

#[derive(Serialize, Deserialize)]
pub(super) struct NativeEntry {
    domain: NativeDomain,
    locus: NativeLocus,
    kind: NativeKind,
    value: NativeValue,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) enum NativeDomain {
    Value,
    AspectContract,
    AspectMask,
    AuthoritativeState,
    AuthoritativePatch,
    Identity,
    Locator,
    Profile,
    Performance,
    BoundaryArtifact,
    Transition,
    Diagnostic,
    CompatibilityLowering,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum NativeKind {
    Header,
    Shape,
    Value,
    Field,
    Mask,
    StateAspect,
    PatchOperation,
    Identity,
    Locator,
    Profile,
    PerformanceClaim,
    PerformanceLayout,
    PerformanceCounter,
    PerformanceSupport,
    BoundaryArtifact,
    BoundaryAttachment,
    TransitionArtifact,
    TransitionLocator,
    DiagnosticBundle,
    DiagnosticRow,
    DiagnosticGap,
    CompatibilityOrigin,
    Cost,
}

#[derive(Serialize, Deserialize)]
enum NativeLocus {
    Root,
    EntryOrdinal(u32),
    Aspect(AspectKey),
    AspectField {
        aspect: AspectKey,
        path: Vec<FieldKey>,
    },
    Named(InternedString),
}

impl TryFrom<&CanonicalBasisEntry> for NativeEntry {
    type Error = String;

    fn try_from(entry: &CanonicalBasisEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            domain: entry.domain().try_into()?,
            locus: entry.locus().try_into()?,
            kind: entry.kind().try_into()?,
            value: entry.value().try_into()?,
        })
    }
}

impl TryFrom<NativeEntry> for CanonicalBasisEntry {
    type Error = String;

    fn try_from(entry: NativeEntry) -> Result<Self, Self::Error> {
        Ok(Self::new(
            entry.domain.try_into()?,
            entry.locus.try_into()?,
            entry.kind.try_into()?,
            entry.value.try_into()?,
        ))
    }
}

impl TryFrom<CanonicalBasisDomain> for NativeDomain {
    type Error = String;

    fn try_from(domain: CanonicalBasisDomain) -> Result<Self, Self::Error> {
        Ok(match domain {
            CanonicalBasisDomain::Value => Self::Value,
            CanonicalBasisDomain::AspectContract => Self::AspectContract,
            CanonicalBasisDomain::AspectMask => Self::AspectMask,
            CanonicalBasisDomain::AuthoritativeState => Self::AuthoritativeState,
            CanonicalBasisDomain::AuthoritativePatch => Self::AuthoritativePatch,
            CanonicalBasisDomain::Identity => Self::Identity,
            CanonicalBasisDomain::Locator => Self::Locator,
            CanonicalBasisDomain::Profile => Self::Profile,
            CanonicalBasisDomain::Performance => Self::Performance,
            CanonicalBasisDomain::BoundaryArtifact => Self::BoundaryArtifact,
            CanonicalBasisDomain::Transition => Self::Transition,
            CanonicalBasisDomain::Diagnostic => Self::Diagnostic,
            CanonicalBasisDomain::CompatibilityLowering => Self::CompatibilityLowering,
            CanonicalBasisDomain::Future(label) => {
                return Err(format!(
                    "future canonical basis domain `{label}` is not durable"
                ))
            }
        })
    }
}

impl TryFrom<NativeDomain> for CanonicalBasisDomain {
    type Error = String;

    fn try_from(domain: NativeDomain) -> Result<Self, Self::Error> {
        Ok(match domain {
            NativeDomain::Value => Self::Value,
            NativeDomain::AspectContract => Self::AspectContract,
            NativeDomain::AspectMask => Self::AspectMask,
            NativeDomain::AuthoritativeState => Self::AuthoritativeState,
            NativeDomain::AuthoritativePatch => Self::AuthoritativePatch,
            NativeDomain::Identity => Self::Identity,
            NativeDomain::Locator => Self::Locator,
            NativeDomain::Profile => Self::Profile,
            NativeDomain::Performance => Self::Performance,
            NativeDomain::BoundaryArtifact => Self::BoundaryArtifact,
            NativeDomain::Transition => Self::Transition,
            NativeDomain::Diagnostic => Self::Diagnostic,
            NativeDomain::CompatibilityLowering => Self::CompatibilityLowering,
        })
    }
}

impl TryFrom<CanonicalBasisEntryKind> for NativeKind {
    type Error = String;

    fn try_from(kind: CanonicalBasisEntryKind) -> Result<Self, Self::Error> {
        use CanonicalBasisEntryKind as K;
        Ok(match kind {
            K::Header => Self::Header,
            K::Shape => Self::Shape,
            K::Value => Self::Value,
            K::Field => Self::Field,
            K::Mask => Self::Mask,
            K::StateAspect => Self::StateAspect,
            K::PatchOperation => Self::PatchOperation,
            K::Identity => Self::Identity,
            K::Locator => Self::Locator,
            K::Profile => Self::Profile,
            K::PerformanceClaim => Self::PerformanceClaim,
            K::PerformanceLayout => Self::PerformanceLayout,
            K::PerformanceCounter => Self::PerformanceCounter,
            K::PerformanceSupport => Self::PerformanceSupport,
            K::BoundaryArtifact => Self::BoundaryArtifact,
            K::BoundaryAttachment => Self::BoundaryAttachment,
            K::TransitionArtifact => Self::TransitionArtifact,
            K::TransitionLocator => Self::TransitionLocator,
            K::DiagnosticBundle => Self::DiagnosticBundle,
            K::DiagnosticRow => Self::DiagnosticRow,
            K::DiagnosticGap => Self::DiagnosticGap,
            K::CompatibilityOrigin => Self::CompatibilityOrigin,
            K::Cost => Self::Cost,
            K::Future(label) => {
                return Err(format!(
                    "future canonical basis kind `{label}` is not durable"
                ))
            }
        })
    }
}

impl TryFrom<NativeKind> for CanonicalBasisEntryKind {
    type Error = String;

    fn try_from(kind: NativeKind) -> Result<Self, Self::Error> {
        Ok(match kind {
            NativeKind::Header => Self::Header,
            NativeKind::Shape => Self::Shape,
            NativeKind::Value => Self::Value,
            NativeKind::Field => Self::Field,
            NativeKind::Mask => Self::Mask,
            NativeKind::StateAspect => Self::StateAspect,
            NativeKind::PatchOperation => Self::PatchOperation,
            NativeKind::Identity => Self::Identity,
            NativeKind::Locator => Self::Locator,
            NativeKind::Profile => Self::Profile,
            NativeKind::PerformanceClaim => Self::PerformanceClaim,
            NativeKind::PerformanceLayout => Self::PerformanceLayout,
            NativeKind::PerformanceCounter => Self::PerformanceCounter,
            NativeKind::PerformanceSupport => Self::PerformanceSupport,
            NativeKind::BoundaryArtifact => Self::BoundaryArtifact,
            NativeKind::BoundaryAttachment => Self::BoundaryAttachment,
            NativeKind::TransitionArtifact => Self::TransitionArtifact,
            NativeKind::TransitionLocator => Self::TransitionLocator,
            NativeKind::DiagnosticBundle => Self::DiagnosticBundle,
            NativeKind::DiagnosticRow => Self::DiagnosticRow,
            NativeKind::DiagnosticGap => Self::DiagnosticGap,
            NativeKind::CompatibilityOrigin => Self::CompatibilityOrigin,
            NativeKind::Cost => Self::Cost,
        })
    }
}

impl TryFrom<&CanonicalBasisLocus> for NativeLocus {
    type Error = String;

    fn try_from(locus: &CanonicalBasisLocus) -> Result<Self, Self::Error> {
        Ok(match locus {
            CanonicalBasisLocus::Root => Self::Root,
            CanonicalBasisLocus::EntryOrdinal(value) => Self::EntryOrdinal(*value),
            CanonicalBasisLocus::Aspect(aspect) => Self::Aspect(aspect.clone()),
            CanonicalBasisLocus::AspectField { aspect, path } => Self::AspectField {
                aspect: aspect.clone(),
                path: field_path_to_native(path),
            },
            CanonicalBasisLocus::Named(name) => Self::Named(name.clone()),
        })
    }
}

impl TryFrom<NativeLocus> for CanonicalBasisLocus {
    type Error = String;

    fn try_from(locus: NativeLocus) -> Result<Self, Self::Error> {
        Ok(match locus {
            NativeLocus::Root => Self::Root,
            NativeLocus::EntryOrdinal(value) => Self::EntryOrdinal(value),
            NativeLocus::Aspect(aspect) => Self::Aspect(aspect),
            NativeLocus::AspectField { aspect, path } => Self::AspectField {
                aspect,
                path: field_path_from_native(path)?,
            },
            NativeLocus::Named(name) => Self::Named(name),
        })
    }
}
