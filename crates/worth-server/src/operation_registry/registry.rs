use std::{collections::BTreeMap, sync::Arc};

use crate::{
    diagnostics::WorthServerCounters, WorthServerOperationAuthorityMetadata,
    WorthServerOperationAuthorizationPolicy, WorthServerOperationCapabilities,
    WorthServerOperationDenial, WorthServerOperationFamily, WorthServerOperationInventory,
    WorthServerOperationInventoryRow, WorthServerOperationRegistration,
    WorthServerOperationRequest, WorthServerSurfaceFamily,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationRegistry {
    registrations_by_family:
        Arc<BTreeMap<WorthServerOperationFamily, WorthServerOperationRegistration>>,
}

impl WorthServerOperationRegistry {
    pub fn build(
        registrations: Vec<WorthServerOperationRegistration>,
        counters: &WorthServerCounters,
    ) -> Result<Self, WorthServerOperationRegistryError> {
        let mut registrations_by_family = BTreeMap::new();

        for registration in registrations {
            let family = registration.family();
            if let Err(detail) = registration.validate() {
                return Err(WorthServerOperationRegistryError::InvalidRegistration {
                    family,
                    detail,
                });
            }
            if registrations_by_family
                .insert(family, registration)
                .is_some()
            {
                counters.increment_rejected_duplicate_operation_registrations();
                return Err(WorthServerOperationRegistryError::DuplicateOperationFamily { family });
            }
        }

        counters.record_registered_operation_families(registrations_by_family.len());

        Ok(Self {
            registrations_by_family: Arc::new(registrations_by_family),
        })
    }

    pub fn inventory(&self) -> WorthServerOperationInventory {
        WorthServerOperationInventory::new(
            self.registrations_by_family
                .values()
                .map(|registration| {
                    WorthServerOperationInventoryRow::new(
                        registration.family(),
                        registration.is_enabled(),
                        registration.exposed_surfaces().to_vec(),
                    )
                })
                .collect(),
        )
    }

    pub fn capabilities_for(
        &self,
        family: WorthServerOperationFamily,
    ) -> WorthServerOperationCapabilities {
        self.registrations_by_family
            .get(&family)
            .map(|registration| {
                if registration.is_enabled() {
                    WorthServerOperationCapabilities::enabled(
                        family,
                        registration.exposed_surfaces().to_vec(),
                    )
                } else {
                    WorthServerOperationCapabilities::disabled(
                        family,
                        registration.exposed_surfaces().to_vec(),
                    )
                }
            })
            .unwrap_or_else(|| WorthServerOperationCapabilities::absent(family))
    }

    pub fn declared_authority_for(
        &self,
        operation_request: &WorthServerOperationRequest,
    ) -> Result<WorthServerOperationAuthorityMetadata, String> {
        let family = operation_request.identity().operation_family();
        let Some(registration) = self.registrations_by_family.get(&family) else {
            return Err(format!(
                "operation family `{}` has no registered authority declaration",
                family.as_str()
            ));
        };
        let Some(declaration) = registration.authority_declaration() else {
            return Err(format!(
                "operation family `{}` has no declared authority template",
                family.as_str()
            ));
        };
        declaration.lower(operation_request)
    }

    pub fn authorization_policy_for(
        &self,
        family: WorthServerOperationFamily,
    ) -> Option<&WorthServerOperationAuthorizationPolicy> {
        self.registrations_by_family
            .get(&family)
            .and_then(|registration| registration.authorization_policy())
    }

    pub fn admit(
        &self,
        surface_family: WorthServerSurfaceFamily,
        family: WorthServerOperationFamily,
    ) -> Result<WorthServerOperationCapabilities, WorthServerOperationDenial> {
        let capabilities = self.capabilities_for(family);
        if capabilities.is_absent() {
            return Err(WorthServerOperationDenial::UnregisteredFamily {
                family,
                surface_family,
            });
        }
        if capabilities.is_disabled() {
            return Err(WorthServerOperationDenial::DisabledFamily {
                family,
                surface_family,
            });
        }
        if !capabilities.exposed_surfaces().contains(&surface_family) {
            return Err(WorthServerOperationDenial::SurfaceFamilyNotExposed {
                family,
                surface_family,
            });
        }
        Ok(capabilities)
    }

    pub fn admit_operation_name(
        &self,
        family: WorthServerOperationFamily,
        operation_name: &str,
    ) -> Result<(), WorthServerOperationDenial> {
        let Some(registration) = self.registrations_by_family.get(&family) else {
            return Ok(());
        };
        if registration.admitted_operation_names().is_empty() {
            return Ok(());
        }
        let canonical_operation_name = operation_name.trim().to_ascii_lowercase();
        if registration
            .admitted_operation_names()
            .iter()
            .any(|candidate| candidate == &canonical_operation_name)
        {
            return Ok(());
        }
        Err(WorthServerOperationDenial::UnknownOperationName {
            family,
            operation_name: canonical_operation_name,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerOperationRegistryError {
    DuplicateOperationFamily {
        family: WorthServerOperationFamily,
    },
    InvalidRegistration {
        family: WorthServerOperationFamily,
        detail: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::WorthServerCounters;

    #[test]
    fn duplicate_operation_family_rejection_increments_narrow_counter() {
        let counters = WorthServerCounters::default();
        let result = WorthServerOperationRegistry::build(
            vec![
                WorthServerOperationRegistration::disabled(
                    WorthServerOperationFamily::QueryDirectRead,
                ),
                WorthServerOperationRegistration::disabled(
                    WorthServerOperationFamily::QueryDirectRead,
                ),
            ],
            &counters,
        );

        assert_eq!(
            result,
            Err(
                WorthServerOperationRegistryError::DuplicateOperationFamily {
                    family: WorthServerOperationFamily::QueryDirectRead,
                },
            )
        );
        assert_eq!(
            counters
                .snapshot()
                .rejected_duplicate_operation_registrations,
            1
        );
        assert_eq!(counters.snapshot().registered_operation_families, 0);
    }
}
