use std::marker::PhantomData;

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationSchema,
};
use worth_runtime_world::facade::ProductBranchObservation;

use crate::domain_computation::primary_graph::application_entry::mutation::WorthQueryCompletedMutationCandidate;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

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

#[derive(Debug)]
pub enum WorthQueryProgramMigrationPreparationDenial {
    ProgramSupportUnavailable,
    TargetProgramUnrostered,
    TargetBindingUnowned,
    ForeignCandidate,
    MutationPartitionUnavailable,
    UnsupportedPosture(&'static str),
    Candidate(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial),
}

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

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn admit_program_migration<Binding>(
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
            target: target.clone(),
            marker: PhantomData,
        })
    }

    /// Seals a candidate completed by the exact mutation binding admitted for
    /// the target program.
    ///
    /// A completed candidate cannot be relabelled as another binding, even
    /// when both bindings happen to use the same associated types:
    ///
    /// ```compile_fail,E0308
    /// use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
    /// use worth_query_execution::facade::primary_graph::{
    ///     WorthQueryAdmittedProgramMigration, WorthQueryCompletedMutationCandidate,
    ///     WorthQueryPrimaryGraphApplicationRuntime,
    /// };
    /// use worth_query_installation::facade::ApplicationSchema;
    ///
    /// fn cannot_relabel_candidate<Schema, First, Second>(
    ///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ///     admitted: WorthQueryAdmittedProgramMigration<Schema, Second>,
    ///     candidate: WorthQueryCompletedMutationCandidate<Schema, First>,
    /// ) where
    ///     Schema: ApplicationSchema,
    ///     First: ApplicationMutationBinding<Schema>,
    ///     Second: ApplicationMutationBinding<Schema>,
    /// {
    ///     let _ = runtime.seal_program_migration::<Second>(admitted, candidate);
    /// }
    /// ```
    pub fn seal_program_migration<Binding>(
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
                self, program,
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

    fn require_target_binding<Binding>(
        &self,
        target: &ApplicationProgramRevision,
    ) -> Result<(), WorthQueryProgramMigrationPreparationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let support = self
            .installed_program_support()
            .ok_or(WorthQueryProgramMigrationPreparationDenial::ProgramSupportUnavailable)?;
        let entry = support
            .present(target)
            .ok_or(WorthQueryProgramMigrationPreparationDenial::TargetProgramUnrostered)?;
        if !entry.acts_through_mutation_binding(std::any::TypeId::of::<Binding>()) {
            return Err(WorthQueryProgramMigrationPreparationDenial::TargetBindingUnowned);
        }
        Ok(())
    }
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
