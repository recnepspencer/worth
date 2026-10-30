//! Fresh ordinary reads and mutation assessment precede exact-source creation.
use super::*;

#[derive(Debug)]
pub enum PreparationRefusal {
    Request(Failure),
    UnsupportedVersion(u16),
    EmptyProposal,
    ModelMismatch,
    ModelHeadMismatch,
    TargetMismatch,
    Mutation(WorthQueryApplicationRequestMutationDenial),
    Description(serde_json::Error),
    Ordinal(std::num::TryFromIntError),
    Reservation(std::collections::TryReserveError),
    Selection(WorthQueryExpectedProductBasisSelectionDenial),
    Fork(WorthQueryProductBranchAdmissionDenial),
    Creation(WorthQueryProductBranchCreateError),
}
pub enum PreparationFailure<'host> {
    Before(PreparationRefusal),
    Refused {
        cause: PreparationRefusal,
        handoff: Option<WorthQueryAdvancementDenial>,
    },
    Advancement(WorthQueryAdvancementDenial),
    Created {
        staging: Staging<'host>,
        handoff: WorthQueryAdvancementDenial,
    },
}
impl std::fmt::Debug for PreparationFailure<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Before(cause) => formatter.debug_tuple("Before").field(cause).finish(),
            Self::Refused { cause, handoff } => formatter
                .debug_struct("Refused")
                .field("cause", cause)
                .field("handoff", handoff)
                .finish(),
            Self::Advancement(cause) => formatter.debug_tuple("Advancement").field(cause).finish(),
            Self::Created { staging, handoff } => formatter
                .debug_struct("Created")
                .field("branch", &staging.branch())
                .field("handoff", handoff)
                .finish(),
        }
    }
}
pub(super) fn authorize(
    model: &Model<'_>,
    caller: &Caller<'_>,
    proposal: &StoredProposal,
) -> Result<(), PreparationRefusal> {
    model
        .describe(caller)
        .map_err(PreparationRefusal::Request)?;
    if proposal.model != model.identity {
        return Err(PreparationRefusal::ModelMismatch);
    }
    let observed = model.read(caller).map_err(PreparationRefusal::Request)?;
    for AuthoredIntent::Retention(input) in &proposal.intents {
        if input.model != model.identity {
            return Err(PreparationRefusal::ModelMismatch);
        }
        if input.target.occurrence != observed.occurrence || input.target.label != observed.name {
            return Err(PreparationRefusal::TargetMismatch);
        }
        model
            .owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(model.branch())
            .advanced()
            .mutate(input.clone())
            .without_source()
            .assess_current_capability_authorization()
            .map_err(PreparationRefusal::Mutation)?;
    }
    Ok(())
}
impl<'host> Staging<'host> {
    pub fn prepare(
        model: &Model<'host>,
        storage: &Storage<'host>,
        caller: &Caller<'_>,
        reference: &ProposalReference,
        attempt: u64,
    ) -> Result<Self, PreparationFailure<'host>> {
        let prepare = || -> Result<_, PreparationRefusal> {
            let proposal = storage
                .load(caller, reference)
                .map_err(PreparationRefusal::Request)?;
            if proposal.format_version != 1 {
                return Err(PreparationRefusal::UnsupportedVersion(
                    proposal.format_version,
                ));
            }
            if proposal.intents.is_empty() {
                return Err(PreparationRefusal::EmptyProposal);
            }
            authorize(model, caller, &proposal)?;
            u64::try_from(proposal.intents.len()).map_err(PreparationRefusal::Ordinal)?;
            let description: WorthQueryExpectedProductBasisDescription =
                serde_json::from_str(&proposal.expected_basis)
                    .map_err(PreparationRefusal::Description)?;
            let mut completed = Vec::new();
            completed
                .try_reserve_exact(proposal.intents.len())
                .map_err(PreparationRefusal::Reservation)?;
            let metadata = AdoptionMetadata {
                proposal: reference.clone(),
                attempt,
            };
            let model_metadata = metadata.clone();
            Ok((proposal, description, completed, metadata, model_metadata))
        };
        let (proposal, description, completed, metadata, model_metadata) =
            prepare().map_err(PreparationFailure::Before)?;
        let host = model.owner.host;
        let mut created = None;
        let advancement = host.with_application_advancement(caller.scope, |phase| {
            created = Some((|| {
                let selected = host
                    .branches()
                    .select_expected(&description, &phase)
                    .map_err(PreparationRefusal::Selection)?;
                if selected.product().product_branch() != model.branch() {
                    return Err(PreparationRefusal::ModelHeadMismatch);
                }
                host.branches()
                    .fork_expected(selected)
                    .map_err(PreparationRefusal::Fork)?
                    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                    .create(&phase)
                    .map_err(PreparationRefusal::Creation)
            })());
        });
        match (created, advancement) {
            (Some(Ok(candidate)), handoff) => {
                let staging = Self {
                    host,
                    candidate: Some(candidate),
                    metadata,
                    model_metadata: Some(model_metadata),
                    proposal,
                    next: 0,
                    step: None,
                    completed,
                };
                match handoff {
                    Ok(()) => Ok(staging),
                    Err(handoff) => Err(PreparationFailure::Created { staging, handoff }),
                }
            }
            (Some(Err(cause)), handoff) => Err(PreparationFailure::Refused {
                cause,
                handoff: handoff.err(),
            }),
            (None, Err(cause)) => Err(PreparationFailure::Advancement(cause)),
            (None, Ok(())) => unreachable!("successful advancement invoked creation"),
        }
    }
}
