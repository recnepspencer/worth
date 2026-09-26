use crate::basis::{WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease};
use crate::domain_computation::primary_graph::{
    application_query::{
        resource_lifecycle::WorthQueryApplicationBasisLease, WorthQueryApplicationQueryBasisCustody,
    },
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::ProductBranchObservation;

pub(in crate::domain_computation) trait WorthQueryProductObservationSource {
    fn product_observation(&self) -> &ProductBranchObservation;

    fn current_security_guard(&self) -> Option<&ProductBranchObservation> {
        None
    }
}

impl WorthQueryProductObservationSource for WorthQueryProductBranchLease {
    fn product_observation(&self) -> &ProductBranchObservation {
        self.observation()
    }
}

impl WorthQueryProductObservationSource for crate::basis::WorthQueryProductObservationLease {
    fn product_observation(&self) -> &ProductBranchObservation {
        self.observation()
    }

    fn current_security_guard(&self) -> Option<&ProductBranchObservation> {
        self.current_security_guard()
    }
}

/// Security truth is resolved freshly from the selected branch at each
/// admission stage. A different lifecycle occurrence is rejected before this
/// basis obtains current application truth, reusing an exact live Query-owned
/// snapshot when it already carries the same selected program interpretation.
pub(in crate::domain_computation) struct WorthQueryProductSecurityBasis<'basis> {
    _observation: ProductBranchObservation,
    application_basis: SecurityApplicationBasis<'basis>,
    _selected_program:
        Option<super::super::program_occurrence::WorthQueryProgramSupportInterpretation>,
}

enum SecurityApplicationBasis<'basis> {
    Owned(WorthQueryApplicationBasisLease),
    Reused(&'basis worth_relational::facade::snapshots::SnapshotHandle),
}

impl WorthQueryProductSecurityBasis<'_> {
    pub(in crate::domain_computation) fn snapshot_handle(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        match &self.application_basis {
            SecurityApplicationBasis::Owned(basis) => basis.snapshot_handle(),
            SecurityApplicationBasis::Reused(snapshot) => snapshot,
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn retain_indexed_security_basis<'basis>(
        &self,
        observation: &ProductBranchObservation,
        query_basis: Option<&'basis WorthQueryApplicationQueryBasisCustody>,
    ) -> Result<
        (
            SecurityApplicationBasis<'basis>,
            Option<super::super::program_occurrence::WorthQueryProgramSupportInterpretation>,
        ),
        WorthQueryProductBranchAdmissionDenial,
    > {
        let graph = self
            .runtime
            .primary_graph()
            .ok_or(WorthQueryProductBranchAdmissionDenial::ObservationRejected)?
            .integration_handle();
        graph
            .with_runtime_mut(|runtime| {
                graph.ensure_primary_indexes_for_basis(
                    runtime,
                    observation.basis().relational_basis(),
                )
            })
            .map_err(|_| WorthQueryProductBranchAdmissionDenial::ObservationRejected)?;
        let relational = observation.basis().relational_basis();
        if let Some(query_basis) =
            query_basis.filter(|basis| basis.can_reuse_security_snapshot_at(observation))
        {
            let (_, interpretation) = self
                .retain_selected_program_interpretation(relational)?
                .into_parts();
            if let Some(snapshot) = query_basis.reusable_security_snapshot(interpretation.as_ref())
            {
                return Ok((SecurityApplicationBasis::Reused(snapshot), interpretation));
            }
        }
        let mut application_basis = self.retain_product_application_basis(observation)?;
        let _ = self.bind_selected_program_interpretation(relational, &mut application_basis)?;
        Ok((SecurityApplicationBasis::Owned(application_basis), None))
    }

    pub(in crate::domain_computation) fn admit_product_security_basis(
        &self,
        product: &impl WorthQueryProductObservationSource,
    ) -> Result<WorthQueryProductSecurityBasis<'static>, WorthQueryProductBranchAdmissionDenial>
    {
        self.admit_product_security_basis_with_query_basis(product, None)
    }

    pub(in crate::domain_computation::primary_graph) fn admit_query_product_security_basis<
        'basis,
    >(
        &self,
        product: &impl WorthQueryProductObservationSource,
        query_basis: &'basis WorthQueryApplicationQueryBasisCustody,
    ) -> Result<WorthQueryProductSecurityBasis<'basis>, WorthQueryProductBranchAdmissionDenial>
    {
        self.admit_product_security_basis_with_query_basis(product, Some(query_basis))
    }

    fn admit_product_security_basis_with_query_basis<'basis>(
        &self,
        product: &impl WorthQueryProductObservationSource,
        query_basis: Option<&'basis WorthQueryApplicationQueryBasisCustody>,
    ) -> Result<WorthQueryProductSecurityBasis<'basis>, WorthQueryProductBranchAdmissionDenial>
    {
        let selected = product.product_observation();
        if let Some(guard) = product.current_security_guard() {
            return self.product_runtime.with_product_observation(
                guard.branch_identity(),
                |current| {
                    if current.lifecycle_incarnation() != guard.lifecycle_incarnation()
                        || current.selected_commit() != guard.selected_commit()
                        || current
                            .basis()
                            .relational_basis()
                            .materialization_is_complete()
                        || selected.lifecycle_incarnation() != guard.lifecycle_incarnation()
                        || !selected
                            .basis()
                            .relational_basis()
                            .materialization_is_complete()
                    {
                        return Err(WorthQueryProductBranchAdmissionDenial::IncarnationChanged);
                    }
                    let (application_basis, selected_program) =
                        self.retain_indexed_security_basis(selected, query_basis)?;
                    Ok(WorthQueryProductSecurityBasis {
                        _observation: selected.clone(),
                        application_basis,
                        _selected_program: selected_program,
                    })
                },
            );
        }
        self.product_runtime
            .with_product_observation(selected.branch_identity(), |observation| {
                if observation.lifecycle_incarnation() != selected.lifecycle_incarnation() {
                    return Err(WorthQueryProductBranchAdmissionDenial::IncarnationChanged);
                }
                let (application_basis, selected_program) =
                    self.retain_indexed_security_basis(&observation, query_basis)?;
                Ok(WorthQueryProductSecurityBasis {
                    _observation: observation,
                    application_basis,
                    _selected_program: selected_program,
                })
            })
    }
}
