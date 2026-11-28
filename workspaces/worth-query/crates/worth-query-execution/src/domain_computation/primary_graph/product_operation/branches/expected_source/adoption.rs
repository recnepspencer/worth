//! Replace an application head only while the candidate's exact source is current.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementDenial;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductObservationLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeBranchLifecycleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use worth_runtime_world::facade::{ProductBranchCurrentnessFailure, ProductBranchObservation};

use worth_relational::facade::branch::RelationalBranchBasisDenial;
use worth_runtime_world::facade::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldServiceDenial};

#[derive(Debug)]
pub enum WorthQueryExpectedProductBranchAdoptionDenial {
    ForeignPhase,
    Stopped(WorthQueryAdvancementDenial),
    /// The application's actual head slot no longer names the selected source.
    HeadReplaced,
    CandidateAdmission(crate::basis::WorthQueryProductBranchAdmissionDenial),
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
        self.adopt_and_release(head, phase)
            .map(|(previous, release)| {
                drop(release);
                previous
            })
            .map_err(|mut failure| {
                // Release transient pins before returning the candidate owner.
                drop(failure.deferred.take());
                failure
            })
    }

    fn adopt_and_release(
        self,
        head: &mut WorthQueryProductBranch,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        (
            WorthQueryProductBranch,
            ProductBranchAdoptionRelease<'runtime, Schema>,
        ),
        WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema>,
    > {
        let application = self.application;
        let execution = match phase
            .execution_request_for(application.inner.runtime.product_runtime_ref())
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
        let mut occurrences = [
            self.source.product_branch().occurrence(),
            self.branch.occurrence(),
        ];
        occurrences.sort_unstable();
        let lanes = occurrences.map(|occurrence| {
            application
                .inner
                .runtime
                .primary_provider_ref()
                .application_branch_commit_lane_for_occurrence(occurrence)
        });
        let _first = lanes[0].enter();
        let _second = lanes[1].enter();
        let candidate = match application
            .inner
            .runtime
            .product_runtime_ref()
            .admit_product_occurrence(self.branch.occurrence())
        {
            Ok(candidate) => candidate,
            Err(denial) => {
                return Err(self.adoption_failed(
                    WorthQueryExpectedProductBranchAdoptionDenial::CandidateAdmission(denial),
                ))
            }
        };
        let expected_source = self.source.observation().clone();
        // Keep resource drops outside Native access and the outer head-slot guard.
        let result = application
            .inner
            .runtime
            .primary_provider_ref()
            .graph_ref()
            .with_runtime(&execution, |native| {
                let basis = native.owner_component_services().basis_port();
                if let Err(denial) = basis.compare_current_exact(self.source.relational_basis()) {
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
        let application = self.application;
        let world = application
            .inner
            .runtime
            .product_runtime_ref()
            .owner_ref()
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
        let denial =
            crate::domain_computation::primary_graph::application_contribution::advancement_denial(
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
