//! The dashboard's authored content, allocation and intra-surface paint hierarchy.
mod activity;
mod deployments;
mod graphics;
mod metrics;
mod navigation;
mod period;
mod review;
mod scrolling;
mod services;
pub use scrolling::DashboardScrollPanel;
mod signals;
mod traffic;

#[cfg(test)]
mod declaration_coverage_tests;

pub use graphics::DashboardGraphic;

use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentId, ComponentViewportRegion, MosaicLayoutCell,
    MosaicResponsiveLayout,
};

#[derive(Clone, Copy, Debug)]
pub enum DashboardContent {
    Text {
        value: &'static str,
        size: u16,
        serif: bool,
        weight: u16,
        alignment: worth_ui::facade::app::UiTextAlignment,
        line_height: u16,
    },
    Surface {
        radius: u16,
        border: bool,
        graphic: DashboardGraphic,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct DashboardElement {
    pub id: &'static str,
    pub rect: [u16; 4],
    pub color: &'static str,
    pub content: DashboardContent,
    pub paint_order: u32,
    pub portal_owner: Option<&'static str>,
    pub interaction: Option<&'static str>,
    pub scroll_panel: Option<DashboardScrollPanel>,
    pub layout_cell: Option<DashboardLayoutCell>,
}

/// Where a layout container places one element: a cell of the container's
/// tracks, and the element's region within that cell.
#[derive(Clone, Copy, Debug)]
pub struct DashboardLayoutCell {
    pub container: &'static str,
    pub cell: MosaicLayoutCell,
    pub region: ComponentViewportRegion,
}

/// A component that paints nothing and lays its member elements out in
/// flexible tracks.
#[derive(Clone, Debug)]
pub struct DashboardContainer {
    pub id: &'static str,
    pub allocation: ComponentAllocationMeasurementContract,
    pub layout: MosaicResponsiveLayout,
}

impl DashboardContainer {
    pub fn component(&self) -> ComponentId {
        ComponentId::new(format!("platform.pulse.component.{}", self.id))
            .expect("container identities are valid component identities")
    }
}

pub fn dashboard_elements() -> Vec<DashboardElement> {
    let mut elements = Vec::new();
    elements.extend(navigation::elements());
    elements.extend(metrics::elements());
    elements.extend(traffic::elements());
    elements.extend(services::elements());
    elements.extend(activity::elements());
    elements.extend(deployments::elements());
    elements.extend(signals::elements());
    elements.extend(review::elements());
    elements.extend(period::elements());
    elements
}

pub fn dashboard_containers() -> Vec<DashboardContainer> {
    vec![metrics::container(&dashboard_elements())]
}

const fn text(
    id: &'static str,
    value: &'static str,
    rect: [u16; 4],
    size: u16,
    serif: bool,
    color: &'static str,
) -> DashboardElement {
    DashboardElement {
        id,
        rect,
        color,
        content: DashboardContent::Text {
            value,
            size,
            serif,
            weight: 500,
            alignment: worth_ui::facade::app::UiTextAlignment::Start,
            line_height: rect[3],
        },
        paint_order: 6,
        portal_owner: None,
        interaction: None,
        scroll_panel: None,
        layout_cell: None,
    }
}

const fn surface(
    id: &'static str,
    rect: [u16; 4],
    color: &'static str,
    radius: u16,
    border: bool,
    paint_order: u32,
) -> DashboardElement {
    DashboardElement {
        id,
        rect,
        color,
        content: DashboardContent::Surface {
            radius,
            border,
            graphic: DashboardGraphic::Rectangle,
        },
        paint_order,
        portal_owner: None,
        interaction: None,
        scroll_panel: None,
        layout_cell: None,
    }
}

impl DashboardElement {
    pub fn component_id(self) -> String {
        format!("platform.pulse.component.{}", self.id)
    }
    pub fn component(self) -> ComponentId {
        ComponentId::new(self.component_id())
            .expect("element identities are valid component identities")
    }
    /// The allocation this element declares: its cell region when a layout
    /// container places it, otherwise its authored rectangle.
    pub fn allocation(self) -> ComponentAllocationMeasurementContract {
        match self.layout_cell {
            Some(cell) => ComponentAllocationMeasurementContract::layout_cell(cell.region),
            None => {
                let [x, y, width, height] = self.rect;
                super::PlatformPulseLogicalRect::new(
                    x.into(),
                    y.into(),
                    width.into(),
                    height.into(),
                )
                .allocation()
            }
        }
    }
    pub fn role_id(self) -> String {
        format!("platform.pulse.appearance.{}", self.id)
    }
    pub fn color_token(self) -> String {
        format!("theme.platform_pulse.{}", self.color)
    }

    const fn centered(mut self) -> Self {
        if let DashboardContent::Text {
            ref mut alignment, ..
        } = self.content
        {
            *alignment = worth_ui::facade::app::UiTextAlignment::Center;
        }
        self
    }

    const fn wrapped(mut self) -> Self {
        if let DashboardContent::Text {
            size,
            ref mut line_height,
            ..
        } = self.content
        {
            *line_height = size + 8;
        }
        self
    }

    const fn graphic(mut self, graphic: DashboardGraphic) -> Self {
        self.content = DashboardContent::Surface {
            radius: 0,
            border: false,
            graphic,
        };
        self
    }
    const fn portal(mut self, owner: &'static str) -> Self {
        self.portal_owner = Some(owner);
        self
    }
    const fn action(mut self, route: &'static str) -> Self {
        self.interaction = Some(route);
        self
    }
    const fn in_cell(
        mut self,
        container: &'static str,
        cell: MosaicLayoutCell,
        region: ComponentViewportRegion,
    ) -> Self {
        self.layout_cell = Some(DashboardLayoutCell {
            container,
            cell,
            region,
        });
        self
    }
}
