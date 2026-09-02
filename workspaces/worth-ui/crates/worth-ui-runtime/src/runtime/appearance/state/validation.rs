use worth_ui_dsl::{UiAppearanceAxisClass, UiAppearanceStateAxis};
use worth_ui_host_contract::UiMountedNodeReceiptIdentity;

use super::{UiAppearanceCoherentBasis, UiAppearanceOwnerSnapshot, UiAppearanceStateAdapterDenial};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiValidationAppearanceState {
    class: UiAppearanceAxisClass,
    source_class: Option<crate::runtime::intent::UiValidationAppearanceClass>,
    owner_revision: u64,
    fact_identity: Option<u64>,
    fact_revision: Option<u64>,
    node_receipt: Option<UiMountedNodeReceiptIdentity>,
}

pub(crate) fn adapt(
    snapshot: &UiAppearanceOwnerSnapshot,
    basis: &UiAppearanceCoherentBasis,
) -> Result<UiValidationAppearanceState, UiAppearanceStateAdapterDenial> {
    let owner = snapshot
        .validation()
        .ok_or(UiAppearanceStateAdapterDenial::MissingOwner(
            UiAppearanceStateAxis::Validation,
        ))?;
    let source = owner.fact_basis_for(basis.graph_node(), basis.mounted_instance());
    let source_class = owner.class_for(
        basis.graph_node(),
        basis.mounted_instance(),
        basis.node_receipt(),
    );
    let (fact_identity, fact_revision, node_receipt) = source
        .map_or((None, None, None), |(identity, revision, receipt)| {
            (Some(identity), Some(revision), Some(receipt))
        });
    Ok(UiValidationAppearanceState {
        class: source_class.map_or(UiAppearanceAxisClass::ValidationUnspecified, map_class),
        source_class,
        owner_revision: owner.owner_revision(),
        fact_identity,
        fact_revision,
        node_receipt,
    })
}

fn map_class(class: crate::runtime::intent::UiValidationAppearanceClass) -> UiAppearanceAxisClass {
    match class {
        crate::runtime::intent::UiValidationAppearanceClass::Valid => {
            UiAppearanceAxisClass::ValidationValid
        }
        crate::runtime::intent::UiValidationAppearanceClass::Advisory => {
            UiAppearanceAxisClass::ValidationAdvisory
        }
        crate::runtime::intent::UiValidationAppearanceClass::Invalid => {
            UiAppearanceAxisClass::ValidationInvalid
        }
        crate::runtime::intent::UiValidationAppearanceClass::Pending => {
            UiAppearanceAxisClass::ValidationPending
        }
        crate::runtime::intent::UiValidationAppearanceClass::Stale => {
            UiAppearanceAxisClass::ValidationStale
        }
    }
}

impl UiValidationAppearanceState {
    pub(crate) const fn class(&self) -> UiAppearanceAxisClass {
        self.class
    }

    pub(crate) const fn source_class(
        &self,
    ) -> Option<crate::runtime::intent::UiValidationAppearanceClass> {
        self.source_class
    }

    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) const fn fact_identity(&self) -> Option<u64> {
        self.fact_identity
    }

    pub(crate) const fn fact_revision(&self) -> Option<u64> {
        self.fact_revision
    }

    pub(crate) const fn node_receipt(&self) -> Option<UiMountedNodeReceiptIdentity> {
        self.node_receipt
    }
}
