//! Store retains the genuine standalone Signal owner before World admission.
use crate::ApplicationHomeOpenRefusal as D;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::SignalDurableJournal;
use worth_signal::facade::durability::*;

pub(super) fn attach_medium(
    home: &crate::ReadmittedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Arc<crate::StoreSignalMedium>, D> {
    let selected = home
        .context
        .selected_home_source
        .as_ref()
        .ok_or(D::SignalSourceRequired)?;
    let manifest = selected
        .selected(&home.context.serving, request)
        .map_err(D::Checkpoint)?;
    let image = manifest
        .as_ref()
        .map(|manifest| {
            let (bytes, range) = manifest.signal_region();
            SignalAuthoredHomeImage::readmit_region(request, bytes, range)
                .map_err(|cause| D::SignalImage(Box::new(cause)))
        })
        .transpose()?;
    let medium = home
        .context
        .world_slot
        .as_ref()
        .ok_or(D::SignalSourceRequired)?
        .signal_slot()
        .attach(image, Some(selected.clone()), request)
        .map_err(D::SignalMedium)?;
    *home
        .context
        .signal_close_probe
        .try_lock()
        .map_err(|_| D::SignalMedium(SignalDurableMediumDenial::PublicationBusy))? =
        Some(Arc::downgrade(&medium));
    Ok(medium)
}

pub(in crate::world_section) fn readmit<Dm, I, E, Ctx, T>(
    home: &crate::ReadmittedApplicationHome,
    program: Box<worth_signal::facade::SignalRuntime<Dm, I, E, Ctx, T>>,
    vocabulary: SignalDurableTierVocabulary<T>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<SignalDurableOwner<Dm, I, E, Ctx, T>, D>
where
    Dm: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let medium = attach_medium(home, request)?;
    let source = medium
        .capture_recovery_source(request)
        .map_err(D::SignalMedium)?;
    let contribution = source.identity_contribution();
    let definitions = source
        .frontier(SignalDurableJournal::Definitions, request)
        .map_err(D::SignalMedium)?;
    let advances = source
        .frontier(SignalDurableJournal::Advances, request)
        .map_err(D::SignalMedium)?;
    drop(source);
    struct Installation;
    impl SignalAuthoredOwnerInstallation for Installation {
        fn readmit_branch(
            &mut self,
            branch: SignalAuthoredBranchInstallation<'_>,
            _: &ExecutionRequest<'_, '_>,
        ) -> Result<(), SignalAuthoredInstallationRefused> {
            if branch.bridge_section().is_empty() {
                Ok(())
            } else {
                Err(SignalAuthoredInstallationRefused)
            }
        }
    }
    let signal = if definitions == 0
        && advances == 0
        && home.relational.world_fresh_source_admission().is_some()
    {
        // The original protected cut was empty. The original ACKs below must be
        // read through a current source, not that earlier empty cut.
        medium
            .complete_home_source_admission()
            .map_err(D::SignalMedium)?;
        let empty = worth_execution::ExecutionRequestArrayBuilder::allocate(*request, 0)
            .map_err(|cause| D::SignalMedium(SignalDurableMediumDenial::Backing(cause)))?
            .seal_immutable_bytes()
            .map_err(|cause| D::SignalMedium(SignalDurableMediumDenial::Backing(cause)))?;
        let section = SignalBridgeAuthoredSection::from_bridge_bytes(empty).map_err(|cause| {
            D::SignalMedium(SignalDurableMediumDenial::Storage(format!("{cause:?}")))
        })?;
        let task = program
            .prepare_initial_authored_source(
                medium.clone(),
                contribution,
                vocabulary,
                request,
                |_| Ok(section),
            )
            .map_err(|refusal| D::SignalInitial(Box::new(refusal.into_parts().1)))?;
        let slot = home
            .context
            .world_slot
            .as_ref()
            .ok_or(D::SignalSourceRequired)?
            .signal_slot();
        let retained = slot
            .retain_initial(task, request)
            .map_err(D::SignalMedium)?;
        loop {
            match retained
                .advance(request)
                .map_err(|cause| D::SignalInitialProgress(Box::new(cause)))?
            {
                SignalInitialSourceProgress::Pending => request.consult().map_err(D::Execution)?,
                SignalInitialSourceProgress::Published => break,
            }
        }
        let task = retained
            .take_published()
            .map_err(|cause| D::SignalInitialProgress(Box::new(cause)))?;
        let published = task
            .try_into_published()
            .unwrap_or_else(|_| unreachable!("same original reported Published"));
        published.readmit_authored_owner(&mut Installation, request)
    } else {
        program.readmit_authored_owner(
            medium.clone(),
            contribution,
            vocabulary,
            &mut Installation,
            request,
        )
    }
    .map_err(|cause| D::SignalReadmission(Box::new(cause)))?;
    medium
        .complete_home_source_admission()
        .map_err(D::SignalMedium)?;
    Ok(signal)
}

impl<D, I, E, Ctx, T> super::WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn standalone_signal_source(&self) -> Option<SignalDurableBranchSourcePort<D, I, T>> {
        self.standalone_signal
            .as_ref()
            .map(SignalDurableOwner::source_port)
    }
}
