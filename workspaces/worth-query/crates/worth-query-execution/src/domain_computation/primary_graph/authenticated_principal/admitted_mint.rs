//! Exact framework copy admission for a freshly resolved principal proof.

use std::mem::size_of;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryPrincipalAttribute,
};

use super::{WorthQueryAuthenticatedPrincipal, WorthQueryResolvedPrincipalEvidence};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum PrincipalMintAdmissionStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>
{
    /// The existing mint is the only owner that copies the external proof.
    /// Charge its actual initialized text and Vec backing before that mint.
    pub(in crate::domain_computation::primary_graph) fn mint_admitted<Stop>(
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        evidence: WorthQueryResolvedPrincipalEvidence<PrincipalIdentity>,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, PrincipalMintAdmissionStop<Stop>> {
        use PrincipalMintAdmissionStop as Denial;
        let attributes = external.attributes();
        admit(
            u64::try_from(attributes.len()).map_err(|_| Denial::AccountingOverflow)?,
            0,
        )
        .map_err(Denial::Admission)?;
        let text = attributes.iter().try_fold(
            external
                .identity()
                .issuer()
                .len()
                .checked_add(external.identity().subject().len())
                .ok_or(Denial::AccountingOverflow)?,
            |total, attribute| {
                total
                    .checked_add(attribute.key().len())
                    .and_then(|n| n.checked_add(attribute.value().len()))
                    .ok_or(Denial::AccountingOverflow)
            },
        )?;
        let backing = attributes
            .len()
            .checked_mul(size_of::<WorthQueryPrincipalAttribute>())
            .and_then(|n| n.checked_add(text))
            .ok_or(Denial::AccountingOverflow)?;
        let work = attributes
            .len()
            .checked_add(text)
            .and_then(|n| n.checked_add(3))
            .ok_or(Denial::AccountingOverflow)?;
        admit(
            u64::try_from(work).map_err(|_| Denial::AccountingOverflow)?,
            u64::try_from(backing).map_err(|_| Denial::AccountingOverflow)?,
        )
        .map_err(Denial::Admission)?;
        Ok(Self::mint(external, evidence))
    }
}
