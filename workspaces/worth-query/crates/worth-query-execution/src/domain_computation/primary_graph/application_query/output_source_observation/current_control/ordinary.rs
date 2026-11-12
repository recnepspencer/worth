//! An ordinary producer lends its admitted C control without navigation lifetimes.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::product_operation::current_read::{
    CurrentReadControl, CurrentReadDenial,
};
pub(in crate::domain_computation::primary_graph::application_query::output_source_observation)
struct CurrentSourceQueryControl
<'a> {
    proof: CurrentReadControl<'a>,
    lane: Arc<WorthQueryApplicationBranchCommitLane>,
    held: Option<&'a crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'a>>,
}
impl<'a, S: ApplicationSchema> SourceQueryReaderAccess<'a, S> {
    pub(in crate::domain_computation::primary_graph) fn with_predecessor_reads(
        mut self,
        authorization: Option<
            crate::domain_computation::authorization::BorrowedReadOperationAdmission<'a>,
        >,
        access: Option<crate::domain_computation::primary_graph::application_contribution::producer::predecessor_reads::ProducerPredecessorReadAccess>,
    ) -> Self {
        if access.is_some() {
            self.read_authorization = authorization;
        }
        self.predecessor = access;
        self
    }
    pub(in crate::domain_computation::primary_graph) fn with_current_control(
        mut self,
        proof: CurrentReadControl<'a>,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        if self.history.is_some()
            || self.shared.selected().product().observation() != proof.current().observation()
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
    pub(in crate::domain_computation::primary_graph) fn with_coordinated_control(
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
    pub(in crate::domain_computation::primary_graph::application_query::output_source_observation) fn read_control(
        &self,
    ) -> Option<CurrentReadControl<'_>> {
        self.history
            .as_ref()
            .map(|history| history.invocation.read_control())
            .or_else(|| self.current.as_ref().map(|current| current.proof))
    }
    pub(in crate::domain_computation::primary_graph::application_query::output_source_observation) fn confirm_current_control<
        T,
    >(
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
                    .graph
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
