//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryIssuedSelectedPrincipal<'_, '_, Schema, Principal, PrincipalIdentity>
{
    #[doc(hidden)]
    pub fn principal_for_test(
        &self,
    ) -> &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity> {
        Self::principal(self)
    }
}
