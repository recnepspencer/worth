//! Source and Parent admissions share one installed permission/session owner.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryIssuedSelectedPrincipal, WorthQueryIssuedSelectedScope,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryParameterSet, ApplicationQueryScopeBinding,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

pub(super) enum SourceQueryParameters<Q> {
    Live(ApplicationQueryParameterSet<Q>),
    Recorded(Arc<super::super::request_recipe::CapturedSourceQueryRequest>),
}
pub(super) enum SourceScopePosture {
    Current,
    Historical(CurrentSourceRefusal),
}
pub(super) enum CurrentSourceRefusal {
    Absent(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    DifferentGeneration,
}

impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn with_resolved_plan<
        B: ApplicationQueryBinding<Schema>,
        R,
        E: From<SourceQueryObservationStop>,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationQueryBinding<Schema, B>,
        parameters: SourceQueryParameters<B::Query>,
        principal: &WorthQueryIssuedSelectedPrincipal<
            '_,
            '_,
            Schema,
            B::Principal,
            B::PrincipalIdentity,
        >,
        scope: &WorthQueryIssuedSelectedScope<
            '_,
            '_,
            Schema,
            <B::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
        posture: SourceScopePosture,
        native: &mut ApplicationQueryNativeAccess<'_>,
        consume: impl for<'plan> FnOnce(
            SourceReadPlan<'plan, Schema, B>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<R, E>,
    ) -> Result<R, E> {
        use SourceQueryObservationStop as Stop;
        let application = self.application;
        let selected = self
            .history
            .as_ref()
            .map_or(self.shared, |history| history.current)
            .selected();
        let access =
            WorthQueryApplicationQueryAccessContext::new(principal.principal(), scope.scope());
        let query = installed.query();
        let limits = application.resolve_application_query_limits(installed.limits());
        let query_maximum = NonZeroUsize::new(query.read_graph().maximum_result_count())
            .unwrap_or(NonZeroUsize::MAX);
        let (product, basis) = application
            .retain_selected_query_basis_admitted(self.shared)
            .map_err(|stop| self.record_control_refusal(Stop::Basis(stop)))?;
        let controls = if let Some(history) = &self.history {
            WorthQueryApplicationQueryControls::selected_read_one_shot_with_control(
                history.invocation.current().retained_clone(),
                product,
                basis,
                limits.maximum_results().min(query_maximum),
                self.request,
            )
        } else {
            WorthQueryApplicationQueryControls::selected_read_one_shot(
                product,
                basis,
                limits.maximum_results().min(query_maximum),
                self.request,
            )
            .map_err(|stop| {
                self.record_control_refusal(Stop::Read(WorthQueryAdmittedOneShotStop::Admission(
                    stop,
                )))
            })?
        }
        .with_selection_origin(self.shared.selected().selection_origin())
        .with_original_settled_producer(
            self.shared.selected().original_settled_producer().cloned(),
        );
        let prepared = match parameters {
            SourceQueryParameters::Live(parameters) => {
                application.prepare_source_query_admission(query, &access, parameters, controls)
            }
            SourceQueryParameters::Recorded(recipe) => application
                .prepare_application_query_source_readmission(
                    query,
                    &access,
                    recipe.parameters(),
                    controls,
                ),
        };
        let (parameters, controls) = prepared.map_err(|cause| {
            self.record_control_refusal(Stop::Preparation {
                phase: SourceQueryPreparationPhase::Parameters,
                cause,
            })
        })?;
        let permission = match &posture {
            SourceScopePosture::Current => application
                .prepare_application_query_permission_with_parameters(
                    execution,
                    query,
                    access,
                    query.identity(),
                    parameters,
                    controls,
                    selected,
                    self.history.as_ref().map(|_| self.shared.selected()),
                    self.request,
                    principal,
                    scope,
                    native,
                ),
            SourceScopePosture::Historical(_) => {
                let history = self.history.as_ref().ok_or(Stop::ForeignClosure)?;
                if !std::ptr::eq(scope.selected(), self.shared.selected()) {
                    return Err(Stop::ForeignClosure.into());
                }
                application.prepare_historical_source_query_permission_with_parameters(
                    execution,
                    query,
                    access,
                    query.identity(),
                    parameters,
                    controls,
                    selected,
                    Some(self.shared.selected()),
                    self.request,
                    principal,
                    history.invocation,
                    native,
                )
            }
        }
        .map_err(|cause| {
            // A denied C audience cannot distinguish historical scope presence.
            if let SourceScopePosture::Historical(absent) = &posture
                && matches!(
                    cause.kind(),
                    WorthQueryApplicationQueryAdmissionDenialKind::Authorization(_)
                ) {
                    return self.record_control_refusal(match absent {
                        CurrentSourceRefusal::Absent(absent) => Stop::Scope(absent.clone()),
                        CurrentSourceRefusal::DifferentGeneration => Stop::Preparation {
                            phase: SourceQueryPreparationPhase::Permission,
                            cause: WorthQueryApplicationQueryAdmissionDenial::new(
                                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                                format!(
                                    "{}: {}",
                                    self.history
                                        .as_ref()
                                        .expect("historical source")
                                        .invocation
                                        .family(),
                                    query.name()
                                ),
                            ),
                        },
                    });
                }
            // Only these identity causes are issued after C authorization.
            // A stopped permission observation must not disclose a privately
            // resolved generation through its diagnostic attachment.
            let cause = if matches!(posture, SourceScopePosture::Historical(_))
                && !matches!(cause.kind(),
                    WorthQueryApplicationQueryAdmissionDenialKind::HistoricalSourceNotLive { .. }
                    | WorthQueryApplicationQueryAdmissionDenialKind::HistoricalSourceKindMismatch { .. }
                    | WorthQueryApplicationQueryAdmissionDenialKind::HistoricalSourceOutsideDomain { .. }
                    | WorthQueryApplicationQueryAdmissionDenialKind::HistoricalSourceOwnershipUnavailable { .. }) {
                cause.without_historical_source()
            } else { cause };
            self.record_control_refusal(Stop::Preparation {
                phase: SourceQueryPreparationPhase::Permission,
                cause,
            })
        })?;
        if let Some(control) = self.read_control() {
            permission
                .require_current_principal(control)
                .map_err(|cause| {
                    self.record_control_refusal(Stop::Preparation {
                        phase: SourceQueryPreparationPhase::Permission,
                        cause,
                    })
                })?;
        }
        let plan = application
            .finish_prepared_application_query_permission_with_native_access(
                execution, permission, native,
            )
            .map_err(|cause| {
                self.record_control_refusal(Stop::Preparation {
                    phase: SourceQueryPreparationPhase::Session,
                    cause,
                })
            })?;
        consume(plan, native)
    }
}
