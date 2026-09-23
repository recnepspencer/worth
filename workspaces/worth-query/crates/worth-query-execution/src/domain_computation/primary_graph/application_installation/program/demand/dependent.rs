use super::*;

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn admit_program_dependent_output<ParentDemand, Connection>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
        basis: &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        minimum_observation: &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, ConnectionDemand<Schema, Connection>>,
            SourceValue<Schema, ConnectionDemand<Schema, Connection>>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<Schema, Program, ConnectionDemand<Schema, Connection>>,
        WorthQueryOutputDemandDenial,
    >
    where
        ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
        Connection: ApplicationConnectionShape<Schema>,
        ConnectionBinding<Schema, Connection>:
            WorthQueryApplicationDependentOutputConnection<Schema, RootDemand = ParentDemand>,
    {
        if !self.contains_connection_type::<Connection>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "dependent output connection is not installed for this program",
            ));
        }
        if parent.target_feature != std::any::TypeId::of::<Connection::SourceFeature>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "dependent output edge does not leave this settled program feature",
            ));
        }
        if !parent.retained.belongs_to(&self.runtime) {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "dependent output parent belongs to another installed application",
            ));
        }
        let artifact = self
            .validate_derived_artifact_demand::<Connection, ConnectionDemand<Schema, Connection>>(
                maximum_work,
                maximum_retained_bytes,
            )?;
        if let (Some(child), Some(parent_artifact)) = (artifact, parent.artifact) {
            let consumes_parent = child.dependencies().iter().any(|dependency| {
                dependency.identity() == parent_artifact.identity()
                    || dependency.identity() == parent_artifact.feature()
            });
            if !consumes_parent {
                return Err(WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                    "the dependent artifact does not consume its settled parent artifact",
                ));
            }
        }
        let selected_source = source
            .observed_sources()
            .first()
            .filter(|_| source.rows().len() == 1 && source.observed_sources().len() == 1);
        let parent_observation = parent.retained.retained_read();
        let parent_commit = parent_observation.selected_commit();
        let parent_occurrence = parent_observation.branch_incarnation();
        let parent_branch = parent_observation.branch_identity();
        if self
            .runtime
            .select_application_read_observation(basis)
            .is_err()
            || self
                .runtime
                .select_application_read_observation(minimum_observation)
                .is_err()
            || basis.branch_identity() != parent_branch
            || minimum_observation.branch_identity() != parent_branch
            || basis.selected_commit().ordinal() < parent_commit.ordinal()
            || basis.selected_commit().ordinal() < minimum_observation.selected_commit().ordinal()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "dependent observation must be owned by this program and follow its settled parent and source publication on the same product branch",
            ));
        }
        if selected_source.is_none_or(|observed| {
            observed.selected_product_commit() != Some(basis.selected_commit())
                || observed.selected_product_occurrence() != Some(parent_occurrence)
        }) {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "dependent output source was not discovered at its Query-owned continuation observation",
            ));
        }
        self.runtime
            .admit_required_output_demand::<Family<Schema, ConnectionDemand<Schema, Connection>>>(
                source,
                maximum_work,
                maximum_retained_bytes,
            )
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                target_feature: std::any::TypeId::of::<Connection::TargetFeature>(),
                artifact,
                marker: std::marker::PhantomData,
            })
    }
}
