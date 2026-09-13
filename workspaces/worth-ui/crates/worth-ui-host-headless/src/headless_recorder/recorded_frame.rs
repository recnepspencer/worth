use std::collections::BTreeMap;
use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedLogicalDamage, UiMountedPaintCommand,
    UiMountedPaintCommandChange, UiMountedPaintOrderEdit, UiMountedPaintOrderIntegrity,
    UiMountedPresentationAuxiliaryState,
};

use crate::headless_transcript::UiHeadlessTranscriptSuccessorIdentity;
use crate::{UiHeadlessMountedFrameTranscript, UiHeadlessRecorderCapacity};

#[derive(Clone)]
pub(super) enum UiHeadlessRecordedFrame {
    Complete(UiHeadlessMountedFrameTranscript),
    Delta(UiHeadlessRecordedDelta),
    Appearance {
        identity: UiHeadlessTranscriptSuccessorIdentity,
        work: crate::headless_transcript::appearance::UiHeadlessAppearancePresentationTranscript,
    },
}

#[derive(Clone)]
pub(super) struct UiHeadlessRecordedDelta {
    identity: UiHeadlessTranscriptSuccessorIdentity,
    changes: Box<[UiMountedPaintCommandChange]>,
    order: Box<[UiMountedPaintOrderEdit]>,
    order_integrity: UiMountedPaintOrderIntegrity,
    damage: Box<[UiMountedLogicalDamage]>,
    nodes: Box<[worth_ui_host_contract::UiMountedPresentationNodeChange]>,
    auxiliary: Option<UiMountedPresentationAuxiliaryState>,
    semantic_text: Box<
        [(
            worth_ui_host_contract::UiMountedPaintCommandIdentity,
            crate::headless_transcript::UiHeadlessSemanticTextMechanic,
        )],
    >,
    appearance:
        Option<crate::headless_transcript::appearance::UiHeadlessAppearancePresentationTranscript>,
    capacity: UiHeadlessRecorderCapacity,
}

impl UiHeadlessRecordedFrame {
    pub(super) fn complete(transcript: UiHeadlessMountedFrameTranscript) -> Self {
        Self::Complete(transcript)
    }

    pub(super) fn appearance(
        view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    ) -> Result<Option<Self>, UiHostSurfacePresentationDenial> {
        let Some(work) = crate::headless_translation::translate_view_appearance(view)? else {
            return Ok(None);
        };
        Ok(Some(Self::Appearance {
            identity: UiHeadlessTranscriptSuccessorIdentity {
                host_session_identity: view.host_session_identity(),
                protocol: view.protocol(),
                attempt: view.attempt(),
                frame: view.frame(),
                binding: view.binding(),
            },
            work,
        }))
    }

    pub(super) fn delta(
        view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
        delta: &worth_ui_host_contract::UiMountedPresentationDelta,
        capacity: UiHeadlessRecorderCapacity,
    ) -> Result<Self, UiHostSurfacePresentationDenial> {
        let semantic_text = delta
            .changes()
            .iter()
            .filter_map(|change| match change {
                UiMountedPaintCommandChange::Insert(UiMountedPaintCommand::SemanticText {
                    identity,
                    mechanic,
                })
                | UiMountedPaintCommandChange::Replace {
                    successor: UiMountedPaintCommand::SemanticText { identity, mechanic },
                    ..
                } => Some((*identity, mechanic)),
                _ => None,
            })
            .map(|(identity, mechanic)| {
                crate::headless_translation::semantic_text::translate_command(view, mechanic)
                    .map(|translated| (identity, translated))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::Delta(UiHeadlessRecordedDelta {
            identity: UiHeadlessTranscriptSuccessorIdentity {
                host_session_identity: view.host_session_identity(),
                protocol: view.protocol(),
                attempt: view.attempt(),
                frame: view.frame(),
                binding: view.binding(),
            },
            changes: delta.changes().into(),
            order: delta.order().into(),
            order_integrity: delta.order_integrity(),
            damage: delta.damage().into(),
            nodes: delta.nodes().into(),
            auxiliary: delta.auxiliary().cloned(),
            semantic_text: semantic_text.into_boxed_slice(),
            appearance: crate::headless_translation::translate_view_appearance(view)?,
            capacity,
        }))
    }

    fn materialize(
        &self,
        predecessor: Option<&UiHeadlessMountedFrameTranscript>,
    ) -> Result<UiHeadlessMountedFrameTranscript, UiHostSurfacePresentationDenial> {
        match self {
            Self::Complete(transcript) => Ok(transcript.clone()),
            Self::Delta(delta) => delta.materialize(
                predecessor.ok_or(UiHostSurfacePresentationDenial::MalformedProjection)?,
            ),
            Self::Appearance { identity, work } => predecessor
                .ok_or(UiHostSurfacePresentationDenial::MalformedProjection)?
                .successor_appearance(*identity, work.clone()),
        }
    }

    fn binding(&self) -> worth_ui_host_contract::UiSurfaceBindingGeneration {
        match self {
            Self::Complete(transcript) => transcript.binding(),
            Self::Delta(delta) => delta.identity.binding,
            Self::Appearance { identity, .. } => identity.binding,
        }
    }
}

impl UiHeadlessRecordedDelta {
    fn materialize(
        &self,
        predecessor: &UiHeadlessMountedFrameTranscript,
    ) -> Result<UiHeadlessMountedFrameTranscript, UiHostSurfacePresentationDenial> {
        let mut commands = predecessor.successor_recorded_delta(
            self.identity,
            &self.changes,
            &self.order,
            self.order_integrity,
            &self.damage,
            &self.nodes,
            &self.semantic_text,
        )?;
        if let Some(appearance) = &self.appearance {
            commands.replace_appearance_work(appearance.clone())?;
        }
        let Some(auxiliary) = &self.auxiliary else {
            return Ok(commands);
        };
        let projection = auxiliary
            .reconstruct_authored()
            .map_err(|_| UiHostSurfacePresentationDenial::MalformedProjection)?;
        crate::headless_translation::translate_auxiliary_delta(
            self.identity,
            &projection,
            &commands,
            self.capacity,
            commands.paint_order(),
            &self.damage,
        )
    }
}

pub(super) fn materialize_frames(
    records: impl IntoIterator<Item = UiHeadlessRecordedFrame>,
    checkpoints: &mut BTreeMap<
        worth_ui_host_contract::UiSurfaceBindingGeneration,
        UiHeadlessMountedFrameTranscript,
    >,
) -> Result<Box<[UiHeadlessMountedFrameTranscript]>, UiHostSurfacePresentationDenial> {
    let mut transcripts = Vec::new();
    for record in records {
        let binding = record.binding();
        let transcript = record.materialize(checkpoints.get(&binding))?;
        checkpoints.insert(binding, transcript.clone());
        transcripts.push(transcript);
    }
    Ok(transcripts.into_boxed_slice())
}
