//! Non-executing declaration membership is separate from temporal operation custody.
use super::installation::{route_binding::InstalledConditionalRoute, ConditionalClockLease};
use std::{collections::BTreeMap, sync::Arc};
use worth_runtime_bridge::facade::{BridgeAuthoredReadmissionDenial, BridgeFreshAuthoredBindings};

pub struct InstalledDefinitionDeclaration {
    pub identity: Arc<str>,
    pub lease: Arc<ConditionalClockLease>,
    pub route: InstalledConditionalRoute,
}
#[derive(Default)]
pub struct ConditionalDefinitionDeclarations {
    rows: BTreeMap<Arc<str>, InstalledDefinitionDeclaration>,
}
impl ConditionalDefinitionDeclarations {
    pub fn install_readiness(
        &mut self,
        identity: Arc<str>,
        lease: Arc<ConditionalClockLease>,
        route: InstalledConditionalRoute,
    ) {
        self.install(InstalledDefinitionDeclaration {
            identity,
            lease,
            route,
        });
    }
    pub fn install(&mut self, declaration: InstalledDefinitionDeclaration) {
        assert!(
            !self.rows.contains_key(&declaration.identity),
            "installation admitted unique declaration identities"
        );
        self.rows.insert(declaration.identity.clone(), declaration);
    }
    pub fn route(
        &self,
        identity: &str,
        lease: &Arc<ConditionalClockLease>,
    ) -> Option<&InstalledConditionalRoute> {
        self.rows
            .get(identity)
            .filter(|row| Arc::ptr_eq(&row.lease, lease))
            .map(|row| &row.route)
    }
    pub fn admit_authored_declarations(
        &mut self,
        fresh: &BridgeFreshAuthoredBindings,
    ) -> Result<(), BridgeAuthoredReadmissionDenial> {
        for row in self.rows.values_mut() {
            row.route.admit_authored_declaration(fresh)?;
        }
        Ok(())
    }
    pub fn clear(&mut self) {
        self.rows.clear();
    }
}
