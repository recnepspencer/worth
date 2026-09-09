use super::{UiSelectionOwnerIdentity, UiSelectionOwnerIncarnation, UiSelectionStableKey};

/// Read-only translation of current collection evidence into Selection's own
/// vocabulary. It neither selects an item nor installs a mounted relationship.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiSelectionProjectionMapping {
    pub(crate) owner: UiSelectionOwnerIdentity,
    pub(crate) incarnation: UiSelectionOwnerIncarnation,
    pub(crate) key: UiSelectionStableKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSelectionProjectionMappingDenial {
    CollectionNotCurrent,
    OptionRevisionChanged,
    ApplicationKeyUnavailable,
    IncarnationUnavailable,
}

impl UiSelectionProjectionMapping {
    pub(crate) fn from_current_option(
        owner: &crate::mounting::UiMountedIdentityBasis,
        collection: &worth_ui_query_binding::UiCollectionProjectionInputFact,
        option: &worth_ui_query_binding::UiProjectionOptionReference,
    ) -> Result<Self, UiSelectionProjectionMappingDenial> {
        if collection.posture() != worth_ui_query_binding::UiProjectionInputPosture::Current {
            return Err(UiSelectionProjectionMappingDenial::CollectionNotCurrent);
        }
        if collection.revision() != option.owner_revision() {
            return Err(UiSelectionProjectionMappingDenial::OptionRevisionChanged);
        }
        let value = option
            .application_item_key()
            .ok_or(UiSelectionProjectionMappingDenial::ApplicationKeyUnavailable)?;
        let family = crate::runtime::UiApplicationItemKeyFamily::from_projection_input(
            collection.revision().slot(),
        );
        Ok(Self {
            owner: UiSelectionOwnerIdentity::new(
                owner.semantic_surface_identity(),
                owner.graph_node_identity(),
                family,
            ),
            incarnation: UiSelectionOwnerIncarnation::new(
                owner.mount_incarnation().diagnostic_value(),
            )
            .ok_or(UiSelectionProjectionMappingDenial::IncarnationUnavailable)?,
            key: UiSelectionStableKey::new(
                crate::runtime::UiApplicationItemKey::from_projection_mapping(family, value),
            ),
        })
    }

    pub(crate) fn registration(
        self,
        collection: &worth_ui_query_binding::UiCollectionProjectionInputFact,
        selection: &super::UiSelectionRuntimeState,
    ) -> Result<Option<super::UiSelectionRegistration>, super::UiSelectionRequestDenial> {
        let revision = collection.revision().observation_order();
        if selection.catalog_is_current(self.owner, self.incarnation, revision) {
            return Ok(None);
        }
        let keys = collection
            .current_application_item_keys()
            .ok_or(super::UiSelectionRequestDenial::CatalogUnavailable)?;
        let catalog = keys
            .iter()
            .copied()
            .map(|value| {
                UiSelectionStableKey::new(
                    crate::runtime::UiApplicationItemKey::from_projection_mapping(
                        self.owner.key_family(),
                        value,
                    ),
                )
            })
            .collect();
        let posture = match collection.completeness() {
            Some(worth_ui_query_binding::UiCollectionCompleteness::Complete) => {
                super::UiSelectionCatalogPosture::Complete
            }
            Some(worth_ui_query_binding::UiCollectionCompleteness::Partial) => {
                super::UiSelectionCatalogPosture::Partial
            }
            None => return Err(super::UiSelectionRequestDenial::CatalogUnavailable),
        };
        super::UiSelectionRegistration::new(
            self.owner,
            self.incarnation,
            selection.default_owner_policy(),
            catalog,
            posture,
        )
        .map(|registration| Some(registration.with_catalog_revision(revision)))
    }
}
