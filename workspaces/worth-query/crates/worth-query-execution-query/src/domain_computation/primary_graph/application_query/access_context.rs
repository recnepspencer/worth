use super::super::{WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal};

/// Exact mapped identity and typed application scope presented for one query
/// admission.
///
/// This context is descriptive input. The execution runtime must freshly
/// validate both sealed proofs before it can mint a query plan.
pub struct WorthQueryApplicationQueryAccessContext<'a, Schema, Principal, PrincipalIdentity, Scope>
{
    principal: &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
}

impl<'a, Schema, Principal, PrincipalIdentity, Scope>
    WorthQueryApplicationQueryAccessContext<'a, Schema, Principal, PrincipalIdentity, Scope>
{
    pub fn new(
        principal: &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    ) -> Self {
        Self { principal, scope }
    }

    pub fn principal(
        &self,
    ) -> &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity> {
        self.principal
    }

    pub fn scope(&self) -> &'a WorthQueryApplicationEntityIdentity<Schema, Scope> {
        self.scope
    }
}
