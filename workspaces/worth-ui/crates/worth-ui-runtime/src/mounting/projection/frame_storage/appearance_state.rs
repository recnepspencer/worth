use crate::runtime::appearance::{
    UiAppearanceChangeReceipt, UiAppearanceInspectionDenial, UiAppearanceInspectionRecord,
    UiAppearanceMountAffinity, UiAppearanceProjectionAttempt,
};

const APPEARANCE_STATE_CAPACITY: usize = 2_048;

#[derive(Clone, Default)]
pub(crate) struct UiMountedAppearanceFrameState {
    entries: Vec<UiMountedAppearanceStateEntry>,
    staged: Vec<UiAppearanceProjectionAttempt>,
}

#[derive(Clone)]
struct UiMountedAppearanceStateEntry {
    key: UiMountedAppearanceStateKey,
    projection: crate::runtime::appearance::UiAppearanceProjection,
    sidecar: super::super::appearance::UiMountedAppearanceSidecar,
}

#[derive(Clone, Eq, PartialEq)]
struct UiMountedAppearanceStateKey {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
}

impl UiMountedAppearanceFrameState {
    pub(crate) fn inherit_from(&mut self, predecessor: Option<&Self>) {
        self.entries = predecessor.map_or_else(Vec::new, |state| state.entries.clone());
        self.staged.clear();
    }

    pub(crate) fn stage(&mut self, attempt: UiAppearanceProjectionAttempt) {
        if self.staged.len() < APPEARANCE_STATE_CAPACITY {
            self.staged.push(attempt);
        }
    }

    pub(crate) fn lower(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Vec<UiAppearanceInspectionRecord> {
        let staged = std::mem::take(&mut self.staged);
        let mut records = Vec::with_capacity(staged.len());
        for attempt in staged {
            let context = attempt.context();
            let Some(projection) = attempt.projection() else {
                records.push(denial_record(
                    context.clone(),
                    attempt
                        .denial()
                        .unwrap_or(UiAppearanceInspectionDenial::Basis),
                ));
                continue;
            };
            let key = state_key(context);
            let predecessor = self
                .entries
                .iter()
                .find(|entry| entry.key == key)
                .map(|entry| (entry.projection.clone(), entry.sidecar.clone()));
            let mut sidecar = predecessor
                .as_ref()
                .map_or_else(Default::default, |(_, sidecar)| sidecar.clone());
            let input = match context.lower_resolved(projection, presentation) {
                Ok(input) => input,
                Err(_) => {
                    records.push(denial_record(
                        context.clone(),
                        UiAppearanceInspectionDenial::MountLowering,
                    ));
                    continue;
                }
            };
            let affinity = UiAppearanceMountAffinity {
                session: context.target().session(),
                generation: context.generation().clone(),
                frame: context.frame(),
                surface: context.semantic_surface(),
                graph_node: context.graph_node(),
                mounted_instance: context.mounted_instance(),
                incarnation: context.incarnation(),
                node_receipt: context.node_receipt(),
                issuer: context.issuer(),
                presentation,
            };
            let work = match sidecar.mount(input) {
                Ok(work) => work,
                Err(_) => {
                    records.push(denial_record(
                        context.clone(),
                        UiAppearanceInspectionDenial::MountLowering,
                    ));
                    continue;
                }
            };
            let predecessor_projection = predecessor.as_ref().map(|(projection, _)| projection);
            let receipt = match UiAppearanceChangeReceipt::from_resolved_mount(
                predecessor_projection,
                projection,
                &work,
                affinity,
            ) {
                Ok(receipt) => receipt,
                Err(_) => {
                    records.push(denial_record(
                        context.clone(),
                        UiAppearanceInspectionDenial::MountAffinity,
                    ));
                    continue;
                }
            };
            replace_entry(
                &mut self.entries,
                UiMountedAppearanceStateEntry {
                    key,
                    projection: projection.clone(),
                    sidecar,
                },
            );
            records.push(UiAppearanceInspectionRecord::Projection {
                projection: projection.clone(),
                consumers_selected: context.consumers_selected(),
                receipt,
            });
        }
        records
    }
}

fn state_key(
    context: &crate::runtime::appearance::UiAppearanceAttemptContext,
) -> UiMountedAppearanceStateKey {
    UiMountedAppearanceStateKey {
        session: context.target().session(),
        generation: context.generation().clone(),
        surface: context.semantic_surface(),
        graph_node: context.graph_node(),
        mounted_instance: context.mounted_instance(),
        incarnation: context.incarnation(),
    }
}

fn replace_entry(
    entries: &mut Vec<UiMountedAppearanceStateEntry>,
    successor: UiMountedAppearanceStateEntry,
) {
    if let Some(entry) = entries.iter_mut().find(|entry| entry.key == successor.key) {
        *entry = successor;
    } else if entries.len() < APPEARANCE_STATE_CAPACITY {
        entries.push(successor);
    }
}

fn denial_record(
    context: crate::runtime::appearance::UiAppearanceAttemptContext,
    denial: UiAppearanceInspectionDenial,
) -> UiAppearanceInspectionRecord {
    UiAppearanceInspectionRecord::Denial {
        context,
        denial,
        receipt: UiAppearanceChangeReceipt::for_denial(),
    }
}
