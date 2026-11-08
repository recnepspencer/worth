//! One physical admission chooses A transition or the ordinary fresh/current B path.
use super::super::{opening, refusal::*, request::*, runtime::*};
use super::{plan::*, refusal::*, request::*};
use worth_query_declaration::facade::{application_program::*, application_schema::*};
use worth_query_execution::facade::{
    application_contribution::*, application_home_integration::*, application_installation::*,
};
use worth_store_application::*;

impl<'a, A, AP, B, BP, Author>
    WorthQueryApplicationHomeRequest<'a, B, BP, WorthQueryHomeProgramTransition<'a, A, AP, Author>>
where
    A: ApplicationSchemaComposition,
    AP: ApplicationProgramDefinition<A> + 'static,
    AP::Outputs: ApplicationProgramOutputsShape<A> + WorthQueryApplicationProgramRoots<A>,
    AP::Contributions: WorthQueryApplicationContributionTuple<A>,
    B: ApplicationSchemaComposition,
    BP: ApplicationProgramDefinition<B> + 'static,
    BP::Outputs: ApplicationProgramOutputsShape<B> + WorthQueryApplicationProgramRoots<B>,
    BP::Contributions: WorthQueryApplicationContributionTuple<B>,
    Author: FnOnce(
        &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
        &WorthQueryHomeProgramTransitionTarget<'_>,
    ) -> Result<
        WorthQueryHomeProgramTransitionPlan,
        worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    >,
{
    #[inline(never)]
    pub fn open_in_advancement(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryApplicationHome<B, BP>,
        WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
    > {
        worth_execution::run_with_execution_stack(move || {
            self.open_in_advancement_on_execution_stack(phase)
        })
    }

    #[inline(never)]
    fn open_in_advancement_on_execution_stack(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryApplicationHome<B, BP>,
        WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
    > {
        let ready = {
            let prepared = self.prepare_transition_opening(phase)?;
            prepared.readmit(phase)?
        };
        ready.finish(phase)
    }

    #[inline(never)]
    fn prepare_transition_opening(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<
        Box<PreparedHomeProgramOpening<'a, A, AP, B, BP, Author>>,
        WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
    > {
        use WorthQueryHomeProgramTransitionDenial as Denial;
        use WorthQueryHomeProgramTransitionOpenRefused as Refused;
        let WorthQueryHomeProgramTransition {
            predecessor,
            author,
            resources,
        } = self.transition;
        let WorthQueryApplicationHomeLocation::Directory(location) = self.location else {
            return Err(Refused::configuration(Denial::DirectoryRequired));
        };
        if [
            resources.maximum_work_units,
            resources.maximum_cold_record_slots,
            resources.maximum_derived_rows,
        ]
        .iter()
        .any(|limit| !matches!(limit, Some(1..)))
        {
            return Err(Refused::configuration(Denial::FiniteResourcesRequired));
        }
        let after = prepare_home_program(
            self.program,
            self.roster,
            self.declaration,
            self.configuration,
            self.build,
            self.limits,
            self.authorization_time_source,
            super::super::support::directory_requirement_refusal,
        )
        .map_err(|cause| {
            Refused::successor(WorthQueryApplicationHomeOpenRefused::Preparation(cause))
        })?;
        let before = prepare_home_program(
            predecessor.program,
            predecessor.roster,
            predecessor.declaration,
            predecessor.configuration,
            predecessor.build,
            predecessor.limits,
            predecessor.authorization_time_source,
            super::super::support::directory_requirement_refusal,
        );
        let before = match before {
            Ok(before) => before,
            Err(cause) => {
                return Err(Refused::predecessor(
                    WorthQueryApplicationHomeOpenRefused::Preparation(cause),
                    after,
                ))
            }
        };
        let binding = match after.admitted_home_binding() {
            Ok(binding) => binding,
            Err(cause) => {
                return Err(Refused::successor(
                    WorthQueryApplicationHomeOpenRefused::Binding {
                        query: after,
                        cause,
                    },
                ))
            }
        };
        let predecessor = match before.admitted_home_binding() {
            Ok(binding) => binding,
            Err(cause) => {
                return Err(Refused::predecessor(
                    WorthQueryApplicationHomeOpenRefused::Binding {
                        query: before,
                        cause,
                    },
                    after,
                ))
            }
        };
        let mut physical = ApplicationHomeRequest::new(
            location,
            after.home_relational_definition(),
            binding,
            self.build,
        )
        .with_program_transition_predecessor(before.home_relational_definition(), predecessor);
        let observation = self
            .opening_control
            .as_ref()
            .map(|control| control.observation())
            .unwrap_or_default();
        if let Some(control) = self.opening_control {
            physical = physical.with_opening_control_for_test(control);
        }
        let request = phase.execution_request();
        let admitted =
            match recover_home(physical, &request).and_then(|home| admit_home(home, &request)) {
                Ok(home) => home,
                Err(cause) => {
                    return Err(Refused::successor(
                        WorthQueryApplicationHomeOpenRefused::Physical {
                            query: after,
                            cause,
                        },
                    ))
                }
            };
        Ok(Box::new(match select_home_program_opening(admitted) {
            ApplicationHomeProgramOpening::Current(home) => {
                drop((before, author));
                PreparedHomeProgramOpening::Current {
                    query: after,
                    home,
                    checkpoint_limits: self.checkpoint_limits,
                    observation,
                    initial_state: self.initial_state,
                }
            }
            ApplicationHomeProgramOpening::TransitionRequired(home) => {
                PreparedHomeProgramOpening::Transition {
                    before,
                    after,
                    home,
                    author,
                    resources,
                    checkpoint_limits: self.checkpoint_limits,
                    observation,
                }
            }
        }))
    }
}

// Each owned phase is consumed before its successor's stack frames begin.
enum PreparedHomeProgramOpening<
    'a,
    A: ApplicationSchemaComposition,
    AP,
    B: ApplicationSchemaComposition,
    BP,
    Author,
> {
    Current {
        query: PreparedHomeApplication<B, BP>, home: worth_proof::HomeAdmitted<AdmittedApplicationHome>,
        checkpoint_limits: ApplicationHomeCheckpointLimits, observation: ApplicationHomeOpeningObservation,
        initial_state: Box<dyn FnOnce(&WorthQueryBootstrapAdvancementPhase<'_>, &mut worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<B>, &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<B>) -> Result<(), worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> + 'a>,
    },
    Transition {
        before: PreparedHomeApplication<A, AP>, after: PreparedHomeApplication<B, BP>,
        home: worth_proof::HomeAdmitted<AdmittedApplicationHome>, author: Author,
        resources: worth_relational::facade::indexes::DerivedIndexMaintenanceBudget,
        checkpoint_limits: ApplicationHomeCheckpointLimits, observation: ApplicationHomeOpeningObservation,
    },
}
enum ReadyHomeProgramOpening<
    'a,
    A: ApplicationSchemaComposition,
    AP,
    B: ApplicationSchemaComposition,
    BP,
    Author,
> {
    Current(opening::preparation::DirectoryOpening<'a, B, BP>),
    Transition {
        before: ReadmittedExistingHomeNativeMeaning<A, AP>,
        after: PreparedHomeApplication<B, BP>,
        home: UnsharedReadmittedApplicationHome,
        author: Author,
        resources: worth_relational::facade::indexes::DerivedIndexMaintenanceBudget,
        checkpoint_limits: ApplicationHomeCheckpointLimits,
        observation: ApplicationHomeOpeningObservation,
    },
}
impl<'a, A, AP, B, BP, Author> PreparedHomeProgramOpening<'a, A, AP, B, BP, Author>
where
    A: ApplicationSchemaComposition,
    AP: ApplicationProgramDefinition<A> + 'static,
    AP::Outputs: ApplicationProgramOutputsShape<A> + WorthQueryApplicationProgramRoots<A>,
    AP::Contributions: WorthQueryApplicationContributionTuple<A>,
    B: ApplicationSchemaComposition,
    BP: ApplicationProgramDefinition<B> + 'static,
    BP::Outputs: ApplicationProgramOutputsShape<B> + WorthQueryApplicationProgramRoots<B>,
    BP::Contributions: WorthQueryApplicationContributionTuple<B>,
    Author: FnOnce(
        &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
        &WorthQueryHomeProgramTransitionTarget<'_>,
    ) -> Result<
        WorthQueryHomeProgramTransitionPlan,
        worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    >,
{
    #[inline(never)]
    fn readmit(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<
        Box<ReadyHomeProgramOpening<'a, A, AP, B, BP, Author>>,
        WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
    > {
        use WorthQueryHomeProgramTransitionOpenRefused as Refused;
        match self {
            Self::Current {
                query,
                home,
                checkpoint_limits,
                observation,
                initial_state,
            } => {
                let (query, home) = opening::preparation::readmit_native(phase, query, home)
                    .map_err(Refused::successor)?;
                Ok(Box::new(ReadyHomeProgramOpening::Current(
                    opening::preparation::DirectoryOpening {
                        query,
                        home,
                        checkpoint_limits,
                        observation,
                        initial_state,
                    },
                )))
            }
            Self::Transition {
                before,
                after,
                home,
                author,
                resources,
                checkpoint_limits,
                observation,
            } => {
                let (query, home) = match opening::preparation::readmit_native(phase, before, home)
                {
                    Ok(parts) => parts,
                    Err(refused) => return Err(Refused::predecessor(refused, after)),
                };
                let (before, home) = match opening::existing::finalize_native(phase, home, query) {
                    Ok(parts) => parts,
                    Err(refused) => return Err(Refused::predecessor(refused, after)),
                };
                Ok(Box::new(ReadyHomeProgramOpening::Transition {
                    before,
                    after,
                    home,
                    author,
                    resources,
                    checkpoint_limits,
                    observation,
                }))
            }
        }
    }
}
impl<A, AP, B, BP, Author> ReadyHomeProgramOpening<'_, A, AP, B, BP, Author>
where
    A: ApplicationSchemaComposition,
    AP: ApplicationProgramDefinition<A> + 'static,
    AP::Outputs: ApplicationProgramOutputsShape<A> + WorthQueryApplicationProgramRoots<A>,
    AP::Contributions: WorthQueryApplicationContributionTuple<A>,
    B: ApplicationSchemaComposition,
    BP: ApplicationProgramDefinition<B> + 'static,
    BP::Outputs: ApplicationProgramOutputsShape<B> + WorthQueryApplicationProgramRoots<B>,
    BP::Contributions: WorthQueryApplicationContributionTuple<B>,
    Author: FnOnce(
        &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
        &WorthQueryHomeProgramTransitionTarget<'_>,
    ) -> Result<
        WorthQueryHomeProgramTransitionPlan,
        worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    >,
{
    #[inline(never)]
    fn finish(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryApplicationHome<B, BP>,
        WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
    > {
        match self {
            Self::Current(opening) => opening::initialization::initialize_directory(phase, opening)
                .map_err(WorthQueryHomeProgramTransitionOpenRefused::successor),
            Self::Transition {
                before,
                after,
                home,
                author,
                resources,
                checkpoint_limits,
                observation,
            } => super::handoff::transition(
                phase,
                before,
                after,
                home,
                author,
                resources,
                checkpoint_limits,
                observation,
            ),
        }
    }
}
