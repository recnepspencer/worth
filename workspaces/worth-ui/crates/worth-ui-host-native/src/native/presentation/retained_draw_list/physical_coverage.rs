//! Derived image coverage for native retained replay; atlas addresses are not retained here.
use std::collections::HashMap;

#[path = "physical_coverage/damage.rs"]
pub(super) mod damage;

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedPaintCommandIdentity,
};

use super::super::damage_index::{UiNativeDamageIndex, UiNativeDamageQuery};
use super::super::raster::UiNativeRasterBasis;
use super::UiNativeRetainedDrawListDenial as Denial;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UiNativeCommandImageCoverage {
    pub(super) base: Box<[[f32; 4]]>,
    pub(super) current: Box<[[f32; 4]]>,
}

pub(super) struct UiNativePhysicalCoverage {
    pub(super) basis: UiNativeRasterBasis,
    records: HashMap<UiMountedPaintCommandIdentity, UiNativeCommandImageCoverage>,
    index: UiNativeDamageIndex<UiMountedPaintCommandIdentity>,
}

impl UiNativePhysicalCoverage {
    pub(super) fn new(basis: UiNativeRasterBasis) -> Self {
        Self {
            basis,
            records: HashMap::new(),
            index: UiNativeDamageIndex::new(),
        }
    }

    pub(super) fn get(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Option<&UiNativeCommandImageCoverage> {
        self.records.get(&identity)
    }

    pub(super) fn replace(
        &mut self,
        identity: UiMountedPaintCommandIdentity,
        replacement: Option<UiNativeCommandImageCoverage>,
    ) -> Result<(), Denial> {
        let old = self
            .records
            .get(&identity)
            .map(|record| candidate_bounds(&record.current))
            .transpose()?
            .flatten();
        let new = replacement
            .as_ref()
            .map(|record| candidate_bounds(&record.current))
            .transpose()?
            .flatten();
        match (old, new) {
            (Some(_), Some(bounds)) => self.index.replace(identity, bounds)?,
            (Some(_), None) => self.index.remove(identity)?,
            (None, Some(bounds)) => self.index.insert(identity, bounds)?,
            (None, None) => {}
        }
        match replacement {
            Some(record) => {
                self.records.insert(identity, record);
            }
            None => {
                self.records.remove(&identity);
            }
        }
        Ok(())
    }

    pub(super) fn intersecting(
        &self,
        damage: [f32; 4],
    ) -> Result<UiNativeDamageQuery<UiMountedPaintCommandIdentity>, Denial> {
        let bounds = candidate_bounds(&[damage])?.ok_or(Denial::CommandMismatch)?;
        let mut query = self.index.intersecting(bounds)?;
        query.identities.retain(|identity| {
            self.records.get(identity).is_some_and(|record| {
                record
                    .current
                    .iter()
                    .any(|image| intersects(*image, damage))
            })
        });
        Ok(query)
    }
}

fn candidate_bounds(images: &[[f32; 4]]) -> Result<Option<UiMountedCanonicalBox>, Denial> {
    if images.is_empty() {
        return Ok(None);
    }
    let mut bounds = [
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    ];
    for &[x, y, width, height] in images {
        if [x, y, width, height, x + width, y + height]
            .iter()
            .any(|v| !v.is_finite())
            || width <= 0.0
            || height <= 0.0
        {
            return Err(Denial::CommandMismatch);
        }
        bounds[0] = bounds[0].min(x.floor());
        bounds[1] = bounds[1].min(y.floor());
        bounds[2] = bounds[2].max((x + width).ceil());
        bounds[3] = bounds[3].max((y + height).ceil());
    }
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: bounds[0],
        y: bounds[1],
        width: bounds[2] - bounds[0],
        height: bounds[3] - bounds[1],
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .map(Some)
    .map_err(|_| Denial::CommandMismatch)
}

fn intersects(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}
