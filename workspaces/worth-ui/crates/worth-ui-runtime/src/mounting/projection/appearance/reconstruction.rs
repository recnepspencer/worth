use worth_ui_host_contract::{UiMountedAppearanceMechanicChange, UiMountedAppearanceWorkPosture};

use super::delta::{
    semantic_facts_changed, UiMountedAppearanceDelta, UiMountedAppearanceDeltaSummary,
};
use super::fact::{UiMountedAppearanceFacts, UiMountedAppearanceLoweringInput};
use super::{lowering, UiMountedAppearanceLoweringDenial};

pub(super) fn rebuild(
    predecessor: Option<&UiMountedAppearanceFacts>,
    input: UiMountedAppearanceLoweringInput,
) -> Result<(UiMountedAppearanceDelta, UiMountedAppearanceFacts), UiMountedAppearanceLoweringDenial>
{
    let predecessor = predecessor.ok_or(UiMountedAppearanceLoweringDenial::WorkConstruction)?;
    let successor = lowering::lower(input)?;
    let order_changed = predecessor.frame().overlay_order() != successor.frame().overlay_order();
    let changes = reconstruction_changes(predecessor, &successor);
    let damage =
        super::damage::damage_for_change(Some(predecessor), &successor, &changes, order_changed)?;
    let predecessor_manifest =
        worth_ui_host_contract::UiMountedAppearancePredecessorManifest::from_runtime_mounting(
            predecessor
                .records()
                .iter()
                .map(|record| record.identity().clone()),
            predecessor
                .frame()
                .overlay_order()
                .bottom_to_top()
                .iter()
                .cloned(),
        )
        .ok_or(UiMountedAppearanceLoweringDenial::WorkConstruction)?;
    let work = worth_ui_host_contract::UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Reconstruction,
        Some(predecessor.frame().frame()),
        Some(predecessor_manifest),
        successor.frame().clone(),
        changes,
        damage,
        order_changed,
    )
    .ok_or(UiMountedAppearanceLoweringDenial::WorkConstruction)?;
    let mechanics_changed = !work.changes().is_empty() || order_changed;
    Ok((
        UiMountedAppearanceDelta {
            work,
            summary: UiMountedAppearanceDeltaSummary {
                semantic_facts_changed: semantic_facts_changed(Some(predecessor), &successor),
                mechanics_changed,
                output_suppressed: false,
                order_changed,
            },
        },
        successor,
    ))
}

fn reconstruction_changes(
    predecessor: &UiMountedAppearanceFacts,
    successor: &UiMountedAppearanceFacts,
) -> Vec<UiMountedAppearanceMechanicChange> {
    let mut changes = predecessor
        .records()
        .iter()
        .filter_map(|record| {
            let successor_record = successor.record(record.identity());
            match successor_record {
                None => Some(UiMountedAppearanceMechanicChange::Remove(
                    record.identity().clone(),
                )),
                Some(successor_record) => Some(
                    UiMountedAppearanceMechanicChange::replacement(
                        record.identity().clone(),
                        successor_record.mechanic().clone(),
                    )
                    .expect("same record identity produces a reconstruction replacement"),
                ),
            }
        })
        .collect::<Vec<_>>();
    changes.extend(
        successor
            .records()
            .iter()
            .filter(|record| predecessor.record(record.identity()).is_none())
            .map(|record| UiMountedAppearanceMechanicChange::Insert(record.mechanic().clone())),
    );
    changes
}
