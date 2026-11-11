mod readmission;
use crate::runtime::{RelationalRuntime, SchemaContractRuntimeSubsystem};
use crate::schema::data::RelationalSchemaRegistry;
use crate::validation::{data::CustomInvariantRegistration, FrozenCustomInvariantRegistry};
use std::sync::Arc;

mod application_publication;
mod fresh_admission;
mod home_target;
mod inventory_digest;
mod publication;
mod receipt;
pub use publication::FreshRelationalInitialSchemaInstallation;
pub(crate) use publication::InitialSchemaCutover;
pub use receipt::RelationalInitialSchemaInstallationReceipt;
mod recovered_codec_transition;
pub use inventory_digest::custom_invariant_inventory_digest;
pub(crate) use inventory_digest::custom_invariant_inventory_digest_for_compilation;
pub use recovered_codec_transition::RecoveredCheckpointProgramActivationUpdate;
pub(crate) use recovered_codec_transition::{
    codec_field_changes, codec_field_changes_in_context, CodecFieldComparisonStop, CodecFieldLocus,
};

/// Move-only authority to extend an uncommitted runtime's initial schema.
#[derive(Debug)]
pub struct RelationalInitialSchemaInstallation<'runtime> {
    runtime: &'runtime mut RelationalRuntime,
    #[cfg(test)]
    transition_pause: Option<(std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>)>,
}

pub(in crate::runtime) mod denial;
use denial::schema_denial;
pub use denial::{
    RelationalInitialSchemaInstallationDenial, RelationalInitialSchemaInstallationDenialKind,
};
impl RelationalRuntime {
    pub(crate) fn initial_schema_authority_snapshot(
        &self,
    ) -> RelationalInitialSchemaAuthoritySnapshot {
        RelationalInitialSchemaAuthoritySnapshot {
            custom_invariants: self
                .schema_contract_runtime
                .custom_invariant_registries
                .clone(),
            custom_invariant_generation: self.schema_contract_runtime.custom_invariant_generation,
            sealed: self
                .schema_contract_runtime
                .initial_custom_invariants_sealed,
        }
    }

    pub(crate) fn restore_initial_schema_authority_after_recovery(
        &mut self,
        snapshot: RelationalInitialSchemaAuthoritySnapshot,
    ) {
        let configuration = self.configuration_binding();
        let mut installed = configuration.initial_installation();
        let rebuilt = rebuilt_schema_contract_runtime(
            &installed.config.schema.registry,
            &installed.config.schema.invariant_catalog.registrations,
            snapshot.custom_invariants,
            snapshot.custom_invariant_generation,
            snapshot.sealed,
            crate::schema::SchemaCompilationControl::Configuration,
        )
        .expect("request-free recovery configuration has no execution refusal");
        installed.schema_contract_runtime = Arc::new(rebuilt);
        drop(installed);
        self.reconfigure(|_| {});
    }

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

    /// Reissues the initial-installation proof after whole-runtime recovery.
    ///
    /// Recovery deliberately mints a fresh runtime identity. The caller may
    /// carry its pre-recovery receipt only as an expected contract; this court
    /// verifies the recovered schema and invariant inventory before binding a
    /// new receipt to the restored authority.
    pub fn readmit_recovered_initial_schema_installation(
        &self,
        expected: RelationalInitialSchemaInstallationReceipt,
    ) -> Result<RelationalInitialSchemaInstallationReceipt, RelationalInitialSchemaInstallationDenial>
    {
        let registry = &self.config().schema.registry;
        let schema = crate::schema::data::schema_authority_snapshot_digest_bytes(
            &registry.authority_snapshot(),
        );
        expected.readmit_against(
            self.runtime_instance_id(),
            registry,
            &self.schema_contract_runtime,
            schema,
            crate::schema::SchemaCompilationControl::Configuration,
        )
    }
}

pub(crate) struct RelationalInitialSchemaAuthoritySnapshot {
    custom_invariants: FrozenCustomInvariantRegistry,
    custom_invariant_generation: u64,
    sealed: bool,
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
        self.install_with_managed_sources(additions, custom_invariants, Vec::new())
    }

    /// Install exact public Source programs alongside actual invariant rules.
    /// Source access is derived from the receiving schema, never caller supplied.
    pub fn install_with_managed_sources(
        self,
        additions: RelationalSchemaRegistry,
        custom_invariants: Vec<CustomInvariantRegistration>,
        managed_sources: Vec<crate::validation::candidate_source::ManagedIndexSourceProgram>,
    ) -> Result<RelationalInitialSchemaInstallationReceipt, RelationalInitialSchemaInstallationDenial>
    {
        self.install_with_publication(additions, custom_invariants, managed_sources, None)
    }

    fn install_with_publication(
        self,
        additions: RelationalSchemaRegistry,
        custom_invariants: Vec<CustomInvariantRegistration>,
        managed_sources: Vec<crate::validation::candidate_source::ManagedIndexSourceProgram>,
        publication: Option<(
            crate::durability::source_initial_inputs::RelationalFreshInitialization,
            &worth_execution::ExecutionRequest<'_, '_>,
        )>,
    ) -> Result<RelationalInitialSchemaInstallationReceipt, RelationalInitialSchemaInstallationDenial>
    {
        if let Some((initialization, request)) = &publication {
            initialization
                .validate(self.runtime, request)
                .map_err(RelationalInitialSchemaInstallationDenial::from_initial_inputs)?;
        }
        let configuration = self.runtime.configuration_binding();
        let mut installed = configuration.initial_installation();
        let routes = self.runtime.history.canonical_checkpoint_gate();
        let publication_guard = if publication.is_some() {
            Some(routes.try_initial_schema_publication().map_err(|_| {
                RelationalInitialSchemaInstallationDenial::new(
                    RelationalInitialSchemaInstallationDenialKind::InitialPublicationBusy,
                    "Native publication is in flight before initialization",
                )
            })?)
        } else {
            None
        };
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
        let retained_schema_authority_digest =
            crate::schema::data::schema_authority_snapshot_digest_bytes(
                &merged.authority_snapshot(),
            );
        // The registry and the contract runtime lowered from it are installed as
        // one change, so no concurrently bound service can observe the new
        // registry against the old contract runtime.
        let generation = installed
            .schema_contract_runtime
            .custom_invariant_generation
            .checked_add(1)
            .expect(/* Proof: Initial installation is a one-way empty-to-installed transition; generation starts at zero and this path increments it only once. */
                "initial custom-invariant generation cannot overflow on its one legal transition",
            );
        let mut registrations = installed
            .schema_contract_runtime
            .custom_invariant_registries
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        registrations.extend(custom_invariants);
        let control = publication.as_ref().map_or(
            crate::schema::SchemaCompilationControl::Configuration,
            |(_, request)| crate::schema::SchemaCompilationControl::Request(**request),
        );
        let inventory_digest =
            custom_invariant_inventory_digest_for_compilation(&registrations, control).map_err(
                |cause| {
                    RelationalInitialSchemaInstallationDenial::new(
                        RelationalInitialSchemaInstallationDenialKind::Request(cause),
                        format!("{cause:?}"),
                    )
                },
            )?;
        let mut sources = installed
            .schema_contract_runtime
            .custom_invariant_registries
            .explicit_source_programs();
        sources.extend(managed_sources);
        let registry =
            FrozenCustomInvariantRegistry::compile(&merged, registrations, sources, control)
                .map_err(denial::inventory_compilation_denial)?;
        let source_groups = registry.source_inventory();
        let rebuilt = rebuilt_schema_contract_runtime(
            &merged,
            &installed.config.schema.invariant_catalog.registrations,
            registry,
            generation,
            true,
            control,
        )
        .map_err(|cause| {
            RelationalInitialSchemaInstallationDenial::new(
                RelationalInitialSchemaInstallationDenialKind::Request(cause),
                format!("{cause:?}"),
            )
        })?;
        // All fallible schema and invariant validation completes before the
        // first owner mutation. This keeps the initial installation atomic.
        self.runtime
            .history
            .transition_empty_branches_to_initial_schema(&merged, |before, after| {
                match publication {
                    None => Ok(InitialSchemaCutover::Receiver),
                    Some((_initialization, request)) => {
                        let original = self.runtime.history.capture_original_branch_schema_inputs(request).map_err(RelationalInitialSchemaInstallationDenial::original_capture)?;
                        let image = crate::durability::source_initial_inputs::capture::capture(
                            &installed.config.history.main_branch, &original, Some((before, after, routes.branch_transition_head())), request,
                        ).map_err(RelationalInitialSchemaInstallationDenial::from_initial_inputs)?;
                        let medium = self.runtime.durability.medium().ok_or_else(|| RelationalInitialSchemaInstallationDenial::new(RelationalInitialSchemaInstallationDenialKind::InitialInputsRejected, "initialization medium detached"))?;
                        medium.publish_initial_inputs(image, request).map(InitialSchemaCutover::Durable).map_err(RelationalInitialSchemaInstallationDenial::publication)
                    }
                }
            })?;
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
        drop(publication_guard);
        // Refresh the exclusive owner's cached references after releasing the
        // write guard: reconfigure takes a fresh snapshot of this same authority.
        self.runtime.reconfigure(|_| {});
        Ok(RelationalInitialSchemaInstallationReceipt {
            runtime_instance_id: self.runtime.runtime_instance_id(),
            source_groups,
            custom_invariant_generation: generation,
            custom_invariant_inventory_digest: inventory_digest,
            retained_entity_kind_count,
            retained_relation_kind_count,
            retained_schema_authority_digest,
        })
    }
}

/// Lower the merged registry into a fresh contract runtime, keeping the custom
/// invariant registries the installed one already carries.
fn rebuilt_schema_contract_runtime(
    merged: &RelationalSchemaRegistry,
    declared: &[crate::validation::data::InvariantRegistration],
    custom_invariants: FrozenCustomInvariantRegistry,
    custom_invariant_generation: u64,
    sealed: bool,
    control: crate::schema::SchemaCompilationControl<'_, '_>,
) -> Result<SchemaContractRuntimeSubsystem, crate::execution::RelationalExecutionDenialCause> {
    let plans = crate::branch::RetainedRelationalAspectPlans::compile(merged, control)?;
    SchemaContractRuntimeSubsystem::for_schema_compilation(
        merged,
        declared,
        plans,
        custom_invariants,
        custom_invariant_generation,
        sealed,
        control,
    )
}

mod recovery_configuration;
#[cfg(test)]
mod tests;
