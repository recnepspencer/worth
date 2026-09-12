use crate::runtime::{
    RelationalRuntime, RelationalRuntimeConfigurationSnapshot, RuntimeSubsystem,
    SchemaContractRuntimeSubsystem,
};
use crate::schema::data::{RelationalSchemaRegistry, SchemaRegistryError};
use crate::validation::{data::CustomInvariantRegistration, FrozenCustomInvariantRegistry};
use std::sync::Arc;

/// Move-only authority to extend an uncommitted runtime's initial schema.
#[derive(Debug)]
pub struct RelationalInitialSchemaInstallation<'runtime> {
    runtime: &'runtime mut RelationalRuntime,
    #[cfg(test)]
    transition_pause: Option<(std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalInitialSchemaInstallationDenialKind {
    RuntimeAlreadyCommitted,
    InitialInvariantsAlreadySealed,
    DuplicateCustomInvariant,
    SchemaRejected,
    BranchTransitionRejected,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    RetentionOwnerUnavailable,
    RetentionRootSetTooLarge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalInitialSchemaInstallationDenial {
    kind: RelationalInitialSchemaInstallationDenialKind,
    detail: String,
}

impl RelationalInitialSchemaInstallationDenial {
    fn new(kind: RelationalInitialSchemaInstallationDenialKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub const fn kind(&self) -> RelationalInitialSchemaInstallationDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for RelationalInitialSchemaInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "initial Relational schema installation denied: {:?} ({})",
            self.kind, self.detail
        )
    }
}

impl std::error::Error for RelationalInitialSchemaInstallationDenial {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalInitialSchemaInstallationReceipt {
    runtime_instance_id: u64,
    custom_invariant_generation: u64,
    custom_invariant_inventory_digest: [u8; 32],
    retained_entity_kind_count: usize,
    retained_relation_kind_count: usize,
}

impl RelationalInitialSchemaInstallationReceipt {
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }
    pub const fn custom_invariant_generation(&self) -> u64 {
        self.custom_invariant_generation
    }
    pub const fn custom_invariant_inventory_digest(&self) -> &[u8; 32] {
        &self.custom_invariant_inventory_digest
    }
    pub const fn retained_entity_kind_count(&self) -> usize {
        self.retained_entity_kind_count
    }

    pub const fn retained_relation_kind_count(&self) -> usize {
        self.retained_relation_kind_count
    }
}

impl RelationalRuntime {
    pub fn prepare_initial_schema_installation(
        &mut self,
    ) -> Result<RelationalInitialSchemaInstallation<'_>, RelationalInitialSchemaInstallationDenial>
    {
        if self.history().latest_commit().is_some() || self.history.has_published_branch_basis() {
            return Err(RelationalInitialSchemaInstallationDenial::new(
                RelationalInitialSchemaInstallationDenialKind::RuntimeAlreadyCommitted,
                "initial schema authority closes after the first committed mutation",
            ));
        }
        Ok(RelationalInitialSchemaInstallation {
            runtime: self,
            #[cfg(test)]
            transition_pause: None,
        })
    }
}

impl RelationalInitialSchemaInstallation<'_> {
    pub fn install(
        self,
        additions: RelationalSchemaRegistry,
    ) -> Result<RelationalInitialSchemaInstallationReceipt, RelationalInitialSchemaInstallationDenial>
    {
        self.install_with_custom_invariants(additions, Vec::new())
    }

    pub fn install_with_custom_invariants(
        self,
        additions: RelationalSchemaRegistry,
        custom_invariants: Vec<CustomInvariantRegistration>,
    ) -> Result<RelationalInitialSchemaInstallationReceipt, RelationalInitialSchemaInstallationDenial>
    {
        let configuration = self.runtime.configuration_binding();
        let mut installed = configuration.initial_installation();
        // Retained ports may have published since the installation token was
        // issued. Eligibility is decided under the same epoch lock as cutover.
        if self.runtime.history().latest_commit().is_some()
            || self.runtime.history.has_published_branch_basis()
        {
            return Err(RelationalInitialSchemaInstallationDenial::new(
                RelationalInitialSchemaInstallationDenialKind::RuntimeAlreadyCommitted,
                "initial schema authority closes after the first published mutation",
            ));
        }
        if installed
            .schema_contract_runtime
            .initial_custom_invariants_sealed
        {
            return Err(RelationalInitialSchemaInstallationDenial::new(
                RelationalInitialSchemaInstallationDenialKind::InitialInvariantsAlreadySealed,
                "initial custom-invariant inventory is already sealed",
            ));
        }
        let merged = installed
            .config
            .schema
            .registry
            .clone()
            .extend(additions)
            .map_err(schema_denial)?;
        let retained_entity_kind_count = merged.entity_kinds.len();
        let retained_relation_kind_count = merged.relation_kinds.len();
        // The registry and the contract runtime lowered from it are installed as
        // one change, so no concurrently bound service can observe the new
        // registry against the old contract runtime.
        let generation = installed
            .schema_contract_runtime
            .custom_invariant_generation
            .checked_add(1)
            .expect(
                "initial custom-invariant generation cannot overflow on its one legal transition",
            );
        let mut registrations = installed
            .schema_contract_runtime
            .custom_invariant_registries
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        registrations.extend(custom_invariants);
        let inventory_digest = custom_invariant_inventory_digest(&registrations);
        let registry = FrozenCustomInvariantRegistry::from_registrations(registrations).map_err(
            |duplicate| {
                RelationalInitialSchemaInstallationDenial::new(
                    RelationalInitialSchemaInstallationDenialKind::DuplicateCustomInvariant,
                    duplicate.identity.rule_id.as_str(),
                )
            },
        )?;
        let rebuilt =
            rebuilt_schema_contract_runtime(&installed, merged.clone(), registry, generation);
        // All fallible schema and invariant validation completes before the
        // first owner mutation. This keeps the initial installation atomic.
        self.runtime
            .history
            .transition_empty_branches_to_initial_schema(&merged)
            .map_err(branch_transition_denial)?;
        #[cfg(test)]
        if let Some((reached, resume)) = self.transition_pause {
            reached
                .send(())
                .expect("installation observer remains alive");
            resume
                .recv()
                .expect("installation observer releases cutover");
        }
        Arc::make_mut(&mut installed.config).schema.registry = merged;
        installed.schema_contract_runtime = Arc::new(rebuilt);
        drop(installed);
        // Refresh the exclusive owner's cached references after releasing the
        // write guard: reconfigure takes a fresh snapshot of this same authority.
        self.runtime.reconfigure(|_| {});
        Ok(RelationalInitialSchemaInstallationReceipt {
            runtime_instance_id: self.runtime.runtime_instance_id(),
            custom_invariant_generation: generation,
            custom_invariant_inventory_digest: inventory_digest,
            retained_entity_kind_count,
            retained_relation_kind_count,
        })
    }
}

/// Lower the merged registry into a fresh contract runtime, keeping the custom
/// invariant registries the installed one already carries.
fn rebuilt_schema_contract_runtime(
    installed: &RelationalRuntimeConfigurationSnapshot,
    merged: RelationalSchemaRegistry,
    custom_invariants: FrozenCustomInvariantRegistry,
    custom_invariant_generation: u64,
) -> SchemaContractRuntimeSubsystem {
    let mut config = installed.config.as_ref().clone();
    config.schema.registry = merged;
    let mut rebuilt = <SchemaContractRuntimeSubsystem as RuntimeSubsystem>::new(&config);
    rebuilt.custom_invariant_registries = custom_invariants;
    rebuilt.custom_invariant_generation = custom_invariant_generation;
    rebuilt.initial_custom_invariants_sealed = true;
    rebuilt
}

pub fn custom_invariant_inventory_digest(
    registrations: &[CustomInvariantRegistration],
) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut identities = registrations
        .iter()
        .map(|registration| {
            let descriptor = registration.descriptor();
            (
                descriptor.identity.rule_id.as_str().to_owned(),
                descriptor.identity.semantic_version,
                registration.execution_point(),
                descriptor.display_name.to_string(),
                registration.groups().mask(),
                registration.cost_class(),
                registration.failure_effect(),
                registration.maximum_work_units(),
                descriptor.operational.access.clone(),
            )
        })
        .collect::<Vec<_>>();
    identities.sort();
    let mut digest = Sha256::new();
    for (rule, version, point, display_name, groups, cost, failure, work, access) in identities {
        digest.update((rule.len() as u64).to_le_bytes());
        digest.update(rule.as_bytes());
        digest.update(version.major.to_le_bytes());
        digest.update(version.minor.to_le_bytes());
        digest.update([point as u8]);
        digest.update((display_name.len() as u64).to_le_bytes());
        digest.update(display_name.as_bytes());
        digest.update(groups.to_le_bytes());
        digest.update([cost as u8, failure as u8]);
        digest.update(work.get().to_le_bytes());
        for kinds in [
            &access.read_entity_kinds,
            &access.read_relation_kinds,
            &access.affected_entity_kinds,
            &access.affected_relation_kinds,
        ] {
            digest.update((kinds.len() as u64).to_le_bytes());
            for kind in kinds {
                digest.update(kind.0.to_le_bytes());
            }
        }
    }
    digest.finalize().into()
}

fn schema_denial(error: SchemaRegistryError) -> RelationalInitialSchemaInstallationDenial {
    RelationalInitialSchemaInstallationDenial::new(
        RelationalInitialSchemaInstallationDenialKind::SchemaRejected,
        error.detail,
    )
}

fn branch_transition_denial(
    denial: crate::branch::RelationalBranchCellDenial,
) -> RelationalInitialSchemaInstallationDenial {
    let kind = match denial {
        crate::branch::RelationalBranchCellDenial::RetentionCapacityExhausted => {
            RelationalInitialSchemaInstallationDenialKind::RetentionCapacityExhausted
        }
        crate::branch::RelationalBranchCellDenial::RetentionIdentityExhausted => {
            RelationalInitialSchemaInstallationDenialKind::RetentionIdentityExhausted
        }
        crate::branch::RelationalBranchCellDenial::RetentionOwnerUnavailable => {
            RelationalInitialSchemaInstallationDenialKind::RetentionOwnerUnavailable
        }
        crate::branch::RelationalBranchCellDenial::RetentionRootSetTooLarge => {
            RelationalInitialSchemaInstallationDenialKind::RetentionRootSetTooLarge
        }
        _ => RelationalInitialSchemaInstallationDenialKind::BranchTransitionRejected,
    };
    RelationalInitialSchemaInstallationDenial::new(
        kind,
        format!("empty branch schema transition failed: {denial:?}"),
    )
}

#[cfg(test)]
mod tests;
