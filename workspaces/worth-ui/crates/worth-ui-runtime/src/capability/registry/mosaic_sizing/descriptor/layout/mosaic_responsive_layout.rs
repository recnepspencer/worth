use std::collections::BTreeSet;

use super::{MosaicLayoutContract, MosaicLayoutDenial};

/// A half-open interval of logical viewport widths, `[min, max)`, or
/// unbounded above.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MosaicViewportWidthInterval {
    min_logical_points: u16,
    max_logical_points: Option<u16>,
}

impl MosaicViewportWidthInterval {
    pub const fn at_least(min_logical_points: u16) -> Self {
        Self {
            min_logical_points,
            max_logical_points: None,
        }
    }

    pub const fn between(
        min_logical_points: u16,
        max_logical_points: u16,
    ) -> Result<Self, MosaicLayoutDenial> {
        if min_logical_points >= max_logical_points {
            return Err(MosaicLayoutDenial::EmptyViewportInterval);
        }
        Ok(Self {
            min_logical_points,
            max_logical_points: Some(max_logical_points),
        })
    }

    pub const fn min_logical_points(self) -> u16 {
        self.min_logical_points
    }

    pub const fn max_logical_points(self) -> Option<u16> {
        self.max_logical_points
    }

    pub fn contains(self, viewport_width_logical_points: f32) -> bool {
        viewport_width_logical_points >= f32::from(self.min_logical_points)
            && self
                .max_logical_points
                .is_none_or(|max| viewport_width_logical_points < f32::from(max))
    }

    fn overlaps(self, other: Self) -> bool {
        let ends_by =
            |interval: Self, min: u16| interval.max_logical_points.is_some_and(|max| max <= min);
        !ends_by(self, other.min_logical_points) && !ends_by(other, self.min_logical_points)
    }
}

/// A container layout that selects its tracks by viewport width.
///
/// Variants hold ordered, nonoverlapping viewport-width intervals; a width no
/// variant admits selects the required fallback. Every variant places the
/// same members as the fallback, so crossing an interval edge moves members
/// between cells without mounting or unmounting any of them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MosaicResponsiveLayout {
    fallback: MosaicLayoutContract,
    variants: Vec<(MosaicViewportWidthInterval, MosaicLayoutContract)>,
}

impl MosaicResponsiveLayout {
    pub const fn new(fallback: MosaicLayoutContract) -> Self {
        Self {
            fallback,
            variants: Vec::new(),
        }
    }

    pub fn with_variant(
        mut self,
        interval: MosaicViewportWidthInterval,
        layout: MosaicLayoutContract,
    ) -> Result<Self, MosaicLayoutDenial> {
        if self
            .variants
            .iter()
            .any(|(declared, _)| declared.overlaps(interval))
        {
            return Err(MosaicLayoutDenial::OverlappingViewportIntervals);
        }
        if member_set(&layout) != member_set(&self.fallback) {
            return Err(MosaicLayoutDenial::VariantMembershipMismatch);
        }
        let index = self
            .variants
            .partition_point(|(declared, _)| *declared < interval);
        self.variants.insert(index, (interval, layout));
        Ok(self)
    }

    /// The layout that applies at one logical viewport width.
    pub fn select(&self, viewport_width_logical_points: f32) -> &MosaicLayoutContract {
        self.variants
            .iter()
            .find(|(interval, _)| interval.contains(viewport_width_logical_points))
            .map_or(&self.fallback, |(_, layout)| layout)
    }

    pub const fn fallback(&self) -> &MosaicLayoutContract {
        &self.fallback
    }

    pub fn variants(
        &self,
    ) -> impl Iterator<Item = (MosaicViewportWidthInterval, &MosaicLayoutContract)> {
        self.variants
            .iter()
            .map(|(interval, layout)| (*interval, layout))
    }

    pub(crate) fn digest_basis(&self) -> String {
        let variants = self
            .variants
            .iter()
            .map(|(interval, layout)| {
                let max = interval
                    .max_logical_points
                    .map_or_else(|| "open".to_owned(), |max| max.to_string());
                format!(
                    "{}..{max}={}",
                    interval.min_logical_points,
                    layout.digest_basis()
                )
            })
            .collect::<Vec<_>>()
            .join(";");
        format!("responsive:{}:[{variants}]", self.fallback.digest_basis())
    }
}

impl From<MosaicLayoutContract> for MosaicResponsiveLayout {
    fn from(fallback: MosaicLayoutContract) -> Self {
        Self::new(fallback)
    }
}

fn member_set(layout: &MosaicLayoutContract) -> BTreeSet<&str> {
    layout
        .members()
        .map(|(component, _)| component.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{MosaicResponsiveLayout, MosaicViewportWidthInterval};
    use crate::capability::{
        ComponentId, MosaicLayoutCell, MosaicLayoutContract, MosaicLayoutDenial, MosaicTrack,
    };

    fn stacked() -> MosaicLayoutContract {
        MosaicLayoutContract::rows([
            MosaicTrack::flex(1, 0).unwrap(),
            MosaicTrack::flex(1, 0).unwrap(),
        ])
        .unwrap()
        .with_member(id("demo.component.chart"), MosaicLayoutCell::at(0, 0))
        .unwrap()
        .with_member(id("demo.component.activity"), MosaicLayoutCell::at(0, 1))
        .unwrap()
    }

    fn columns() -> MosaicLayoutContract {
        MosaicLayoutContract::columns([
            MosaicTrack::flex(2, 480).unwrap(),
            MosaicTrack::flex(1, 320).unwrap(),
        ])
        .unwrap()
        .with_member(id("demo.component.chart"), MosaicLayoutCell::at(0, 0))
        .unwrap()
        .with_member(id("demo.component.activity"), MosaicLayoutCell::at(1, 0))
        .unwrap()
    }

    fn id(value: &str) -> ComponentId {
        ComponentId::new(value).unwrap()
    }

    #[test]
    fn selection_uses_half_open_intervals_and_falls_back_outside_them() {
        let layout = MosaicResponsiveLayout::new(stacked())
            .with_variant(MosaicViewportWidthInterval::at_least(1200), columns())
            .unwrap();
        assert_eq!(layout.select(1199.5), &stacked());
        assert_eq!(layout.select(1200.0), &columns());
        assert_eq!(layout.select(4000.0), &columns());
    }

    #[test]
    fn variants_reject_overlap_empty_intervals_and_membership_changes() {
        let layout = MosaicResponsiveLayout::new(stacked())
            .with_variant(
                MosaicViewportWidthInterval::between(800, 1200).unwrap(),
                columns(),
            )
            .unwrap();
        assert_eq!(
            layout
                .clone()
                .with_variant(MosaicViewportWidthInterval::at_least(1199), columns()),
            Err(MosaicLayoutDenial::OverlappingViewportIntervals)
        );
        assert!(layout
            .clone()
            .with_variant(MosaicViewportWidthInterval::at_least(1200), columns())
            .is_ok());
        assert_eq!(
            MosaicViewportWidthInterval::between(1200, 1200),
            Err(MosaicLayoutDenial::EmptyViewportInterval)
        );
        let partial = MosaicLayoutContract::frame()
            .unwrap()
            .with_member(id("demo.component.chart"), MosaicLayoutCell::at(0, 0))
            .unwrap();
        assert_eq!(
            layout.with_variant(MosaicViewportWidthInterval::at_least(1200), partial),
            Err(MosaicLayoutDenial::VariantMembershipMismatch)
        );
    }

    #[test]
    fn variant_order_is_canonical() {
        let wide = MosaicViewportWidthInterval::at_least(1200);
        let middle = MosaicViewportWidthInterval::between(800, 1200).unwrap();
        let forward = MosaicResponsiveLayout::new(stacked())
            .with_variant(middle, columns())
            .and_then(|layout| layout.with_variant(wide, columns()))
            .unwrap();
        let reverse = MosaicResponsiveLayout::new(stacked())
            .with_variant(wide, columns())
            .and_then(|layout| layout.with_variant(middle, columns()))
            .unwrap();
        assert_eq!(forward, reverse);
        assert_eq!(forward.digest_basis(), reverse.digest_basis());
    }
}
