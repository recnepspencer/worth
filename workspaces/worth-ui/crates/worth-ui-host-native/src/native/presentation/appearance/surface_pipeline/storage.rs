use super::{
    border_edge_bits, color_vector, micros_to_pixels, UiNativeSurfacePaintKind,
    UiNativeSurfacePrimitive,
};
use worth_ui_host_contract::UiMountedSurfaceBorderSide;

impl UiNativeSurfacePrimitive {
    pub(super) fn raster_storage(&self) -> Box<[f32]> {
        let mut storage = Vec::with_capacity(28 + self.border_omissions.len() * 4);
        storage.extend(self.allocation_edges_pixels());
        storage.extend([
            self.clip.left as f32,
            self.clip.top as f32,
            self.clip.right as f32,
            self.clip.bottom as f32,
        ]);
        storage.extend(self.radii.map(micros_to_pixels));
        storage.extend(color_vector(self.fill.map(|fill| fill.colors()[0])));
        storage.extend(color_vector(self.border_color));
        storage.extend([
            micros_to_pixels(self.border_width),
            f32::from(self.opacity) / f32::from(u16::MAX),
            match self.paint_kind {
                UiNativeSurfacePaintKind::Fill => 1.0,
                UiNativeSurfacePaintKind::Border => 2.0,
                UiNativeSurfacePaintKind::FillAndBorder => 3.0,
            },
            f32::from(border_edge_bits(self.border_edges)),
        ]);
        let (geometry_header, geometry_rows) = self.geometry.storage();
        storage.extend([
            self.border_omissions.len() as f32,
            geometry_header[0],
            geometry_header[1],
            geometry_header[2],
        ]);
        storage.extend(self.border_omissions.iter().flat_map(|omission| {
            [
                match omission.side {
                    UiMountedSurfaceBorderSide::Top => 0.0,
                    UiMountedSurfaceBorderSide::Right => 1.0,
                    UiMountedSurfaceBorderSide::Bottom => 2.0,
                    UiMountedSurfaceBorderSide::Left => 3.0,
                },
                micros_to_pixels(omission.start),
                micros_to_pixels(omission.end),
                0.0,
            ]
        }));
        storage.extend(geometry_rows);
        // A fixed trailer keeps geometry/omission offsets stable. Endpoints
        // remain allocation-relative when Motion transforms that allocation.
        let axis = match self.fill {
            Some(worth_ui_host_contract::UiMountedSurfaceFill::LinearGradient(gradient)) => {
                let [sx, sy] = gradient.start().coordinates();
                let [ex, ey] = gradient.end().coordinates();
                [sx, sy, ex, ey].map(|v| f32::from(v) / 10_000.0)
            }
            _ => [0.0; 4],
        };
        storage.extend(axis);
        storage.extend(color_vector(self.fill.map(|fill| fill.colors()[1])));
        storage.into_boxed_slice()
    }
}
