use crate::identity::{
    RuntimeWorldIdentityExhaustion, RuntimeWorldIdentityIssuer, RuntimeWorldOwnerIdentity,
};

/// Unforgeable capability held only during the canonical owner construction
/// call. It prevents any other crate module from resetting the identity issuer.
#[derive(Debug)]
pub(crate) struct RuntimeWorldOwnerConstructionCapability {
    _private: (),
}

impl RuntimeWorldOwnerConstructionCapability {
    fn new() -> Self {
        Self { _private: () }
    }
}

/// The serial owner-construction seam. It creates the sole issuer once and
/// does not expose a reset or an owner-identity substitution path.
#[derive(Debug)]
pub(crate) struct RuntimeWorldOwnerConstructionContract {
    issuer: RuntimeWorldIdentityIssuer,
}

impl RuntimeWorldOwnerConstructionContract {
    pub(crate) fn new() -> Result<Self, RuntimeWorldIdentityExhaustion> {
        let capability = RuntimeWorldOwnerConstructionCapability::new();
        let (issuer, _) = crate::identity::issuer_for_owner_construction(&capability)?;
        Ok(Self { issuer })
    }

    pub(crate) const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.issuer.owner()
    }

    #[cfg(test)]
    pub(crate) const fn issuer(&self) -> &RuntimeWorldIdentityIssuer {
        &self.issuer
    }

    pub(crate) fn into_issuer(self) -> RuntimeWorldIdentityIssuer {
        self.issuer
    }

    #[cfg(test)]
    pub(crate) fn issuer_mut(&mut self) -> &mut RuntimeWorldIdentityIssuer {
        &mut self.issuer
    }
}
