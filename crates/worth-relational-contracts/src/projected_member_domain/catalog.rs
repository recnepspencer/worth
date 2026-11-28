//! Atomic domain registration and exact selected-registry admission.
use super::RegisteredProjectedMemberDomain;
use super::{ProjectedMemberDomainDenial as Denial, ProjectedMemberDomainHandle as Handle};
use super::{ProjectedMemberDomainIdentity as Identity, ProjectedMemberDomainParts as Parts};
use crate::schema::data::RelationalSchemaRegistry;
use std::sync::Arc;
use worth_foundational::{is_valid_role_identifier, is_valid_simple_identifier};

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
            if !is_valid_simple_identifier(&parts.identity.name)
                || !is_valid_role_identifier(&parts.identity.schema.0)
            {
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
            if parts
                .cases
                .iter()
                .any(|case| !is_valid_simple_identifier(case))
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

    pub fn admits_projected_member_domain(&self, handle: &Handle) -> bool {
        self.projected_member_domains
            .get(&handle.parts().identity)
            .is_some_and(|entry| Arc::ptr_eq(entry, &handle.entry))
    }

    pub fn projected_domain_parts(&self) -> Vec<Parts> {
        self.projected_member_domains
            .values()
            .map(|entry| entry.parts.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::data::KindId;
    use crate::schema::data::{EntityKindRegistration, SchemaId, SchemaVersionId};

    #[test]
    fn registration_requires_simple_domain_and_case_names() {
        let schema = SchemaId("example.schema".into());
        let version = SchemaVersionId(1);
        let carrier = KindId(1);
        let registry = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: carrier,
                kind_name: "carrier".into(),
                schema_id: schema.clone(),
                schema_version_id: version,
                aspect_contract_declarations: Default::default(),
            })
            .unwrap();
        let valid = Parts {
            identity: Identity {
                schema,
                version,
                name: "domain".into(),
            },
            carrier,
            cases: vec!["case".into()],
        };
        for invalid in ["a b", "a\tb", "a\nb", "a\0b", "a.b"] {
            let mut parts = valid.clone();
            parts.identity.name = invalid.into();
            assert_eq!(
                registry
                    .clone()
                    .register_projected_member_domains(vec![parts]),
                Err(Denial::InvalidIdentity),
            );
            let mut parts = valid.clone();
            parts.cases = vec![invalid.into()];
            assert_eq!(
                registry
                    .clone()
                    .register_projected_member_domains(vec![parts]),
                Err(Denial::NoncanonicalCases),
            );
        }
        for invalid in [".schema", "schema.", "schema..name", "bad schema"] {
            let mut parts = valid.clone();
            parts.identity.schema = SchemaId(invalid.into());
            assert_eq!(
                registry
                    .clone()
                    .register_projected_member_domains(vec![parts]),
                Err(Denial::InvalidIdentity),
            );
        }
        let registry = registry
            .register_projected_member_domains(vec![valid.clone()])
            .unwrap();
        assert!(registry
            .projected_member_domain(&valid.identity)
            .unwrap()
            .case("case")
            .is_some());
    }
}
