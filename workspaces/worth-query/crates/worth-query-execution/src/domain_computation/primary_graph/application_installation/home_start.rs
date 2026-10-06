//! Starting the home: seeding an empty one, or resuming the image it holds.

use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use super::home_opening::HomeStarted;
use super::open_plan::InitialState;
use super::open_refusal::OpenFailure;
use super::program_admission::WorthQueryAdmittedProgramSupport;
use super::{WorthQueryApplicationOpenDenial, WorthQueryOpenAdoption, WorthQueryOpenEntryKind};
use crate::domain_computation::primary_graph::bootstrap::checkpoint_transition::transition_checkpoint;
use crate::domain_computation::primary_graph::bootstrap::{
    recorded_program_activation, WorthQueryProgramActivationSeed,
};
use crate::domain_computation::primary_graph::program_occurrence::{
    program_revision_rendering, WorthQueryProgramActivationCell,
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};

/// The entry after its admission step ran against the installed schema.
pub(super) enum AdmittedEntry<Schema> {
    Declaration,
    Program(WorthQueryAdmittedProgramSupport<Schema>),
}

/// Starts an empty home: a program entry seeds its activation as the first
/// transaction, then the declared initial state runs.
pub(super) fn seed_empty<Schema>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    entry: &AdmittedEntry<Schema>,
    activation: &WorthQueryProgramActivationCell,
    initial_state: InitialState<'_, Schema>,
) -> Result<(), WorthQueryApplicationOpenDenial> {
    if let AdmittedEntry::Program(support) = entry {
        graph.program_activation_seed = Some(WorthQueryProgramActivationSeed::for_initial_program(
            &support.initial_revision,
            activation.clone(),
        ));
    }
    initial_state(graph, installed).map_err(WorthQueryApplicationOpenDenial::InitialState)
}

/// Resumes a recovered image through the entry that wrote it.
///
/// The image kind is derived, never stored: an image with a program activation
/// is a program image. The declared adoption predecessor adopts; any other
/// recorded revision in the admitted roster resumes; the rest is refused unchanged.
pub(super) fn resume<Schema: ApplicationSchema>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    entry: &AdmittedEntry<Schema>,
    activation: &WorthQueryProgramActivationCell,
    adoption: Option<WorthQueryOpenAdoption<'_, Schema>>,
    image_retains_outputs: bool,
) -> Result<HomeStarted, OpenFailure> {
    use WorthQueryApplicationOpenDenial as Denial;

    let recorded = recorded_program_activation(&graph.graph).map_err(Denial::Graph)?;
    let (support, recorded) = match (entry, recorded) {
        (AdmittedEntry::Declaration, None) => return Ok(HomeStarted::DeclarationResumed),
        (AdmittedEntry::Declaration, Some(_)) => {
            return Err(Denial::EntryKindMismatch {
                image: WorthQueryOpenEntryKind::Program,
                entry: WorthQueryOpenEntryKind::Declaration,
            }
            .into())
        }
        (AdmittedEntry::Program(_), None) => {
            return Err(Denial::EntryKindMismatch {
                image: WorthQueryOpenEntryKind::Declaration,
                entry: WorthQueryOpenEntryKind::Program,
            }
            .into())
        }
        (AdmittedEntry::Program(support), Some(recorded)) => (support, recorded),
    };
    // A declared adoption wins over the roster: a host may keep its predecessor
    // rostered and still adopt an image recorded under it.
    let Some(adoption) = adoption.filter(|adoption| adoption.adopts(&recorded.rendering)) else {
        let installed_revision = support
            .roster
            .entries()
            .iter()
            .map(|entry| *entry.revision())
            .find(|revision| program_revision_rendering(revision) == recorded.rendering)
            .ok_or_else(|| {
                Denial::Graph(denial(
                    "recovered program activation is not in the admitted roster",
                ))
            })?;
        activation.publish(recorded.identity).map_err(|_| {
            Denial::Graph(denial("recovered program activation was already published"))
        })?;
        return Ok(HomeStarted::ProgramResumed {
            installed: installed_revision,
        });
    };
    if image_retains_outputs {
        return Err(Denial::Graph(denial(
            "checkpoint transition requires accepted-output migration support",
        ))
        .into());
    }
    let from = adoption.predecessor.clone();
    let successor = transition_checkpoint(graph, installed, support, activation, adoption)?;
    Ok(HomeStarted::Adopted {
        from,
        installed: support.initial_revision,
        successor,
    })
}

fn denial(subject: &str) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        subject,
    )
}
