//! Inspect the actual native replay plan using runtime-owned complete text and an admitted atlas.
use super::super::raster::{raster_damage_for_basis, UiNativeRasterBasis};
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use worth_ui_host_contract::*;

impl UiNativeRetainedDrawList {
    pub(crate) fn physical_replay_for_certification(
        commands: &[UiMountedPaintCommand],
        order: &[UiMountedPaintOrderIdentity],
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
        damage: UiMountedLogicalDamage,
        atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    ) -> Result<
        (
            Box<[UiMountedPaintCommandIdentity]>,
            Box<[Option<UiMountedPaintCommandIdentity>]>,
        ),
        Denial,
    > {
        let mut retained =
            Self::from_text_view_for_certification(commands, order, view, extent, atlas)?;
        let basis = retained
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?
            .basis;
        let clear = raster_damage_for_basis(damage.bounds(), basis)
            .map_err(|_| Denial::CommandMismatch)?
            .ok_or(Denial::CommandMismatch)?;
        let selected = retained.physical_replay_for_damage(
            basis,
            clear.physical_bounds(),
            &mut Default::default(),
        )?;
        let replay = retained.replay_plan(&[damage], 0, 0)?;
        let plan =
            super::super::retained_raster::build_plan(basis, &mut retained, replay, 0, atlas)
                .map_err(|_| Denial::CommandMismatch)?;
        let draws = plan
            .operations
            .iter()
            .filter_map(|operation| match operation {
                crate::native::presentation::UiNativeRasterOperation::Glyph(glyph) => {
                    Some(Some(glyph.run.mechanic()))
                }
                crate::native::presentation::UiNativeRasterOperation::FilledRect { .. } => {
                    Some(None)
                }
                crate::native::presentation::UiNativeRasterOperation::Clear(_) => None,
                crate::native::presentation::UiNativeRasterOperation::Surface(_) => Some(None),
            })
            .collect();
        Ok((selected, draws))
    }
    pub(crate) fn from_text_view_for_certification(
        commands: &[UiMountedPaintCommand],
        order: &[UiMountedPaintOrderIdentity],
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
        atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    ) -> Result<Self, Denial> {
        let affinity = view.presentation_work().affinity();
        let work = view.text_raster_work().ok_or(Denial::CommandMismatch)?;
        for command in commands {
            if let UiMountedPaintCommand::SemanticText { identity, mechanic } = command {
                let demand = work
                    .demands()
                    .iter()
                    .find(|demand| demand.layout_identity() == mechanic.qualified_layout_identity())
                    .ok_or(Denial::CommandMismatch)?;
                let runs = work
                    .glyph_runs()
                    .iter()
                    .copied()
                    .filter(|run| run.mechanic() == *identity)
                    .collect::<Vec<_>>();
                work.validate_complete_demand(*identity, *demand, &runs)
                    .map_err(|_| Denial::CommandMismatch)?;
            }
        }
        let mut retained = Self::from_complete(
            affinity.successor(),
            affinity.surface(),
            affinity.binding(),
            affinity.content(),
            affinity.baseline(),
            commands,
            order,
            UiMountedPaintOrderIntegrity::for_order(order),
            work.glyph_runs(),
        )?;
        let basis = UiNativeRasterBasis::new(
            extent,
            view.requirement().device_scale_milli() as f32 / 1_000.0,
        );
        retained.initialize_physical_coverage(basis, atlas)?;
        Ok(retained)
    }
}
