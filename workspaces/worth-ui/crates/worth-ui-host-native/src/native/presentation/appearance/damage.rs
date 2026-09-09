use super::geometry::UiNativePhysicalPixelRect;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct UiNativeAppearanceDamageRect {
    pub(crate) left: i64,
    pub(crate) top: i64,
    pub(crate) right: i64,
    pub(crate) bottom: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeAppearanceDamageSetDenial {
    CapacityExceeded,
    Empty,
}

#[derive(Clone, Debug)]
pub(crate) struct UiNativeAppearanceDamage {
    regions: Vec<UiNativeAppearanceDamageRect>,
    capacity: usize,
}

impl UiNativeAppearanceDamageRect {
    pub(crate) fn from_pixel_rect(rect: UiNativePhysicalPixelRect) -> Self {
        Self {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }

    pub(crate) fn is_empty(self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    pub(crate) fn intersects(self, other: Self) -> bool {
        self.left < other.right
            && other.left < self.right
            && self.top < other.bottom
            && other.top < self.bottom
    }

    pub(crate) fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    fn coalesces(self, other: Self) -> bool {
        self.contains(other)
            || other.contains(self)
            || (self.top == other.top
                && self.bottom == other.bottom
                && self.left <= other.right
                && other.left <= self.right)
            || (self.left == other.left
                && self.right == other.right
                && self.top <= other.bottom
                && other.top <= self.bottom)
    }

    fn contains(self, other: Self) -> bool {
        self.left <= other.left
            && self.top <= other.top
            && self.right >= other.right
            && self.bottom >= other.bottom
    }

    fn sort_key(self) -> (i64, i64, i64, i64) {
        (self.top, self.left, self.bottom, self.right)
    }
}

impl UiNativeAppearanceDamage {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            regions: Vec::new(),
            capacity,
        }
    }

    pub(crate) fn add(
        &mut self,
        region: UiNativeAppearanceDamageRect,
    ) -> Result<(), UiNativeAppearanceDamageSetDenial> {
        if region.is_empty() {
            return Err(UiNativeAppearanceDamageSetDenial::Empty);
        }
        let mut merged = region;
        let mut index = 0;
        while index < self.regions.len() {
            if merged.coalesces(self.regions[index]) {
                merged = merged.union(self.regions.remove(index));
                index = 0;
            } else {
                index += 1;
            }
        }
        if self.regions.len() == self.capacity {
            return Err(UiNativeAppearanceDamageSetDenial::CapacityExceeded);
        }
        self.regions.push(merged);
        self.regions.sort_unstable_by_key(|left| left.sort_key());
        Ok(())
    }

    pub(crate) fn take(&mut self) -> Box<[UiNativeAppearanceDamageRect]> {
        let mut regions = std::mem::take(&mut self.regions);
        regions.sort_unstable_by_key(|left| left.sort_key());
        regions.into_boxed_slice()
    }

    pub(crate) fn regions(&self) -> &[UiNativeAppearanceDamageRect] {
        &self.regions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(left: i64, top: i64, right: i64, bottom: i64) -> UiNativeAppearanceDamageRect {
        UiNativeAppearanceDamageRect {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn overlapping_and_edge_adjacent_regions_normalize_in_stable_order() {
        let mut damage = UiNativeAppearanceDamage::new(4);
        damage.add(region(10, 0, 20, 10)).unwrap();
        damage.add(region(0, 0, 10, 10)).unwrap();
        damage.add(region(0, 10, 20, 20)).unwrap();
        assert_eq!(damage.take().as_ref(), &[region(0, 0, 20, 20)]);
    }

    #[test]
    fn corner_only_contact_does_not_claim_edge_adjacency() {
        let mut damage = UiNativeAppearanceDamage::new(4);
        damage.add(region(0, 0, 10, 10)).unwrap();
        damage.add(region(10, 10, 20, 20)).unwrap();
        assert_eq!(damage.regions().len(), 2);
    }

    #[test]
    fn offset_images_preserve_their_coverage_instead_of_filling_the_hull() {
        for images in [
            [region(0, 0, 3, 2), region(2, 1, 4, 4)],
            [region(0, 0, 2, 3), region(2, 1, 4, 2)],
            [region(0, 0, 1, 1), region(3, 3, 4, 4)],
        ] {
            for order in [images, [images[1], images[0]]] {
                let mut damage = UiNativeAppearanceDamage::new(2);
                for image in order {
                    damage.add(image).unwrap();
                }
                assert_eq!(damage.regions().len(), 2);
                for y in 0..4 {
                    for x in 0..4 {
                        let covers = |rect: &UiNativeAppearanceDamageRect| {
                            rect.left <= x && x < rect.right && rect.top <= y && y < rect.bottom
                        };
                        assert_eq!(
                            damage.regions().iter().any(covers),
                            images.iter().any(covers),
                            "damage changed image coverage at ({x}, {y})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn containment_coalesces_but_capacity_cannot_widen_coverage() {
        let outer = region(0, 0, 4, 4);
        let mut damage = UiNativeAppearanceDamage::new(1);
        damage.add(region(1, 1, 2, 2)).unwrap();
        damage.add(outer).unwrap();
        damage.add(region(2, 2, 3, 3)).unwrap();
        assert_eq!(damage.regions(), &[outer]);
        assert_eq!(
            damage.add(region(3, 3, 5, 5)),
            Err(UiNativeAppearanceDamageSetDenial::CapacityExceeded)
        );
        assert_eq!(damage.regions(), &[outer]);
    }
}
