//! Atomic domain registration and exact selected-registry admission.
use super::RegisteredProjectedMemberDomain;
use super::{ProjectedMemberDomainDenial as Denial, ProjectedMemberDomainHandle as Handle};
use super::{ProjectedMemberDomainIdentity as Identity, ProjectedMemberDomainParts as Parts};
use crate::schema::data::RelationalSchemaRegistry;
use std::sync::Arc;

impl RelationalSchemaRegistry {
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
