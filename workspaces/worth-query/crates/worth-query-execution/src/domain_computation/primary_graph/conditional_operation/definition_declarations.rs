//! Non-executing declaration membership is separate from temporal operation custody.
use super::installation::{route_binding::InstalledConditionalRoute, ConditionalClockLease};
use std::{collections::BTreeMap, sync::Arc};
use worth_runtime_bridge::facade::{BridgeAuthoredReadmissionDenial, BridgeFreshAuthoredBindings};

pub(in crate::domain_computation::primary_graph) struct InstalledDefinitionDeclaration {
    pub(super) identity: Arc<str>,
    pub(super) lease: Arc<ConditionalClockLease>,
    pub(super) route: InstalledConditionalRoute,
}
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct ConditionalDefinitionDeclarations {
    rows: BTreeMap<Arc<str>, InstalledDefinitionDeclaration>,
}
impl ConditionalDefinitionDeclarations {
    pub(super) fn install(&mut self, declaration: InstalledDefinitionDeclaration) {
        assert!(
            !self.rows.contains_key(&declaration.identity),
            "installation admitted unique declaration identities"
        );
        self.rows.insert(declaration.identity.clone(), declaration);
    }
    pub(in crate::domain_computation::primary_graph) fn route(
        &self,
        identity: &str,
        lease: &Arc<ConditionalClockLease>,
    ) -> Option<&InstalledConditionalRoute> {
        self.rows
            .get(identity)
            .filter(|row| Arc::ptr_eq(&row.lease, lease))
            .map(|row| &row.route)
    }
    pub(in crate::domain_computation::primary_graph) fn admit_authored_declarations(
        &mut self,
        fresh: &BridgeFreshAuthoredBindings,
    ) -> Result<(), BridgeAuthoredReadmissionDenial> {
        for row in self.rows.values_mut() {
            row.route.admit_authored_declaration(fresh)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn clear(&mut self) {
        self.rows.clear();
    }
}
