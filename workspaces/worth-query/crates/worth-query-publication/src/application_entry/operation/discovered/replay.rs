//! Replay rejoins genuine registered custody after fresh original-key admission, never handler reexecution.
use super::*;
use WorthQueryOperationAdvance as Advance;
use WorthQueryOperationAdvanceDenial as Denial;
use WorthQueryOperationProgress as Progress;
impl<S, I, P, R, A> WorthQueryDiscoveredOperation<S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationAction<S, I::Binding>,
    ActionDelivery<S, I, A>: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    Admission<S, I>: WorthQueryBindingAdmission<S, I::Binding>,
    MutationInput<S, I>: Clone + Send + Sync,
    MutationScope<S, I>: ApplicationMutationScopeResolution<S, MutationPrincipal<S, I>>,
    RootDemand<S, R>: Clone,
    DiscoveryValue<S, R>: WorthQueryApplicationProjection<S, DiscoveryQuery<S, R>> + Clone,
    DiscoveryScope<S, R>: ApplicationQueryScopeResolution<S, DiscoveryPrincipal<S, R>>,
    RootInput<S, R>: ApplicationQueryIntent<S, Binding = RootSource<S, R>>,
    RootScope<S, R>: ApplicationQueryScopeResolution<S, RootPrincipal<S, R>>,
    RootValue<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
    R::Dependents: ProgramOutputContinuationFactory<S, P, RootDemand<S, R>>,
{
    pub(super) fn advance_replay(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> DiscoveredAdvance<S, I, P, R, A> {
        use worth_query_execution::facade::primary_graph::{
            WorthQueryApplicationIdempotencyResolution as Resolution,
            WorthQueryManagedApplicationRecoveryDenial as Recovery,
        };
        let State::Replay(original) = &self.state else {
            unreachable!("dispatch owns committed replay evidence")
        };
        let admitted = (|| {
            let mut native = self
                .description
                .request_view(request, &self.installed_action);
            let prepared =
                native.authorize_discovered_source::<P, R>(phase, &self.runtime, None)?;
            let read = native.read_recovery_idempotency(phase, &prepared)?;
            let Resolution::AlreadyCommitted(receipt) = read.into_resolution() else {
                return Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(
                    Recovery::BindingMismatch,
                ));
            };
            if !receipt.is_same_authoritative_commit(original) {
                return Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(
                    Recovery::BindingMismatch,
                ));
            }
            Ok((prepared, receipt))
        })();
        let (prepared, receipt) = match admitted {
            Ok(admitted) => admitted,
            Err(denial) => {
                return Advance::Refused {
                    operation: self,
                    denial: Denial::Recovery(denial),
                };
            }
        };
        match request.recover_admitted_discovered_outputs_in_advancement::<P, R, _, _, _>(
            phase,
            &self.runtime,
            &receipt,
            WorthQueryOutputDemandControls::host_policy(),
            &prepared.admission,
        ) {
            Ok(outputs) => {
                self.state = State::Outputs(outputs);
                Advance::Progress {
                    operation: self,
                    progress: Progress::RegisteredOutputsRecovered,
                }
            }
            Err(AdmittedDiscoveredRecoveryDenial::Authorization(denial)) => Advance::Refused {
                operation: self,
                denial: Denial::Recovery(WorthQueryApplicationRecoveryRequestDenial::Recovery(
                    Recovery::Authorization(denial),
                )),
            },
            Err(AdmittedDiscoveredRecoveryDenial::Outputs(denial)) => {
                if denial.recovery_posture() == WorthQueryRequiredOutputRecoveryPosture::Terminal {
                    self.state = State::NotPerformed(
                        WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt),
                    );
                    self.terminal(Denial::Outputs(denial))
                } else {
                    Advance::Refused {
                        operation: self,
                        denial: Denial::Outputs(denial),
                    }
                }
            }
        }
    }
}
