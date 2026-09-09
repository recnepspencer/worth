use std::collections::HashMap;

use super::{UiHeadlessRetainedCommandStore, UiHeadlessRetainedOrder};

pub(crate) struct UiHeadlessRetainedPresentation {
    pub(crate) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(crate) epoch: Option<worth_ui_host_contract::UiHostPresentationEpoch>,
    pub(crate) receipt_affinity: Option<worth_ui_host_contract::UiMountedNodeReceiptAffinity>,
    pub(crate) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(crate) binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    pub(crate) content: worth_ui_host_contract::UiMountedContentGeneration,
    pub(crate) baseline: worth_ui_host_contract::UiHostSurfaceBaselineIdentity,
    pub(super) commands: UiHeadlessRetainedCommandStore,
    pub(super) order: UiHeadlessRetainedOrder,
    pub(crate) node_positions: HashMap<worth_ui_host_contract::UiMountedInstanceIdentity, u64>,
    pub(crate) node_by_position: HashMap<u64, worth_ui_host_contract::UiMountedInstanceIdentity>,
    pub(crate) auxiliary: worth_ui_host_contract::UiMountedPresentationAuxiliaryState,
    sample_overrides: HashMap<
        worth_ui_host_contract::UiMountedPaintCommandIdentity,
        worth_ui_host_contract::UiMountedPresentationSampleChange,
    >,
    sample_damage: Box<[worth_ui_host_contract::UiMountedLogicalDamage]>,
}

impl UiHeadlessRetainedPresentation {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn initial(
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        content: worth_ui_host_contract::UiMountedContentGeneration,
        baseline: worth_ui_host_contract::UiHostSurfaceBaselineIdentity,
        commands: UiHeadlessRetainedCommandStore,
        order: UiHeadlessRetainedOrder,
        node_positions: HashMap<worth_ui_host_contract::UiMountedInstanceIdentity, u64>,
        node_by_position: HashMap<u64, worth_ui_host_contract::UiMountedInstanceIdentity>,
        auxiliary: worth_ui_host_contract::UiMountedPresentationAuxiliaryState,
    ) -> Self {
        Self {
            frame,
            epoch: None,
            receipt_affinity: None,
            surface,
            binding,
            content,
            baseline,
            commands,
            order,
            node_positions,
            node_by_position,
            auxiliary,
            sample_overrides: HashMap::new(),
            sample_damage: Box::default(),
        }
    }

    pub(super) fn apply_sample(
        &mut self,
        sample: &worth_ui_host_contract::UiMountedPresentationSample,
    ) -> Result<(), worth_ui_host_contract::UiHostSurfacePresentationDenial> {
        let affinity = sample.affinity();
        if affinity.predecessor() != Some(self.frame)
            || affinity.successor() != self.frame
            || affinity.surface() != self.surface
            || affinity.binding() != self.binding
            || affinity.content() != self.content
            || affinity.baseline() != self.baseline
        {
            return Err(malformed());
        }
        let unique = sample
            .changes()
            .iter()
            .map(|change| change.command())
            .collect::<std::collections::HashSet<_>>();
        if unique.len() != sample.changes().len()
            || sample.changes().is_empty()
            || sample
                .changes()
                .iter()
                .any(|change| !self.commands.contains_key(&change.command()))
        {
            return Err(malformed());
        }
        for change in sample.changes() {
            self.sample_overrides.insert(change.command(), *change);
        }
        self.sample_damage = sample.damage().to_vec().into_boxed_slice();
        Ok(())
    }

    pub(super) fn install_reconstruction_sample_overrides(
        &mut self,
        changes: &[worth_ui_host_contract::UiMountedPresentationSampleChange],
        reconstruction_damage: &[worth_ui_host_contract::UiMountedLogicalDamage],
    ) -> Result<(), worth_ui_host_contract::UiHostSurfacePresentationDenial> {
        let overrides = changes
            .iter()
            .copied()
            .map(|change| (change.command(), change))
            .collect::<HashMap<_, _>>();
        if overrides.len() != changes.len()
            || overrides
                .keys()
                .any(|identity| !self.commands.contains_key(identity))
        {
            return Err(malformed());
        }
        self.sample_damage = if overrides.is_empty() {
            Box::default()
        } else {
            reconstruction_damage.to_vec().into_boxed_slice()
        };
        self.sample_overrides = overrides;
        Ok(())
    }

    pub(super) fn clear_sample_overrides_for(
        &mut self,
        identities: impl IntoIterator<Item = worth_ui_host_contract::UiMountedPaintCommandIdentity>,
    ) {
        for identity in identities {
            self.sample_overrides.remove(&identity);
        }
    }

    pub(super) fn sample_overrides(
        &self,
    ) -> impl Iterator<Item = worth_ui_host_contract::UiMountedPresentationSampleChange> + '_ {
        self.sample_overrides.values().copied()
    }

    pub(super) fn sample_damage(&self) -> &[worth_ui_host_contract::UiMountedLogicalDamage] {
        &self.sample_damage
    }
}

fn malformed() -> worth_ui_host_contract::UiHostSurfacePresentationDenial {
    worth_ui_host_contract::UiHostSurfacePresentationDenial::MalformedProjection
}
