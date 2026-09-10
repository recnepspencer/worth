use super::{UiNativeSurfacePrimitive, UiNativeSurfaceRasterOperation};
use crate::native::presentation::{raster::raster_physical_bounds, RasterRect};

impl UiNativeSurfaceRasterOperation {
    pub(crate) const fn rect(&self) -> RasterRect {
        self.rect
    }

    pub(crate) fn storage_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.storage.len() * std::mem::size_of::<f32>());
        for value in &self.storage {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
        bytes
    }

    pub(crate) fn clipped_to(mut self, clip: RasterRect, extent: [u32; 2]) -> Option<Self> {
        let rect = self.rect.intersection(clip, extent)?;
        let [left, top, width, height] = rect.physical_bounds();
        let right = left + width;
        let bottom = top + height;
        self.storage[4..8].copy_from_slice(&[left as f32, top as f32, right as f32, bottom as f32]);
        self.rect = rect;
        Some(self)
    }

    pub(in crate::native::presentation) fn with_sample(
        mut self,
        sample: worth_ui_host_contract::UiMountedPresentationSampleChange,
        basis: crate::native::presentation::raster::UiNativeRasterBasis,
    ) -> Result<Option<Self>, worth_ui_host_contract::UiHostSurfacePresentationDenial> {
        if sample.opacity().units() == 0 {
            return Ok(None);
        }
        self.storage[21] = sample.opacity().factor();
        if let Some(transform) = sample.transform() {
            let transform_box = |edges: [f32; 4]| {
                crate::native::presentation::sample::transform_physical_box(
                    [edges[0], edges[1], edges[2] - edges[0], edges[3] - edges[1]],
                    transform,
                    basis,
                )
                .map(|rect| [rect[0], rect[1], rect[0] + rect[2], rect[1] + rect[3]])
            };
            let allocation = transform_box(self.storage[0..4].try_into().expect("four edges"))?;
            let clip = transform_box(self.storage[4..8].try_into().expect("four edges"))?;
            let source = transform.source();
            let sampled = transform.sampled();
            let scale = (sampled.width() / source.width()).min(sampled.height() / source.height());
            for value in &mut self.storage[8..12] {
                *value *= scale;
            }
            self.storage[20] *= scale;
            let omission_count = self.storage[24] as usize;
            for row in self.storage[28..].chunks_exact_mut(4).take(omission_count) {
                let axis_scale = if row[0] == 0.0 || row[0] == 2.0 {
                    sampled.width() / source.width()
                } else {
                    sampled.height() / source.height()
                };
                row[1] *= axis_scale;
                row[2] *= axis_scale;
            }
            self.storage[0..4].copy_from_slice(&allocation);
            self.storage[4..8].copy_from_slice(&clip);
        }
        let allocation: [f32; 4] = self.storage[0..4].try_into().expect("four edges");
        let clip: [f32; 4] = self.storage[4..8].try_into().expect("four edges");
        let [width, height] = basis.extent();
        let edges = [
            allocation[0].max(clip[0]).clamp(0.0, width as f32).floor() as u32,
            allocation[1].max(clip[1]).clamp(0.0, height as f32).floor() as u32,
            allocation[2].min(clip[2]).clamp(0.0, width as f32).ceil() as u32,
            allocation[3].min(clip[3]).clamp(0.0, height as f32).ceil() as u32,
        ];
        self.rect = match raster_physical_bounds(edges, basis.extent()) {
            Some(rect) => rect,
            None => return Ok(None),
        };
        Ok(Some(self))
    }
}

impl UiNativeSurfacePrimitive {
    pub(in crate::native::presentation) fn raster_operation_with_sample(
        &self,
        sample: worth_ui_host_contract::UiMountedPresentationSampleChange,
        basis: crate::native::presentation::raster::UiNativeRasterBasis,
    ) -> Result<
        Option<UiNativeSurfaceRasterOperation>,
        worth_ui_host_contract::UiHostSurfacePresentationDenial,
    > {
        let placeholder = raster_physical_bounds([0, 0, 1, 1], basis.extent())
            .ok_or(worth_ui_host_contract::UiHostSurfacePresentationDenial::MalformedProjection)?;
        UiNativeSurfaceRasterOperation {
            rect: placeholder,
            storage: self.raster_storage(),
        }
        .with_sample(sample, basis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::presentation::appearance::{
        mounted_mechanic_fixtures::{
            allocation, logical_length, mounted_outline, mounted_surface,
            MountedOutlineFixtureInput, MountedSurfaceFixtureInput,
        },
        UiNativeAppearanceScale, UiNativeOutlinePipeline, UiNativeSurfacePipeline,
    };
    use worth_ui_host_contract::{
        UiAppearanceClip, UiMountedAppearanceColor, UiMountedCanonicalBox,
        UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedPaintCommandIdentity,
        UiMountedPresentationOpacity, UiMountedPresentationSampleChange,
        UiMountedPresentationTransform, UiMountedSurfacePaint,
    };

    #[test]
    fn nonzero_origin_damage_clip_is_stored_as_edges() {
        let operation = UiNativeSurfaceRasterOperation {
            rect: raster_physical_bounds([80, 80, 140, 140], [200, 200]).unwrap(),
            storage: vec![0.0; 28].into_boxed_slice(),
        };
        let clip = raster_physical_bounds([100, 110, 120, 130], [200, 200]).unwrap();
        let clipped = operation.clipped_to(clip, [200, 200]).unwrap();

        assert_eq!(clipped.rect().physical_bounds(), [100.0, 110.0, 20.0, 20.0]);
        assert_eq!(&clipped.storage[4..8], &[100.0, 110.0, 120.0, 130.0]);
    }

    #[test]
    fn authored_outline_becomes_a_positive_analytic_ring() {
        let outline = UiNativeOutlinePipeline::prepare(
            &mounted_outline(MountedOutlineFixtureInput {
                allocation: allocation(20_000, 20_000, 40_000, 40_000),
                clip: UiAppearanceClip::new(0, 0, 100_000, 100_000).unwrap(),
                radii: [logical_length(12_000); 4],
                line_width: logical_length(2_000),
                offset: logical_length(0),
                anti_alias_fringe: logical_length(1_000),
                color: UiMountedAppearanceColor::from_straight_srgba([200, 100, 50, 255]),
                opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            }),
            UiNativeAppearanceScale::qualified(1_000).unwrap(),
        )
        .unwrap();
        let operation = UiNativeSurfacePipeline::prepare_outline(&outline)
            .raster_operation([100, 100])
            .unwrap()
            .unwrap();

        assert_eq!(operation.storage[20], 2.0);
        assert_eq!(operation.storage[8..12], [14.0; 4]);
    }

    #[test]
    fn accepted_motion_can_translate_an_offscreen_surface_into_view() {
        let surface = mounted_surface(MountedSurfaceFixtureInput {
            allocation: allocation(120_000, 0, 20_000, 20_000),
            clip: UiAppearanceClip::new(0, 0, 200_000, 100_000).unwrap(),
            radii: [logical_length(0); 4],
            paint: UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                20, 40, 60, 255,
            ])),
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
        });
        let primitive = UiNativeSurfacePipeline::prepare(
            &surface,
            UiNativeAppearanceScale::qualified(1_000).unwrap(),
        )
        .unwrap();
        assert!(primitive.raster_operation([100, 100]).unwrap().is_none());
        let bounds = |x| {
            UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                x,
                y: 0.0,
                width: 20.0,
                height: 20.0,
                coordinate_space: UiMountedCoordinateSpace::HostSurface,
            })
            .unwrap()
        };
        let sample = UiMountedPresentationSampleChange::from_runtime_sampling(
            UiMountedPaintCommandIdentity::filled_rect_from_correspondence(
                surface.node_receipt().mounted_instance(),
            ),
            Some(
                UiMountedPresentationTransform::from_runtime_sampling(bounds(120.0), bounds(20.0))
                    .unwrap(),
            ),
            UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
        );

        let operation = primitive
            .raster_operation_with_sample(
                sample,
                crate::native::presentation::raster::UiNativeRasterBasis::new([100, 100], 1.0),
            )
            .unwrap()
            .expect("the accepted transform moves the surface into the viewport");
        assert_eq!(operation.rect().physical_bounds(), [20.0, 0.0, 20.0, 20.0]);
    }
}
