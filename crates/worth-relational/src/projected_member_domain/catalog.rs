//! Atomic domain registration and exact selected-registry admission.
use super::RegisteredProjectedMemberDomain;
use super::{ProjectedMemberDomainDenial as Denial, ProjectedMemberDomainHandle as Handle};
use super::{ProjectedMemberDomainIdentity as Identity, ProjectedMemberDomainParts as Parts};
use crate::schema::data::RelationalSchemaRegistry;
use std::sync::Arc;

impl RelationalSchemaRegistry {
    /// Validate the receiving carrier and cases before retaining an original
    /// domain issuer. Newly declared domains use ordinary registration.
    pub fn register_projected_member_domains_from_original(
        self,
        additions: Vec<Parts>,
        original: &Self,
    ) -> Result<Self, Denial> {
        let identities = additions
            .iter()
            .map(|parts| parts.identity.clone())
            .collect::<Vec<_>>();
        let mut registry = self.register_projected_member_domains(additions)?;
        for identity in identities {
            if let Some(entry) = original.projected_member_domains.get(&identity) {
                if registry.projected_member_domains[&identity].parts != entry.parts {
                    return Err(Denial::ForeignDomain);
                }
                registry
                    .projected_member_domains
                    .insert(identity, entry.clone());
            }
        }
        Ok(registry)
    }

    pub fn register_projected_member_domains(
        mut self,
        additions: Vec<Parts>,
    ) -> Result<Self, Denial> {
        for parts in additions {
            if !valid_name(&parts.identity.name) || !valid_name(&parts.identity.schema.0) {
                return Err(Denial::InvalidIdentity);
            }
            let carrier = self
                .entity_registration(parts.carrier)
                .map_err(|_| Denial::InvalidCarrier(parts.carrier))?;
            if carrier.schema_id != parts.identity.schema
                || carrier.schema_version_id != parts.identity.version
            {
                return Err(Denial::InvalidCarrier(parts.carrier));
            }
            if parts.cases.is_empty() {
                return Err(Denial::EmptyCases);
            }
            if parts.cases.iter().any(|case| !valid_name(case))
                || parts.cases.windows(2).any(|pair| pair[0] >= pair[1])
            {
                return Err(Denial::NoncanonicalCases);
            }
            if self.projected_member_domains.contains_key(&parts.identity) {
                return Err(Denial::DuplicateDomain(parts.identity));
            }
            self.projected_member_domains.insert(
                parts.identity.clone(),
                Arc::new(RegisteredProjectedMemberDomain { parts }),
            );
        }
        Ok(self)
    }

    pub fn projected_member_domain(&self, identity: &Identity) -> Option<Handle> {
        self.projected_member_domains
            .get(identity)
            .map(|entry| Handle {
                entry: Arc::clone(entry),
            })
    }

    pub(crate) fn admits_projected_member_domain(&self, handle: &Handle) -> bool {
        self.projected_member_domains
            .get(&handle.parts().identity)
            .is_some_and(|entry| Arc::ptr_eq(entry, &handle.entry))
    }

    pub(crate) fn projected_domain_parts(&self) -> Vec<Parts> {
        self.projected_member_domains
            .values()
            .map(|entry| entry.parts.clone())
            .collect()
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.trim() == name
}
