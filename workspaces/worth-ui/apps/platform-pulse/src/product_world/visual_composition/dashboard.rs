//! The dashboard's authored content, allocation and intra-surface paint hierarchy.
mod activity;
mod deployments;
mod frame;
mod graphics;
mod metrics;
mod navigation;
mod page;
mod period;
mod review;
mod scroll_owner;
mod scrolling;
mod services;
pub use scroll_owner::DashboardScrollOwner;
pub use scrolling::DashboardScrollPanel;
mod signals;
mod traffic;

#[cfg(test)]
mod declaration_coverage_tests;
#[cfg(test)]
mod declared_layout;
#[cfg(test)]
mod page_tests;

pub use graphics::DashboardGraphic;
pub use page::{PLATFORM_PULSE_MASTHEAD_HEIGHT, PLATFORM_PULSE_SIDEBAR_WIDTH};

use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentId, ComponentViewportRegion, MosaicLayoutCell,
    MosaicLayoutContract, MosaicResponsiveLayout, MosaicViewportWidthInterval,
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
    pub placement: DashboardPlacement,
}

/// Where a dashboard component stands.
#[derive(Clone, Copy, Debug)]
pub enum DashboardPlacement {
    /// At its authored rectangle: from the viewport origin, or, for a Portal
    /// child, from its owner's.
    Authored,
    /// A region of the viewport.
    Viewport(ComponentViewportRegion),
    /// A region of a layout container's cell.
    Cell(DashboardLayoutCell),
}

/// Where a layout container places one component: a cell of the container's
/// tracks, and the component's region within that cell.
#[derive(Clone, Copy, Debug)]
pub struct DashboardLayoutCell {
    pub container: &'static str,
    pub cell: MosaicLayoutCell,
    pub region: ComponentViewportRegion,
}

/// A component that paints nothing and lays its members out in flexible
/// tracks. A scroll owner is one: its members travel with it.
#[derive(Clone, Debug)]
pub struct DashboardContainer {
    pub id: &'static str,
    pub placement: DashboardPlacement,
    pub layout: MosaicResponsiveLayout,
    pub scroll_owner: Option<DashboardScrollOwner>,
}

impl DashboardContainer {
    pub fn component(&self) -> ComponentId {
        container_component(self.id)
    }
    pub fn allocation(&self) -> ComponentAllocationMeasurementContract {
        self.placement.allocation(None)
    }
}

fn container_component(id: &str) -> ComponentId {
    ComponentId::new(format!("platform.pulse.component.{id}"))
        .expect("container identities are valid component identities")
}

/// A container's tracks before the members that name it are gathered.
struct ContainerTracks {
    id: &'static str,
    placement: DashboardPlacement,
    /// The tracks at every width, or, for a container with a stacked
    /// layout, at widths from the dashboard's breakpoint up.
    tracks: MosaicLayoutContract,
    stacked: Option<StackedTracks>,
    scroll_owner: Option<DashboardScrollOwner>,
}

/// The tracks a container lays its members out in below the dashboard's
/// breakpoint, and the cell each member moves to there from the cell its
/// placement names.
struct StackedTracks {
    tracks: MosaicLayoutContract,
    cell: fn(MosaicLayoutCell) -> MosaicLayoutCell,
}

impl DashboardPlacement {
    /// The allocation this placement declares. Only an element has an
    /// authored rectangle to stand at.
    fn allocation(self, rect: Option<[u16; 4]>) -> ComponentAllocationMeasurementContract {
        match self {
            Self::Authored => {
                let [x, y, width, height] = rect.expect("only an element stands where authored");
                super::PlatformPulseLogicalRect::new(
                    x.into(),
                    y.into(),
                    width.into(),
                    height.into(),
                )
                .allocation()
            }
            Self::Viewport(region) => {
                ComponentAllocationMeasurementContract::viewport_region(region)
            }
            Self::Cell(cell) => ComponentAllocationMeasurementContract::layout_cell(cell.region),
        }
    }

    pub fn layout_cell(self) -> Option<DashboardLayoutCell> {
        match self {
            Self::Cell(cell) => Some(cell),
            Self::Authored | Self::Viewport(_) => None,
        }
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

/// Every layout container, each declaring as members exactly the elements
/// and containers placed in its cells.
pub fn dashboard_containers() -> Vec<DashboardContainer> {
    let mut containers = page::containers();
    containers.push(metrics::container());
    containers.extend(traffic::containers());
    containers.extend(DashboardScrollPanel::ALL.map(DashboardScrollPanel::container));
    let members = dashboard_elements()
        .into_iter()
        .map(|element| (element.component(), element.placement))
        .chain(
            containers
                .iter()
                .map(|container| (container_component(container.id), container.placement)),
        )
        .filter_map(|(component, placement)| Some((component, placement.layout_cell()?)))
        .collect::<Vec<_>>();
    containers
        .into_iter()
        .map(|container| {
            let with_members =
                |tracks: MosaicLayoutContract, cell: fn(MosaicLayoutCell) -> MosaicLayoutCell| {
                    members
                        .iter()
                        .filter(|(_, placed)| placed.container == container.id)
                        .fold(tracks, |layout, (component, placed)| {
                            layout
                                .with_member(component.clone(), cell(placed.cell))
                                .expect("each member sits once within its container's tracks")
                        })
                };
            let wide = with_members(container.tracks, |cell| cell);
            let layout = match container.stacked {
                None => wide.into(),
                Some(stacked) => {
                    MosaicResponsiveLayout::new(with_members(stacked.tracks, stacked.cell))
                        .with_variant(
                            MosaicViewportWidthInterval::at_least(page::BREAKPOINT),
                            wide,
                        )
                        .expect("both layouts hold the same members")
                }
            };
            DashboardContainer {
                id: container.id,
                placement: container.placement,
                layout,
                scroll_owner: container.scroll_owner,
            }
        })
        .collect()
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
        placement: DashboardPlacement::Authored,
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
        placement: DashboardPlacement::Authored,
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
    /// The allocation this element's placement declares.
    pub fn allocation(self) -> ComponentAllocationMeasurementContract {
        self.placement.allocation(Some(self.rect))
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
}
