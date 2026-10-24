//! Axis-pair, external-effect, and pre-image coverage checks for aftermath install.

use worth_query_declaration::facade::application_aftermath::{
    DeclaredCorrectionAuthority, PortableApplicationAftermathContract, PortableCorrectionMechanism,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget;

use super::denial::{
    WorthQueryAftermathInstallationDenial, WorthQueryAftermathInstallationDenialKind,
};
use super::external_effect_contract::InstalledExternalEffectContract;

/// Exact field targets projected from an operation's retained portable reads.
///
/// Built only at the operation-compile resolution site (and crate-internal
/// tests of that derivation). There is no public constructor: a caller cannot
/// author a coverage list independent of the operation being installed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OperationDeclaredReadFields {
    field_targets: Vec<ApplicationOperationDecisionReadTarget>,
}

impl OperationDeclaredReadFields {
    pub(crate) fn from_targets<'a>(
        targets: impl IntoIterator<Item = &'a ApplicationOperationDecisionReadTarget>,
    ) -> Self {
        Self {
            field_targets: targets
                .into_iter()
                .filter(|target| {
                    matches!(target, ApplicationOperationDecisionReadTarget::Field { .. })
                })
                .cloned()
                .collect(),
        }
    }

    pub(crate) fn field_targets(&self) -> &[ApplicationOperationDecisionReadTarget] {
        &self.field_targets
    }
}

pub(super) fn validate_axis_pair(
    portable: &PortableApplicationAftermathContract,
    has_reconciliation: bool,
) -> Result<(), WorthQueryAftermathInstallationDenial> {
    match portable.authority() {
        DeclaredCorrectionAuthority::NotCorrectable => {
            if portable.mechanism().is_some() {
                return Err(WorthQueryAftermathInstallationDenial::new(
                    WorthQueryAftermathInstallationDenialKind::MechanismPresentForNotCorrectable,
                    "not-correctable-rejects-mechanism",
                ));
            }
            if has_reconciliation {
                return Err(WorthQueryAftermathInstallationDenial::new(
                    WorthQueryAftermathInstallationDenialKind::ReconciliationForbidden,
                    "not-correctable-rejects-reconciliation",
                ));
            }
            Ok(())
        }
        DeclaredCorrectionAuthority::RuntimeAlone => {
            if portable.mechanism().is_none() {
                return Err(WorthQueryAftermathInstallationDenial::new(
                    WorthQueryAftermathInstallationDenialKind::MechanismRequired,
                    "runtime-alone-requires-mechanism",
                ));
            }
            if has_reconciliation {
                return Err(WorthQueryAftermathInstallationDenial::new(
                    WorthQueryAftermathInstallationDenialKind::ReconciliationForbidden,
                    "runtime-alone-rejects-reconciliation",
                ));
            }
            Ok(())
        }
        DeclaredCorrectionAuthority::RuntimeWithExternalOwner => {
            if portable.mechanism().is_none() {
                return Err(WorthQueryAftermathInstallationDenial::new(
                    WorthQueryAftermathInstallationDenialKind::MechanismRequired,
                    "external-owner-requires-mechanism",
                ));
            }
            if !has_reconciliation {
                return Err(WorthQueryAftermathInstallationDenial::new(
                    WorthQueryAftermathInstallationDenialKind::ReconciliationRequired,
                    "external-owner-requires-reconciliation",
                ));
            }
            Ok(())
        }
    }
}

pub(super) fn validate_preimage_coverage(
    portable: &PortableApplicationAftermathContract,
    declared_reads: &OperationDeclaredReadFields,
) -> Result<(), WorthQueryAftermathInstallationDenial> {
    let Some(PortableCorrectionMechanism::RecordedInverse(inverse)) = portable.mechanism() else {
        return Ok(());
    };
    let demand = inverse.preimage_demand();
    if demand.loci().is_empty() && demand.maximum_encoded_bytes() == 0
        && portable.correction_binding().and_then(|binding| binding.authored_identity())
            .is_some_and(|identity| identity.parts().source_posture == worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture::Create)
    {
        return Ok(());
    }
    if demand.maximum_encoded_bytes() == 0 {
        return Err(WorthQueryAftermathInstallationDenial::new(
            WorthQueryAftermathInstallationDenialKind::PreImageDemandExceedsBound,
            "preimage-zero-bound",
        ));
    }
    // Ordered so each denial can actually fire. The whole-coverage case is
    // tested first: an operation that declares no reads at all is a different
    // installation mistake from one that declares reads and misses a slot, and
    // the installer needs to be told which. Behind the per-slot loop this arm
    // was unreachable, because the loop returns on the first uncovered slot.
    if declared_reads.field_targets().is_empty() && !demand.loci().is_empty() {
        return Err(WorthQueryAftermathInstallationDenial::new(
            WorthQueryAftermathInstallationDenialKind::MissingDeclaredReadsCoverage,
            "no-declared-reads",
        ));
    }
    for locus in demand.loci() {
        let covered = declared_reads.field_targets().iter().any(|read| {
            matches!(
                read,
                ApplicationOperationDecisionReadTarget::Field {
                    entity,
                    aspect,
                    field,
                } if entity == locus.entity()
                    && aspect == locus.aspect()
                    && field == locus.field()
            )
        });
        if !covered {
            return Err(WorthQueryAftermathInstallationDenial::new(
                WorthQueryAftermathInstallationDenialKind::PreImageDemandNotCoveredByDeclaredReads,
                format!("{}.{}.{}", locus.entity(), locus.aspect(), locus.field()),
            ));
        }
    }
    Ok(())
}

pub(super) fn escaping_effect_subject(external_effect: &InstalledExternalEffectContract) -> String {
    external_effect.correlation_family().map_or_else(
        || "unspecified-escaping-effect".into(),
        |family| family.as_str().to_owned(),
    )
}
