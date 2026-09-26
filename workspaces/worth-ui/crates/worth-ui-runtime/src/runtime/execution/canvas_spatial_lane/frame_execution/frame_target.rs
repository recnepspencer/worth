use crate::runtime::{
    WorthUiCanvasViewportRequest, WorthUiLaneHandle, WorthUiSpatialHitTestRequest,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiCanvasSpatialFrameTarget {
    kind: WorthUiCanvasSpatialFrameTargetKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiCanvasSpatialFrameTargetKind {
    Viewport(WorthUiCanvasViewportRequest),
    Draw(WorthUiLaneHandle),
    HitTest(WorthUiSpatialHitTestRequest),
    Overlay(WorthUiLaneHandle),
    ToolState(WorthUiLaneHandle),
}

impl WorthUiCanvasSpatialFrameTarget {
    pub fn viewport(request: WorthUiCanvasViewportRequest) -> Self {
        Self {
            kind: WorthUiCanvasSpatialFrameTargetKind::Viewport(request),
        }
    }

    pub fn draw(lane_handle: WorthUiLaneHandle) -> Self {
        Self {
            kind: WorthUiCanvasSpatialFrameTargetKind::Draw(lane_handle),
        }
    }

    pub fn hit_test(request: WorthUiSpatialHitTestRequest) -> Self {
        Self {
            kind: WorthUiCanvasSpatialFrameTargetKind::HitTest(request),
        }
    }

    pub fn overlay(lane_handle: WorthUiLaneHandle) -> Self {
        Self {
            kind: WorthUiCanvasSpatialFrameTargetKind::Overlay(lane_handle),
        }
    }

    pub fn tool_state(lane_handle: WorthUiLaneHandle) -> Self {
        Self {
            kind: WorthUiCanvasSpatialFrameTargetKind::ToolState(lane_handle),
        }
    }

    pub(crate) fn kind(self) -> WorthUiCanvasSpatialFrameTargetKind {
        self.kind
    }

    pub(crate) fn request_meaning_digest(self) -> u64 {
        match self.kind {
            WorthUiCanvasSpatialFrameTargetKind::Viewport(request) => {
                1_u64
                    ^ u64::from(request.pan_delta_x().cast_unsigned()).rotate_left(7)
                    ^ u64::from(request.pan_delta_y().cast_unsigned()).rotate_left(23)
                    ^ u64::from(request.zoom_milli_factor()).rotate_left(41)
            }
            WorthUiCanvasSpatialFrameTargetKind::Draw(_) => 2,
            WorthUiCanvasSpatialFrameTargetKind::HitTest(request) => {
                3_u64
                    ^ u64::from(request.viewport_point().x().cast_unsigned()).rotate_left(11)
                    ^ u64::from(request.viewport_point().y().cast_unsigned()).rotate_left(37)
            }
            WorthUiCanvasSpatialFrameTargetKind::Overlay(_) => 4,
            WorthUiCanvasSpatialFrameTargetKind::ToolState(_) => 5,
        }
    }
}
