use worth_ui_dsl::{UiAppearanceAxisClass, UiAppearanceStateAxis};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity};

use super::{UiAppearanceCoherentBasis, UiAppearanceOwnerSnapshot, UiAppearanceStateAdapterDenial};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOperabilityAppearanceState {
    class: UiAppearanceAxisClass,
    source_class: crate::runtime::intent::UiIntentOperabilityAppearanceClass,
    owner_revision: u64,
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: UiMountedInstanceIdentity,
    node_receipt: UiMountedNodeReceiptIdentity,
    route: Box<str>,
    decision: crate::runtime::intent::UiIntentOperabilityDecision,
}

pub(crate) fn adapt(
    snapshot: &UiAppearanceOwnerSnapshot,
    basis: &UiAppearanceCoherentBasis,
) -> Result<UiOperabilityAppearanceState, UiAppearanceStateAdapterDenial> {
    let owner = snapshot
        .operability()
        .ok_or(UiAppearanceStateAdapterDenial::MissingOwner(
            UiAppearanceStateAxis::Operability,
        ))?;
    let Some(route) = basis.operability_route() else {
        return Err(UiAppearanceStateAdapterDenial::MissingSource(
            UiAppearanceStateAxis::Operability,
        ));
    };
    let fact = owner
        .fact_for(basis.graph_node(), basis.mounted_instance(), route)
        .ok_or(UiAppearanceStateAdapterDenial::MissingSource(
            UiAppearanceStateAxis::Operability,
        ))?;
    if Some(fact.node_receipt()) != basis.owner_node_receipt() {
        return Err(UiAppearanceStateAdapterDenial::StaleSource(
            UiAppearanceStateAxis::Operability,
        ));
    }
    Ok(UiOperabilityAppearanceState {
        class: map_class(fact.class()),
        source_class: fact.class(),
        owner_revision: fact.owner_revision(),
        graph_node: fact.graph_node(),
        mounted_instance: fact.mounted_instance(),
        node_receipt: fact.node_receipt(),
        route: fact.route().into(),
        decision: fact.decision().clone(),
    })
}

fn map_class(
    class: crate::runtime::intent::UiIntentOperabilityAppearanceClass,
) -> UiAppearanceAxisClass {
    match class {
        crate::runtime::intent::UiIntentOperabilityAppearanceClass::Ready => {
            UiAppearanceAxisClass::OperabilityReady
        }
        crate::runtime::intent::UiIntentOperabilityAppearanceClass::Pending => {
            UiAppearanceAxisClass::OperabilityPending
        }
        crate::runtime::intent::UiIntentOperabilityAppearanceClass::Occupied => {
            UiAppearanceAxisClass::OperabilityOccupied
        }
        crate::runtime::intent::UiIntentOperabilityAppearanceClass::Denied => {
            UiAppearanceAxisClass::OperabilityDenied
        }
        crate::runtime::intent::UiIntentOperabilityAppearanceClass::Unsupported => {
            UiAppearanceAxisClass::OperabilityUnsupported
        }
        crate::runtime::intent::UiIntentOperabilityAppearanceClass::Stale => {
            UiAppearanceAxisClass::OperabilityStale
        }
    }
}

impl UiOperabilityAppearanceState {
    pub(crate) const fn class(&self) -> UiAppearanceAxisClass {
        self.class
    }

    pub(crate) const fn source_class(
        &self,
    ) -> crate::runtime::intent::UiIntentOperabilityAppearanceClass {
        self.source_class
    }

    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }

    pub(crate) const fn mounted_instance(&self) -> UiMountedInstanceIdentity {
        self.mounted_instance
    }

    pub(crate) const fn node_receipt(&self) -> UiMountedNodeReceiptIdentity {
        self.node_receipt
    }

    pub(crate) fn route(&self) -> &str {
        &self.route
    }

    pub(crate) fn decision(&self) -> &crate::runtime::intent::UiIntentOperabilityDecision {
        &self.decision
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operability_adapter_rejects_stale_receipt_and_foreign_route() {
        let fixture = super::super::adapter_tests::fixture();
        for (receipt, route, denial) in [
            (
                UiMountedNodeReceiptIdentity::mint_unbound().unwrap(),
                "select",
                UiAppearanceStateAdapterDenial::StaleSource(UiAppearanceStateAxis::Operability),
            ),
            (
                fixture.basis.node_receipt(),
                "other-route",
                UiAppearanceStateAdapterDenial::MissingSource(UiAppearanceStateAxis::Operability),
            ),
        ] {
            let basis = UiAppearanceCoherentBasis::for_test(
                &fixture.snapshot,
                super::super::UiAppearanceStateConsumer::all_axes_for_test(
                    fixture.basis.graph_node(),
                ),
                fixture.basis.mounted_instance(),
                fixture.basis.incarnation(),
                receipt,
                fixture.basis.surface(),
                fixture.basis.presentation(),
                None,
                Some(route.into()),
            );
            assert_eq!(adapt(&fixture.snapshot, &basis), Err(denial));
        }
    }
}
