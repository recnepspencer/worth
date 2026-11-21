//! Changed Signal inputs a reopening Home must refuse, written by the oracle
//! that owns this Home's actual Signal owner.
use super::{WorldHomeFixture, WorldHomeFixtureRefusal};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;
use worth_store_application::ApplicationHomeOpenRefusal as Refusal;

/// One durable Signal advance whose recorded input this program cannot admit.
#[derive(Clone, Copy, Debug)]
pub enum ChangedSignalAdvance {
    /// Recorded by another Signal owner of the same Home.
    ForeignOwner,
    /// Framed in a Signal owner-frame version this program does not read.
    UnknownVersion,
}

/// The cause a refused Home open names for a changed Signal input.
#[derive(Debug, PartialEq, Eq)]
pub enum SignalOpenRefusal {
    Physical,
    ForeignOwner,
    UnsupportedVersion,
    Other(String),
}

impl WorldHomeFixture {
    /// A Signal advance row for this Home's root branch with one changed input.
    pub fn changed_signal_advance(
        &self,
        change: ChangedSignalAdvance,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Vec<u8> {
        let source = self.home.standalone_signal_source().unwrap();
        let branch = source.branches().unwrap()[0].0;
        let descriptor = source.observe(branch).unwrap();
        let (owner, descriptor) = match change {
            ChangedSignalAdvance::ForeignOwner => {
                let foreign = DurableSignalOwnerIdentity::from_recorded(
                    branch.owner().home(),
                    SignalOwnerSlotIdentity::from_recorded(branch.owner().slot().get() ^ 1),
                );
                (foreign, foreign_descriptor(&descriptor, branch, foreign))
            }
            ChangedSignalAdvance::UnknownVersion => (branch.owner(), descriptor),
        };
        let mut bytes = SignalBranchAdvanceArtifact::capture(
            execution,
            SignalBranchAdvanceIdentity::from_recorded(owner, 1 << 20),
            descriptor,
            None,
            SignalAuthoredBranchChange::Advanced,
        )
        .unwrap()
        .as_bytes()
        .to_vec();
        if let ChangedSignalAdvance::UnknownVersion = change {
            bytes[8..12].copy_from_slice(&999_u32.to_le_bytes());
        }
        bytes
    }
}

/// Classify a refused Home open by its Signal or physical cause.
pub fn signal_open_refusal(refused: &WorldHomeFixtureRefusal) -> SignalOpenRefusal {
    let cause = match refused {
        WorldHomeFixtureRefusal::Home(cause) => cause.as_ref(),
        WorldHomeFixtureRefusal::World(refused) => refused.reason(),
        other => return SignalOpenRefusal::Other(format!("{other:?}")),
    };
    match cause {
        Refusal::Physical(_) => SignalOpenRefusal::Physical,
        Refusal::SignalReadmission(denial) => match **denial {
            SignalOwnerReadmissionDenial::ForeignOwner => SignalOpenRefusal::ForeignOwner,
            SignalOwnerReadmissionDenial::Frame(SignalDurableFrameDenial::UnsupportedVersion(
                _,
            )) => SignalOpenRefusal::UnsupportedVersion,
            _ => SignalOpenRefusal::Other(format!("{cause:?}")),
        },
        other => SignalOpenRefusal::Other(format!("{other:?}")),
    }
}

fn foreign_descriptor(
    original: &SignalDurableBranchDescriptor,
    branch: DurableSignalBranchIdentity,
    foreign: DurableSignalOwnerIdentity,
) -> SignalDurableBranchDescriptor {
    let branch = DurableSignalBranchIdentity::from_recorded(foreign, branch.ordinal());
    SignalDurableBranchDescriptor::from_recorded(
        DurableSignalGeneration::from_recorded(
            DurableSignalCellIncarnation::from_recorded(branch, original.cell().ordinal()),
            original.generation().ordinal(),
        ),
        None,
        DurableSignalAdmissionIdentity::from_recorded(foreign, original.admission().ordinal()),
        SignalDefinitionTransitionReference::from_recorded(
            SignalDefinitionTransitionIdentity::from_recorded(
                foreign,
                original.definition().identity().ordinal(),
            ),
            original.definition().digest(),
        ),
    )
    .unwrap()
}
