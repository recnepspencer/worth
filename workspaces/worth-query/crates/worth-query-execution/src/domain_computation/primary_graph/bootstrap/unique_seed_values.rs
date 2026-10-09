//! Fresh unique-field seeds admit each value at most once. The fresh graph
//! holds only program activation and this installation's seeds, so the values
//! already admitted are the whole lookup. Non-unique migration writes do not
//! use this inventory. The checkpoint_transition/authoring.rs migration writer
//! calls typed_bootstrap.rs::bind_entity while publication.rs::prepare_transition
//! still holds recovered authority; its non-unique rows bypass fresh seeding.

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
        let unique = self.graph.layout.unique_fields();
        let mut admitted = Vec::new();
        for (kind, locator, value) in writes {
            if unique.index(kind, locator).is_none() {
                continue;
            }
            assert!(
                self.recovered_relational_authority.is_none(),
                "unique seed values require a fresh graph"
            );
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
