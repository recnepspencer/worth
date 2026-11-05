use crate::equality_type_id::EqualityTypeId as TypeId;
use std::marker::PhantomData;

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationSchema,
};
use worth_runtime_world::facade::ProductBranchObservation;

use crate::domain_computation::primary_graph::application_entry::mutation::WorthQueryCompletedMutationCandidate;

/// What a sealed program migration does: the operation that authored it and how
/// many effects it carries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramMigrationDescription {
    operation: String,
    effect_count: usize,
}

impl WorthQueryProgramMigrationDescription {
    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub const fn effect_count(&self) -> usize {
        self.effect_count
    }
}

/// Why a program migration could not be admitted or sealed. Nothing changed.
#[derive(Debug)]
pub enum WorthQueryProgramMigrationPreparationDenial {
    /// The runtime has no installed program support.
    ProgramSupportUnavailable,
    /// The target program is not on the installed roster.
    TargetProgramUnrostered,
    /// The target program does not act through this mutation binding.
    TargetBindingUnowned,
    /// The candidate belongs to another runtime or schema binding.
    ForeignCandidate,
    /// The runtime could not issue a mutation partition for the migration.
    MutationPartitionUnavailable,
    /// The candidate does something a migration may not do, such as an external
    /// effect, an aftermath, a conditional definition, platform effects,
    /// emitted output, or output production. The text names which.
    UnsupportedPosture(&'static str),
    /// The candidate's effects were refused while building the migration.
    Candidate(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial),
}

/// Admission to author a program migration for one target program through one
/// mutation binding.
///
/// Get it from `admit_program_migration`, complete a mutation candidate with
/// the same binding, then seal both with `seal_program_migration`. It grants
/// nothing else.
pub struct WorthQueryAdmittedProgramMigration<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    target: ApplicationProgramRevision,
    marker: PhantomData<fn() -> (Schema, Binding)>,
}

/// A sealed, adoption-only candidate issued by the execution owner.
///
/// Callers cannot construct one from a target and a raw Relational batch:
///
/// ```compile_fail,E0451
/// use worth_query_execution::facade::primary_graph::WorthQueryPreparedProgramMigration;
///
/// fn raw_batches_are_not_migration_authority(
///     batch: worth_relational::facade::transactions::WorkerIntentBatch,
/// ) -> WorthQueryPreparedProgramMigration {
///     WorthQueryPreparedProgramMigration {
///         target: unimplemented!(),
///         source_product: unimplemented!(),
///         description: unimplemented!(),
///         batch,
///     }
/// }
/// ```
#[must_use = "a prepared migration can only be consumed by branch adoption preparation"]
pub struct WorthQueryPreparedProgramMigration {
    target: ApplicationProgramRevision,
    source_product: ProductBranchObservation,
    description: WorthQueryProgramMigrationDescription,
    batch: worth_relational::facade::transactions::WorkerIntentBatch,
}

impl WorthQueryPreparedProgramMigration {
    pub fn target(&self) -> &ApplicationProgramRevision {
        &self.target
    }

    pub fn source_product(&self) -> &ProductBranchObservation {
        &self.source_product
    }

    pub const fn description(&self) -> &WorthQueryProgramMigrationDescription {
        &self.description
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        ApplicationProgramRevision,
        ProductBranchObservation,
        WorthQueryProgramMigrationDescription,
        worth_relational::facade::transactions::WorkerIntentBatch,
    ) {
        (
            self.target,
            self.source_product,
            self.description,
            self.batch,
        )
    }
}

pub trait MigrationOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn admit_program_migration<Binding>(
        &self,
        target: &ApplicationProgramRevision,
    ) -> Result<
        WorthQueryAdmittedProgramMigration<Schema, Binding>,
        WorthQueryProgramMigrationPreparationDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.require_target_binding::<Binding>(target)?;
        Ok(WorthQueryAdmittedProgramMigration {
            target: *target,
            marker: PhantomData,
        })
    }

    fn seal_program_migration<Binding>(
        &self,
        admitted: WorthQueryAdmittedProgramMigration<Schema, Binding>,
        candidate: WorthQueryCompletedMutationCandidate<Schema, Binding>,
    ) -> Result<WorthQueryPreparedProgramMigration, WorthQueryProgramMigrationPreparationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.require_target_binding::<Binding>(&admitted.target)?;
        let (program, _result) = candidate.into_parts();
        let (source_product, batch, effect_count) =
            crate::domain_computation::primary_graph::application_attempt::prepare_program_migration_effects(
                self.application_attempt_host(), program,
            )?;
        Ok(WorthQueryPreparedProgramMigration {
            target: admitted.target,
            source_product,
            description: WorthQueryProgramMigrationDescription {
                operation:
                    <Binding::Operation as ApplicationOperationMarkerIdentity<Schema>>::IDENTIFIER
                        .to_owned(),
                effect_count,
            },
            batch,
        })
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > MigrationOps<Schema> for HostRuntime
{
}

trait MigrationOwnOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn require_target_binding<Binding>(
        &self,
        target: &ApplicationProgramRevision,
    ) -> Result<(), WorthQueryProgramMigrationPreparationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let support = self
            .application_runtime()
            .installed_program_support()
            .ok_or(WorthQueryProgramMigrationPreparationDenial::ProgramSupportUnavailable)?;
        let entry = support
            .present(target)
            .ok_or(WorthQueryProgramMigrationPreparationDenial::TargetProgramUnrostered)?;
        if !entry.acts_through_mutation_binding(TypeId::of::<Binding>()) {
            return Err(WorthQueryProgramMigrationPreparationDenial::TargetBindingUnowned);
        }
        Ok(())
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > MigrationOwnOps<Schema> for HostRuntime
{
}

impl From<crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    for WorthQueryProgramMigrationPreparationDenial
{
    fn from(
        denial: crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    ) -> Self {
        Self::Candidate(denial)
    }
}
