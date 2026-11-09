use std::collections::BTreeMap;

use worth_foundational::facade::{
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationPath, ApplicationSchemaMember,
};

use super::authorization_path_artifact::prepare_authorization_path_identity;
use crate::canonical_digest_derivation::InstallationCanonicalIdentityBasis;
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledAuthorizationPath {
    path: ApplicationAuthorizationPath,
    identity: CanonicalDigestId,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryInstalledAuthorizationPath {
    pub fn path(&self) -> &ApplicationAuthorizationPath {
        &self.path
    }

    pub const fn identity(&self) -> &CanonicalDigestId {
        &self.identity
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledAbilityRequirement {
    identity: CanonicalDigestId,
    ability: String,
    scope_entity: String,
    policy: String,
    policy_paths: Vec<WorthQueryInstalledAuthorizationPath>,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryInstalledAbilityRequirement {
    pub(crate) fn new(
        ability: String,
        scope_entity: String,
        policy: String,
        policy_paths: Vec<ApplicationAuthorizationPath>,
        budget: CanonicalDigestWorkBudget,
    ) -> Result<Self, CanonicalDigestDerivationDenial> {
        let mut installed_paths = Vec::with_capacity(policy_paths.len());
        let mut canonical_work = WorthQueryCanonicalWorkEvidence::zero();
        for path in policy_paths {
            let prepared = prepare_authorization_path_identity(&path, budget)?;
            canonical_work = canonical_work.combine(prepared.work);
            installed_paths.push(WorthQueryInstalledAuthorizationPath {
                path,
                identity: prepared.digest,
                canonical_work: prepared.work,
            });
        }
        let mut basis = InstallationCanonicalIdentityBasis::new(
            "worth-query.application-authorization-policy",
            "worth-query-application-authorization-policy-v1",
            budget,
        );
        basis.text("ability", &ability)?;
        basis.text("scope-entity", &scope_entity)?;
        basis.text("policy", &policy)?;
        basis.unsigned_usize("path-count", installed_paths.len())?;
        for (index, path) in installed_paths.iter().enumerate() {
            basis.digest(format!("path[{index}]"), *path.identity())?;
        }
        let (identity, identity_work) = basis.derive()?;
        Ok(Self {
            identity,
            ability,
            scope_entity,
            policy,
            policy_paths: installed_paths,
            canonical_work: canonical_work.combine(identity_work),
        })
    }

    pub const fn identity(&self) -> &CanonicalDigestId {
        &self.identity
    }

    pub fn ability(&self) -> &str {
        &self.ability
    }

    pub fn scope_entity(&self) -> &str {
        &self.scope_entity
    }

    pub fn policy(&self) -> &str {
        &self.policy
    }

    pub fn policy_paths(&self) -> &[WorthQueryInstalledAuthorizationPath] {
        &self.policy_paths
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }
}

pub(crate) type ApplicationAuthorizationPolicyRegistry =
    BTreeMap<String, BTreeMap<String, WorthQueryInstalledAbilityRequirement>>;

pub(crate) fn compile_authorization_policy_registry(
    members: &[ApplicationSchemaMember],
    budget: CanonicalDigestWorkBudget,
) -> Result<ApplicationAuthorizationPolicyRegistry, CanonicalDigestDerivationDenial> {
    let mut registry = ApplicationAuthorizationPolicyRegistry::new();
    for member in members {
        if let ApplicationSchemaMember::AbilityPolicy {
            ability,
            scope_entity,
            policy,
            paths,
        } = member
        {
            let requirement = WorthQueryInstalledAbilityRequirement::new(
                ability.clone(),
                scope_entity.clone(),
                policy.clone(),
                paths.clone(),
                budget,
            )?;
            registry
                .entry(ability.clone())
                .or_default()
                .insert(scope_entity.clone(), requirement);
        }
    }
    Ok(registry)
}
