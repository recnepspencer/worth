//! Inspect unsupported persisted owner state while the original Home stays owned.
mod admission_backing;
mod catalog_admission;
pub(crate) use catalog_admission::{
    admit as admit_catalog, admit_if_pending as admit_pending_catalog,
};
mod history;
mod intent;
pub(crate) use history::{HomeTransitionAdmission, ReadmittedHomeProgramTransitions};
pub(crate) mod completion;
mod opening;
mod owner;
pub use completion::{
    ApplicationHomeTransitionCompletionDenial, ApplicationHomeTransitionStepRejected,
};
pub use opening::{select_home_program_opening, ApplicationHomeProgramOpening};
mod publication;
use publication::HomeIntentPublication;
pub use publication::{
    ApplicationHomeTransitionIntentOutcome, ApplicationHomeTransitionStartRefused,
};
mod readmitted_step_admission;
mod staging;
mod step_admission;
mod step_source;
mod unopened_step_admission;
pub use admission_backing::ApplicationHomeTransitionAdmissionObservation;
pub use intent::{ApplicationHomeTransitionIntentDenial, HomeProgramTransitionIntent};
pub(crate) use owner::HomeTransitionOwner;
pub(crate) use readmitted_step_admission::admit as admit_readmitted_step;
pub(crate) use staging::{advance as advance_intent, stage as stage_intent};
pub(crate) use step_admission::admit as admit_step;
pub use step_admission::AdmittedApplicationHomeProgramStep;
pub(crate) use unopened_step_admission::admit as admit_unopened_step;
mod planning;
use crate::home_context::HomeContext;
use crate::registry::{HomeJournalAdmission, HOME_JOURNALS};
pub(crate) use planning::reserve as reserve_plan;
pub use planning::{
    ApplicationHomeTransitionPlanningDenial, ReservedApplicationHomeTransitionPlan,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority;
use worth_store::physical_runtime::{
    OrderedJournalDenial, OrderedJournalEntry, OrderedJournalName, OrderedJournalReadFailure,
    OrderedJournalResolution,
};

#[derive(Debug)]
pub enum ApplicationHomeOwnerAdmissionDenial {
    Execution(worth_execution::WorkCeilingDenial),
    NativeRecoveryRequired,
    NativeCodecTransitionPending,
    ProgramTransitionRequired,
    TransitionAdmission(ApplicationHomeTransitionIntentDenial),
    JournalUnavailable {
        journal: &'static str,
        cause: OrderedJournalDenial,
    },
    JournalRead {
        journal: &'static str,
        cause: OrderedJournalReadFailure,
    },
    JournalNotEmpty {
        journal: &'static str,
        head: OrderedJournalEntry,
    },
    JournalUnresolved {
        journal: &'static str,
    },
    JournalResolutionChanged {
        journal: &'static str,
        resolution: OrderedJournalResolution,
    },
}

pub(crate) fn readmit_no_pending_owners(
    context: &HomeContext,
    recovered: &RecoveredRelationalRuntimeAuthority,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), ApplicationHomeOwnerAdmissionDenial> {
    use ApplicationHomeOwnerAdmissionDenial as Denial;
    request.consult().map_err(Denial::Execution)?;
    if recovered.codec_transition_pending() {
        return Err(Denial::NativeCodecTransitionPending);
    }
    require_no_pending_home_journals(context, request)
}

pub(crate) fn require_no_pending_home_journals(
    context: &HomeContext,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), ApplicationHomeOwnerAdmissionDenial> {
    request
        .consult()
        .map_err(ApplicationHomeOwnerAdmissionDenial::Execution)?;
    for journal in HOME_JOURNALS {
        match journal.admission {
            HomeJournalAdmission::Native | HomeJournalAdmission::World => {}
            HomeJournalAdmission::Transition => {
                if !context
                    .transition
                    .settled(&context.descriptor.binding)
                    .map_err(ApplicationHomeOwnerAdmissionDenial::TransitionAdmission)?
                {
                    require_empty_journal(context, journal.name, request)?;
                }
            }
            HomeJournalAdmission::RequireAbsent => {
                require_empty_journal(context, journal.name, request)?;
            }
        }
    }
    if context.transition.required().is_some()
        && !context
            .transition
            .settled(&context.descriptor.binding)
            .map_err(ApplicationHomeOwnerAdmissionDenial::TransitionAdmission)?
    {
        return Err(ApplicationHomeOwnerAdmissionDenial::ProgramTransitionRequired);
    }
    Ok(())
}

fn require_empty_journal(
    context: &HomeContext,
    name: &'static str,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), ApplicationHomeOwnerAdmissionDenial> {
    use ApplicationHomeOwnerAdmissionDenial as Denial;
    request.consult().map_err(Denial::Execution)?;
    let journal = context
        .serving
        .ordered_journal(OrderedJournalName::new(name).expect("the closed Home journal name"))
        .map_err(|cause| Denial::JournalUnavailable {
            journal: name,
            cause,
        })?;
    if let Some(head) = journal.head().map_err(|cause| Denial::JournalRead {
        journal: name,
        cause,
    })? {
        return Err(Denial::JournalNotEmpty {
            journal: name,
            head,
        });
    }
    request.consult().map_err(Denial::Execution)?;
    // Admission permits only head+1. With no durable head, 1 is the only
    // possible unresolved successor; the resolver does not compare this
    // descriptive digest until it finds a durable entry.
    let resolution = journal
        .resolve(1, [0; 32])
        .map_err(|cause| Denial::JournalRead {
            journal: name,
            cause,
        })?;
    match resolution {
        OrderedJournalResolution::NotCommitted { head: 0 } => Ok(()),
        OrderedJournalResolution::Unresolved => Err(Denial::JournalUnresolved { journal: name }),
        OrderedJournalResolution::NotCommitted { head: _ }
        | OrderedJournalResolution::Committed(_)
        | OrderedJournalResolution::Conflict(_) => Err(Denial::JournalResolutionChanged {
            journal: name,
            resolution,
        }),
    }
}

mod native_bodies;
