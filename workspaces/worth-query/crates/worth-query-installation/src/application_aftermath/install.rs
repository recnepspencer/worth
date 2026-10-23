//! Installation of declared aftermath contracts.

use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::application_aftermath::{
    DeclaredCorrectionDisclosure, PortableApplicationAftermathContract, PortableCorrectionMechanism,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

use super::canonical_basis::{prepare_aftermath_basis, WorthQueryAftermathCanonicalArtifact};
use super::correction_authority::InstalledCorrectionAuthority;
use super::correction_mechanism::InstalledCorrectionMechanism;
use super::denial::{
    WorthQueryAftermathInstallationDenial, WorthQueryAftermathInstallationDenialKind,
};
use super::external_effect_contract::InstalledExternalEffectPosture;
use super::install_validation::{
    escaping_effect_subject, validate_axis_pair, validate_preimage_coverage,
    OperationDeclaredReadFields,
};
use super::lowering_correspondence::{
    AftermathLoweringCorrespondenceCatalog, InstalledLoweringCorrespondence,
};
use super::next_action_contract::InstalledAftermathNextActionContract;
use super::owner_identity::aftermath_owner_identity_digest;
use super::published_posture::{derive_published_posture, PublishedAftermathPosture};
use super::reconciliation::WorthQueryInstalledReconciliationProcedure;
use super::recovery_contract::InstalledAftermathRecoveryContract;

/// Opaque installed aftermath identity retained from the Foundational digest.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryInstalledAftermathIdentity(CanonicalDigestId);

impl WorthQueryInstalledAftermathIdentity {
    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.0
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        self.0.bytes()
    }

    pub fn render_support_hex(&self) -> String {
        self.0.render_hex()
    }
}

/// Installed aftermath contract for one mutation operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledAftermathContract {
    identity: WorthQueryInstalledAftermathIdentity,
    authority: InstalledCorrectionAuthority,
    mechanism: Option<InstalledCorrectionMechanism>,
    correction_disclosure: Option<DeclaredCorrectionDisclosure>,
    external_effect: InstalledExternalEffectPosture,
    published_posture: PublishedAftermathPosture,
    next_actions: InstalledAftermathNextActionContract,
    recovery: InstalledAftermathRecoveryContract,
    reconciliation: Option<WorthQueryInstalledReconciliationProcedure>,
    canonical: WorthQueryAftermathCanonicalArtifact,
    operation_slot: String,
    compatibility_generation: u64,
}

impl WorthQueryInstalledAftermathContract {
    pub const fn identity(&self) -> &WorthQueryInstalledAftermathIdentity {
        &self.identity
    }

    pub const fn authority(&self) -> InstalledCorrectionAuthority {
        self.authority
    }

    pub const fn mechanism(&self) -> Option<&InstalledCorrectionMechanism> {
        self.mechanism.as_ref()
    }

    pub const fn correction_disclosure(&self) -> Option<&DeclaredCorrectionDisclosure> {
        self.correction_disclosure.as_ref()
    }

    pub const fn external_effect(&self) -> &InstalledExternalEffectPosture {
        &self.external_effect
    }

    pub const fn published_posture(&self) -> PublishedAftermathPosture {
        self.published_posture
    }

    pub const fn next_actions(&self) -> InstalledAftermathNextActionContract {
        self.next_actions
    }

    pub const fn recovery(&self) -> InstalledAftermathRecoveryContract {
        self.recovery
    }

    pub const fn reconciliation(&self) -> Option<&WorthQueryInstalledReconciliationProcedure> {
        self.reconciliation.as_ref()
    }

    pub const fn canonical(&self) -> &WorthQueryAftermathCanonicalArtifact {
        &self.canonical
    }

    pub fn operation_slot(&self) -> &str {
        &self.operation_slot
    }

    pub const fn compatibility_generation(&self) -> u64 {
        self.compatibility_generation
    }
}

/// Install one declared aftermath contract for an application operation.
///
/// Package identity, schema identity, generation, and operation slot are taken
/// from the binding and operation already under compilation. Pre-image coverage
/// is validated against `declared_reads` from that same operation, and the
/// escaping posture is derived from that same operation's external-effect
/// contract. This is the sole aftermath installation door.
pub(crate) fn install_application_aftermath(
    operation: &impl crate::application_operation::WorthQueryOperationAftermathInstallationSource,
) -> Result<Option<WorthQueryInstalledAftermathContract>, WorthQueryAftermathInstallationDenial> {
    let Some(portable) = operation.portable_aftermath() else {
        return Ok(None);
    };
    let binding = operation.binding();
    let operation_slot = operation.operation();
    let declared_reads =
        OperationDeclaredReadFields::from_targets(operation.portable_decision_reads());
    let external_effect = operation.external_effect();
    let lowering_catalog = derived_lowering_catalog(binding, portable)?;
    let compatibility_generation = binding.generation();
    let reconciliation = operation
        .portable_reconciliation()
        .map(WorthQueryInstalledReconciliationProcedure::from_portable);
    validate_axis_pair(portable, reconciliation.is_some())?;
    validate_preimage_coverage(portable, &declared_reads)?;
    let resolved_lowering = resolve_lowering_correspondence(
        portable,
        compatibility_generation,
        binding.schema_identity(),
        &lowering_catalog,
    )?;
    let canonical = prepare_aftermath_basis(
        binding,
        operation_slot,
        portable,
        reconciliation.as_ref(),
        external_effect,
    )?;
    let authority = InstalledCorrectionAuthority::from(portable.authority());
    let mechanism = match portable.mechanism() {
        Some(portable_mechanism) => Some(
            InstalledCorrectionMechanism::from_portable(portable_mechanism, resolved_lowering)
                .map_err(|subject| {
                    WorthQueryAftermathInstallationDenial::new(
                        WorthQueryAftermathInstallationDenialKind::LoweringCorrespondenceUnresolved,
                        subject,
                    )
                })?,
        ),
        None => None,
    };
    let published_posture = derive_published_posture(authority, mechanism.as_ref())?;
    // The single reversibility guard, and it reads two derived facts: the
    // posture the runtime just derived, and the operation's own escaping lane.
    // Neither is a claim the aftermath declaration could have made about itself
    // (Q8.25-C1). A pre-flight twin of this check used to re-derive "would this
    // be reversible?" from (authority, mechanism) inline — a second declaration
    // of what `derive_published_posture` already owns, free to drift from it.
    if published_posture == PublishedAftermathPosture::Reversible && external_effect.is_declared() {
        return Err(WorthQueryAftermathInstallationDenial::new(
            WorthQueryAftermathInstallationDenialKind::ExternalEffectRejectsReversible,
            escaping_effect_subject(external_effect),
        ));
    }
    let next_actions = InstalledAftermathNextActionContract::for_posture(published_posture);
    let recovery = InstalledAftermathRecoveryContract::for_posture(published_posture);
    Ok(Some(WorthQueryInstalledAftermathContract {
        identity: WorthQueryInstalledAftermathIdentity(*canonical.digest()),
        authority,
        mechanism,
        correction_disclosure: portable
            .correction_binding()
            .map(|inverse| inverse.disclosure().clone()),
        external_effect: InstalledExternalEffectPosture::from_operation_contract(external_effect),
        published_posture,
        next_actions,
        recovery,
        reconciliation,
        canonical,
        operation_slot: operation_slot.to_owned(),
        compatibility_generation,
    }))
}

/// Build a lowering catalog for a recorded-inverse declaration from binding truth.
pub(crate) fn derived_lowering_catalog(
    binding: &ApplicationSchemaBindingIdentity,
    portable: &PortableApplicationAftermathContract,
) -> Result<AftermathLoweringCorrespondenceCatalog, WorthQueryAftermathInstallationDenial> {
    let Some(PortableCorrectionMechanism::RecordedInverse(inverse)) = portable.mechanism() else {
        return Ok(AftermathLoweringCorrespondenceCatalog::empty());
    };
    let slot = inverse.lowering_correspondence().correspondence_slot();
    let correspondence_identity =
        aftermath_owner_identity_digest("worth-query.lowering-correspondence", slot, 1, 0)?;
    Ok(AftermathLoweringCorrespondenceCatalog::new([
        InstalledLoweringCorrespondence::new(
            slot,
            correspondence_identity,
            binding.generation(),
            *binding.schema_identity(),
        )
        .map_err(|subject| {
            WorthQueryAftermathInstallationDenial::new(
                WorthQueryAftermathInstallationDenialKind::LoweringCorrespondenceUnresolved,
                subject,
            )
        })?,
    ]))
}

fn resolve_lowering_correspondence(
    portable: &PortableApplicationAftermathContract,
    compatibility_generation: u64,
    graph_participation_identity: &CanonicalDigestId,
    lowering_catalog: &AftermathLoweringCorrespondenceCatalog,
) -> Result<Option<super::InstalledLoweringCorrespondence>, WorthQueryAftermathInstallationDenial> {
    let Some(PortableCorrectionMechanism::RecordedInverse(inverse)) = portable.mechanism() else {
        return Ok(None);
    };
    use super::LoweringCorrespondenceResolutionDenial as D;
    lowering_catalog
        .resolve(
            inverse.lowering_correspondence().correspondence_slot(),
            compatibility_generation,
            graph_participation_identity,
        )
        .map(Some)
        .map_err(|denial| {
            let (kind, subject) = match denial {
                D::Unresolved => (
                    WorthQueryAftermathInstallationDenialKind::LoweringCorrespondenceUnresolved,
                    "unresolved-lowering-correspondence",
                ),
                D::WrongGeneration => (
                    WorthQueryAftermathInstallationDenialKind::LoweringCorrespondenceWrongGeneration,
                    "wrong-generation-lowering-correspondence",
                ),
                D::MismatchedGraphParticipation => (
                    WorthQueryAftermathInstallationDenialKind::LoweringCorrespondenceMismatchedGraphParticipation,
                    "mismatched-graph-participation-lowering-correspondence",
                ),
                D::Ambiguous => (
                    WorthQueryAftermathInstallationDenialKind::LoweringCorrespondenceAmbiguous,
                    "ambiguous-lowering-correspondence",
                ),
            };
            WorthQueryAftermathInstallationDenial::new(kind, subject)
        })
}
