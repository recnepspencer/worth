//! Seeds obey the unique law: one installation writes each value of a unique
//! field at most once. Seeds land only in a fresh graph: Relational refuses
//! initial schema installation once its runtime has committed, and a
//! recovered graph never seeds (asserted below). A fresh graph holds only
//! Query's program activation, which writes no application field, and this
//! installation's own seeds, which are tracked here, so the values already
//! admitted are the whole lookup.

use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, AspectFieldLocator, AspectValue,
    CanonicalAspectValueIdentityBasis,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentityBinding, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_installation::facade::{ApplicationScalarValueBinding, ApplicationSchema};
use worth_relational::facade::identity::KindId;

use super::{
    primary_graph_denial, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
    WorthQueryPrincipalBootstrapRow,
};

/// One admitted value of one unique field.
pub(in crate::domain_computation::primary_graph) type WorthQueryUniqueSeedValue = (
    KindId,
    AspectFieldLocator,
    CanonicalAspectValueIdentityBasis,
);

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    /// Admits every unique value one seed row writes, or none of them.
    pub(in crate::domain_computation::primary_graph) fn admit_unique_seed_values<'row>(
        &mut self,
        writes: impl IntoIterator<Item = (KindId, &'row AspectFieldLocator, &'row AspectValue)>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        assert!(
            self.recovered_relational_authority.is_none(),
            "seeds land only in a fresh graph; a recovered graph never seeds"
        );
        let unique = self.graph.layout.unique_fields();
        let mut admitted = Vec::new();
        for (kind, locator, value) in writes {
            if unique.index(kind, locator).is_none() {
                continue;
            }
            let seed = (
                kind,
                locator.clone(),
                prepare_aspect_value_identity_basis(value),
            );
            if self.unique_seed_values.contains(&seed) || admitted.contains(&seed) {
                return Err(primary_graph_denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::DuplicateUniqueSeedValue,
                    format!("{locator:?}"),
                ));
            }
            admitted.push(seed);
        }
        self.unique_seed_values.extend(admitted);
        Ok(())
    }

    /// Admits the unique values one principal row writes: the principal's
    /// identity and the mapping's identity and status.
    pub(super) fn admit_principal_unique_values(
        &mut self,
        row: &WorthQueryPrincipalBootstrapRow,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let encoding_rejected = |denial| {
            primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::BindingSchemaMismatch,
                format!("principal mapping encoding was rejected: {denial:?}"),
            )
        };
        let identity = WorthQueryExternalPrincipalIdentityBinding::encode(&row.identity)
            .map_err(encoding_rejected)?;
        let status = WorthQueryPrincipalMappingStatusBinding::encode(&row.status)
            .map_err(encoding_rejected)?;
        let layout = &row.layout;
        self.admit_unique_seed_values([
            (
                layout.principal_kind,
                &layout.principal_identity_locator,
                &row.principal_identity,
            ),
            (layout.mapping_kind, &layout.identity_locator, &identity),
            (layout.mapping_kind, &layout.status_locator, &status),
        ])
    }
}
