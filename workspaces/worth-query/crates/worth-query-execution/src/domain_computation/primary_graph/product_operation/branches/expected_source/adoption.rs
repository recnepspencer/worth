//! Replace an application head only while the candidate's exact source is current.
use super::*;
use worth_relational::facade::branch::RelationalBranchBasisDenial;
use worth_runtime_world::facade::{
    ProductBranchCurrentnessFailure, ProductBranchObservation, RuntimeWorldBranchAdmissionDenial,
    RuntimeWorldServiceDenial,
};

#[derive(Debug)]
pub enum WorthQueryExpectedProductBranchAdoptionDenial {
    ForeignPhase,
    Stopped(crate::domain_computation::request_execution::WorthQueryAdvancementDenial),
    /// The application's actual head slot no longer names the selected source.
    HeadReplaced,
    CandidateAdmission(WorthQueryProductBranchAdmissionDenial),
    SourceBasis(RelationalBranchBasisDenial),
    CandidateBasis(RelationalBranchBasisDenial),
    HeadAdmission(RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>),
    HeadMoved,
}

/// Adoption did not change the supplied head. Candidate effects still exist;
/// the returned continuation retains their ordinary close/recovery custody.
#[must_use = "retain or close the candidate returned by failed adoption"]
pub struct WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema> {
    candidate: WorthQueryExpectedProductBranch<'runtime, Schema>,
    denial: WorthQueryExpectedProductBranchAdoptionDenial,
    deferred: Option<(
        WorthQueryProductBranchLease,
        Option<ProductBranchObservation>,
    )>,
}

/// Native/World custody released by the outer owner after its slot guard ends.
pub(in crate::domain_computation::primary_graph) struct ProductBranchAdoptionRelease<
    'runtime,
    Schema,
> {
    _candidate: WorthQueryProductBranchLease,
    _source: WorthQueryExpectedProductBranch<'runtime, Schema>,
    _expected: ProductBranchObservation,
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryExpectedProductBranch<'runtime, Schema> {
    /// Replace the application's actual head slot with this candidate.
    ///
    /// This synchronizes with ordinary commits and closes on both branches.
    /// Native validates their component bases and World holds both exact heads
    /// current across the slot replacement. The original request's liveness is
    /// checked inside those guards. No authored operation runs here.
    /// On success, close the returned previous branch through ordinary lifecycle
    /// handling after updating any application metadata. A later close refusal
    /// does not undo the already adopted head.
    pub fn adopt(
        self,
        head: &mut WorthQueryProductBranch,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryProductBranch,
        WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema>,
    > {
        self.adopt_with_endpoint(head, phase, None)
            .map(|(previous, release)| {
                drop(release);
                previous
            })
            .map_err(|mut failure| {
                // Public adoption has no enclosing document guard. Preserve its
                // original transient-pin lifetime before returning the owner.
                drop(failure.deferred.take());
                failure
            })
    }

    pub(in crate::domain_computation::primary_graph) fn adopt_at_deferred(
        self,
        head: &mut WorthQueryProductBranch,
        phase: &WorthQueryAdvancementPhase<'_>,
        expected: &crate::basis::WorthQueryProductObservationLease,
    ) -> Result<
        (
            WorthQueryProductBranch,
            ProductBranchAdoptionRelease<'runtime, Schema>,
        ),
        WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema>,
    > {
        self.adopt_with_endpoint(head, phase, Some(expected))
    }

    fn adopt_with_endpoint(
        self,
        head: &mut WorthQueryProductBranch,
        phase: &WorthQueryAdvancementPhase<'_>,
        expected: Option<&crate::basis::WorthQueryProductObservationLease>,
    ) -> Result<
        (
            WorthQueryProductBranch,
            ProductBranchAdoptionRelease<'runtime, Schema>,
        ),
        WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema>,
    > {
        let execution = match phase.execution_request_for(&self.application.runtime.product_runtime)
        {
            Ok(execution) => execution,
            Err(_) => {
                return Err(self
                    .adoption_failed(WorthQueryExpectedProductBranchAdoptionDenial::ForeignPhase))
            }
        };
        if let Err(stop) = execution.check_live() {
            return Err(self.adoption_stopped(stop));
        }
        if *head != self.source.product_branch() {
            return Err(
                self.adoption_failed(WorthQueryExpectedProductBranchAdoptionDenial::HeadReplaced)
            );
        }
        let application = self.application;
        let mut occurrences = [
            self.source.product_branch().occurrence(),
            self.branch.occurrence(),
        ];
        occurrences.sort_unstable();
        let lanes = occurrences.map(|occurrence| {
            application
                .runtime
                .primary_provider
                .application_branch_commit_lane_for_occurrence(occurrence)
        });
        let _first = lanes[0].enter();
        let _second = lanes[1].enter();
        let candidate = match application
            .runtime
            .product_runtime
            .admit_product_occurrence(self.branch.occurrence())
        {
            Ok(candidate) => candidate,
            Err(denial) => {
                return Err(self.adoption_failed(
                    WorthQueryExpectedProductBranchAdoptionDenial::CandidateAdmission(denial),
                ))
            }
        };
        if expected.is_some_and(|expected| expected.observation() != candidate.observation()) {
            let mut failure =
                self.adoption_failed(WorthQueryExpectedProductBranchAdoptionDenial::HeadMoved);
            failure.deferred = Some((candidate, None));
            return Err(failure);
        }
        let expected_source = self.source.observation().clone();
        // Keep resource drops outside Native access and the outer head-slot guard.
        let result =
            application
                .runtime
                .primary_provider
                .graph
                .with_runtime(&execution, |native| {
                    let basis = native.owner_component_services().basis_port();
                    if let Err(denial) = basis.compare_current_exact(self.source.relational_basis())
                    {
                        return Err(self.adoption_failed(
                            WorthQueryExpectedProductBranchAdoptionDenial::SourceBasis(denial),
                        ));
                    }
                    if let Err(denial) = basis.compare_current_exact(candidate.relational_basis()) {
                        return Err(self.adoption_failed(
                            WorthQueryExpectedProductBranchAdoptionDenial::CandidateBasis(denial),
                        ));
                    }
                    self.adopt_while_heads_current(head, &candidate, &expected_source, &execution)
                });
        match result {
            Ok((previous, adopted)) => Ok((
                previous,
                ProductBranchAdoptionRelease {
                    _candidate: candidate,
                    _source: adopted,
                    _expected: expected_source,
                },
            )),
            Err(mut failure) => {
                failure.deferred = Some((candidate, Some(expected_source)));
                Err(failure)
            }
        }
    }

    fn adopt_while_heads_current(
        self,
        head: &mut WorthQueryProductBranch,
        candidate: &WorthQueryProductBranchLease,
        expected: &ProductBranchObservation,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (WorthQueryProductBranch, Self),
        WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema>,
    > {
        let world = self
            .application
            .runtime
            .product_runtime
            .owner
            .observation_port();
        match world.while_product_branches_current(
            expected,
            candidate.observation(),
            self,
            |candidate, _source, _candidate_branch| match execution.check_live() {
                Ok(()) => Ok((std::mem::replace(head, candidate.branch), candidate)),
                Err(stop) => Err((candidate, stop)),
            },
        ) {
            Ok(Ok(adopted)) => Ok(adopted),
            Ok(Err((candidate, stop))) => Err(candidate.adoption_stopped(stop)),
            Err(ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(candidate)) => {
                Err(candidate
                    .adoption_failed(WorthQueryExpectedProductBranchAdoptionDenial::HeadMoved))
            }
            Err(ProductBranchCurrentnessFailure::AdmissionDenied { denial, argument }) => {
                Err(argument.adoption_failed(
                    WorthQueryExpectedProductBranchAdoptionDenial::HeadAdmission(denial),
                ))
            }
        }
    }

    fn adoption_failed(
        self,
        denial: WorthQueryExpectedProductBranchAdoptionDenial,
    ) -> WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema> {
        WorthQueryExpectedProductBranchAdoptionFailure {
            candidate: self,
            denial,
            deferred: None,
        }
    }

    fn adoption_stopped(
        self,
        stop: worth_execution::MapKernelStop,
    ) -> WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema> {
        let denial = crate::domain_computation::request_execution::advancement_denial(
            worth_execution::WorkCeilingDenial::Stopped(stop),
        );
        self.adoption_failed(WorthQueryExpectedProductBranchAdoptionDenial::Stopped(
            denial,
        ))
    }
}

impl<'runtime, Schema> WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema> {
    pub fn denial(&self) -> &WorthQueryExpectedProductBranchAdoptionDenial {
        &self.denial
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryExpectedProductBranch<'runtime, Schema>,
        WorthQueryExpectedProductBranchAdoptionDenial,
    ) {
        let Self {
            candidate,
            denial,
            deferred,
        } = self;
        drop(deferred);
        (candidate, denial)
    }
}

impl<Schema> std::fmt::Debug for WorthQueryExpectedProductBranchAdoptionFailure<'_, Schema> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryExpectedProductBranchAdoptionFailure")
            .field("branch", &self.candidate.branch)
            .field("denial", &self.denial)
            .finish()
    }
}
