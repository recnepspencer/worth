use super::{AdmittedLayoutMaterialization, MaterializationDenial};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentMaterializationFrontier {
    source: super::LayoutMaterializationSourceIdentity,
}

impl CurrentMaterializationFrontier {
    pub(crate) fn from_catalog(catalog: &crate::BootstrapCatalogReadAdmission) -> Self {
        Self {
            source: super::LayoutMaterializationSourceIdentity::from_catalog(catalog),
        }
    }

    pub(crate) fn from_btree_source(
        catalog: &crate::BootstrapCatalogReadAdmission,
        source: &crate::BaselineBTreeReadSource,
    ) -> Self {
        Self {
            source: super::LayoutMaterializationSourceIdentity::from_btree_lookup_source(
                catalog, source,
            ),
        }
    }

    pub(crate) fn from_lsm_lookup_source(
        catalog: &crate::BootstrapCatalogReadAdmission,
        source: &crate::strategy::BaselineLsmLookupSource,
    ) -> Self {
        Self {
            source: super::LayoutMaterializationSourceIdentity::from_lsm_lookup_source(
                catalog, source,
            ),
        }
    }

    pub(crate) fn from_lsm_replay_source(
        catalog: &crate::BootstrapCatalogReadAdmission,
        source: &worth_store_lsm_authority::AdmittedLsmReplaySource,
    ) -> Result<Self, MaterializationDenial> {
        Ok(Self {
            source: super::LayoutMaterializationSourceIdentity::from_lsm_replay_source(
                catalog, source,
            )?,
        })
    }

    pub const fn source(&self) -> &super::LayoutMaterializationSourceIdentity {
        &self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentLayoutMaterialization {
    materialization: AdmittedLayoutMaterialization,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleLayoutMaterialization {
    inner: std::sync::Arc<StaleLayoutMaterializationData>,
}

#[derive(Debug, PartialEq, Eq)]
struct StaleLayoutMaterializationData {
    materialization: AdmittedLayoutMaterialization,
    observed_frontier: CurrentMaterializationFrontier,
}

impl StaleLayoutMaterialization {
    fn new(
        materialization: AdmittedLayoutMaterialization,
        observed_frontier: CurrentMaterializationFrontier,
    ) -> Self {
        Self {
            inner: std::sync::Arc::new(StaleLayoutMaterializationData {
                materialization,
                observed_frontier,
            }),
        }
    }

    pub fn materialization(&self) -> &AdmittedLayoutMaterialization {
        &self.inner.materialization
    }

    pub fn observed_frontier(&self) -> &CurrentMaterializationFrontier {
        &self.inner.observed_frontier
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterializationFreshness {
    Current(CurrentLayoutMaterialization),
    Stale(StaleLayoutMaterialization),
}

impl CurrentLayoutMaterialization {
    pub(crate) fn from_btree_replay_source(
        source: &crate::AdmittedBTreeReplaySource<crate::BaselineBTreeReplayAdmission>,
    ) -> Result<Self, MaterializationDenial> {
        let materialization = source.intent().materialization().clone();
        materialization.coverage().require_exact()?;
        if materialization.source().kind()
            != super::LayoutMaterializationSourceKind::BTreeRoot(source.root_reference())
            || !materialization
                .source()
                .matches_btree_replay_source(source.physical_source())
        {
            return Err(MaterializationDenial::CoverageSourceMismatch);
        }
        Ok(Self { materialization })
    }

    pub(super) fn classify_at(
        materialization: AdmittedLayoutMaterialization,
        frontier: CurrentMaterializationFrontier,
    ) -> Result<MaterializationFreshness, MaterializationDenial> {
        materialization.coverage().require_exact()?;
        if materialization.source() != frontier.source() {
            return Ok(MaterializationFreshness::Stale(
                StaleLayoutMaterialization::new(materialization, frontier),
            ));
        }
        Ok(MaterializationFreshness::Current(Self { materialization }))
    }

    pub const fn materialization(&self) -> &AdmittedLayoutMaterialization {
        &self.materialization
    }
}
