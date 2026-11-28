//! An ordinary producer lends its admitted C control without navigation lifetimes.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::product_operation::current_read::CurrentReadControlRuntimeExt as _;
use crate::domain_computation::primary_graph::product_operation::current_read::{
    CurrentReadControl, CurrentReadDenial,
};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductObservationLeaseInternalPort as _;
pub
struct CurrentSourceQueryControl
<'a> {
    proof: CurrentReadControl<'a>,
    lane: Arc<WorthQueryApplicationBranchCommitLane>,
    held: Option<&'a crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'a>>,
}
impl<'a, S: ApplicationSchema> SourceQueryReaderAccess<'a, S> {
    /// Lends the live coupled assembly so a producer decision's managed index
    /// read can select the aligned before basis or borrow the applied Source.
    pub fn with_applied_managed_sources(
        mut self,
        assembly: &'a worth_relational::facade::mvcc::RelationalAuthoredCoupledCandidate<'a>,
    ) -> Self {
        self.applied_managed_sources = Some(assembly);
        self
    }
    pub fn with_applied_participants(
        mut self,
        participants: &'a dyn crate::domain_computation::primary_graph::provider::AppliedOutputParticipants,
    ) -> Self {
        self.applied_participants = Some(participants);
        self
    }
    pub fn with_producer_reads<B: crate::domain_computation::primary_graph::application_contribution::WorthQueryApplicationProducerBinding<S>>(
        mut self,
        authorization: Option<
            crate::domain_computation::authorization::BorrowedReadOperationAdmission<'a>,
        >,
        access: Option<crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::ProducerPredecessorReadAccess>,
    ) -> Self{
        self.original = crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::ProducerOriginalReadAccess::for_binding::<S, B>();
        if access.is_some() || self.original.is_some() {
            self.read_authorization = authorization;
        }
        self.predecessor = access;
        self
    }
    pub fn with_current_control(
        mut self,
        proof: CurrentReadControl<'a>,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        if self.shared.selected().product().observation() != proof.current().observation()
            || !proof.admits_request(self.request)
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "producer source query control",
            ));
        }
        let lane = self
            .application
            .runtime
            .primary_provider
            .application_branch_commit_lane_for_occurrence(
                proof.current().observation().lifecycle_incarnation(),
            );
        self.current = Some(CurrentSourceQueryControl {
            proof,
            lane,
            held: None,
        });
        Ok(self)
    }
    pub fn with_coordinated_control(
        self,
        proof: CurrentReadControl<'a>,
        coordination: &'a crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'a>,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let mut admitted = self.with_current_control(proof)?;
        if !coordination.admits(proof.current().observation()) {
            return Err(demand_denial(CurrentReadDenial::HeadDiverged));
        }
        admitted
            .current
            .as_mut()
            .expect("current control just admitted")
            .held = Some(coordination);
        Ok(admitted)
    }
    pub fn read_control(&self) -> Option<CurrentReadControl<'_>> {
        self.current.as_ref().map(|current| current.proof)
    }
    pub fn confirm_current_control<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &mut ApplicationQueryNativeAccess<'_>,
        value: T,
    ) -> Result<T, SourceQueryObservationStop> {
        let Some(control) = &self.current else {
            return Ok(value);
        };
        if let Some(coordination) = control.held {
            let confirmed = match native {
                ApplicationQueryNativeAccess::Acquire => self
                    .application
                    .runtime
                    .primary_provider
                    .graph_ref()
                    .with_runtime(execution, |native| {
                        control.proof.confirm_in_coordinated(
                            execution,
                            self.application,
                            coordination,
                            native,
                            value,
                        )
                    }),
                ApplicationQueryNativeAccess::Borrowed(native) => {
                    control.proof.confirm_in_coordinated(
                        execution,
                        self.application,
                        coordination,
                        native,
                        value,
                    )
                }
            };
            return confirmed
                .map_err(|cause| SourceQueryObservationStop::CurrentControl(demand_denial(cause)));
        }
        match native {
            ApplicationQueryNativeAccess::Acquire => {
                control
                    .proof
                    .confirm_in_execution(execution, self.application, value)
            }
            ApplicationQueryNativeAccess::Borrowed(native) => control.proof.confirm_in_borrowed(
                execution,
                self.application,
                &control.lane,
                native,
                value,
            ),
        }
        .map_err(|cause| SourceQueryObservationStop::CurrentControl(demand_denial(cause)))
    }
}
fn demand_denial(cause: CurrentReadDenial) -> WorthQueryOutputDemandDenial {
    use WorthQueryOutputDemandDenialKind as Kind;
    let kind = match cause {
        CurrentReadDenial::Busy => Kind::CurrentControlBusy,
        CurrentReadDenial::Authorization(lapse) => Kind::of_request_authorization(
            WorthQueryOperationAuthorizationDenial::new(
                lapse.denial_kind(),
                "producer source query control",
            )
            .kind(),
        ),
        CurrentReadDenial::Stopped(cause) => Kind::ExecutionRequest(cause),
        CurrentReadDenial::NativeBasis(
            worth_relational::facade::branch::RelationalBranchBasisDenial::Cancelled,
        ) => Kind::Cancelled,
        CurrentReadDenial::NativeBasis(
            worth_relational::facade::branch::RelationalBranchBasisDenial::TimedOut,
        ) => Kind::TimedOut,
        CurrentReadDenial::HeadDiverged
        | CurrentReadDenial::NativeBasis(_)
        | CurrentReadDenial::HeadAdmission(_) => Kind::Superseded,
    };
    WorthQueryOutputDemandDenial::new(kind, "producer source query control")
}
