use crate::capability::{ComponentId, MosaicLayoutDenial, MosaicResponsiveLayout};

use super::FrozenComponentCapabilities;

/// Why an authored `layout` block could not restate its container's layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAuthoredLayoutCause {
    /// The block names something that is not a well formed component identity.
    ContainerIdentityMalformed,
    /// A `member` names something that is not a well formed component identity.
    MemberIdentityMalformed,
    /// The tracks, cells, or intervals do not make a Mosaic layout.
    Layout(MosaicLayoutDenial),
    /// A second block states a layout for the same container.
    DuplicateDeclaration,
    /// The named container is not a registered component.
    UnregisteredContainer,
    /// The named component registered no layout for its members.
    ContainerHasNoLayout,
    /// The source and the registration state different layouts for the same
    /// container. Two sources naming one layout disagree, and neither wins.
    LayoutDisagreement,
}

/// A refused authored `layout` block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAuthoredLayoutDenial {
    declaration_index: usize,
    cause: UiAuthoredLayoutCause,
}

impl UiAuthoredLayoutDenial {
    pub(crate) const fn new(declaration_index: usize, cause: UiAuthoredLayoutCause) -> Self {
        Self {
            declaration_index,
            cause,
        }
    }

    /// Which of the package's `layout` blocks was refused, in package order.
    pub const fn declaration_index(&self) -> usize {
        self.declaration_index
    }

    pub const fn cause(&self) -> UiAuthoredLayoutCause {
        self.cause
    }
}

/// One authored `layout` block, lowered into Mosaic meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAuthoredComponentLayout {
    declaration_index: usize,
    container: ComponentId,
    layout: MosaicResponsiveLayout,
}

impl UiAuthoredComponentLayout {
    pub(crate) const fn new(
        declaration_index: usize,
        container: ComponentId,
        layout: MosaicResponsiveLayout,
    ) -> Self {
        Self {
            declaration_index,
            container,
            layout,
        }
    }

    const fn denied(&self, cause: UiAuthoredLayoutCause) -> UiAuthoredLayoutDenial {
        UiAuthoredLayoutDenial::new(self.declaration_index, cause)
    }
}

impl FrozenComponentCapabilities {
    /// Admits authored layouts that restate what their containers registered.
    ///
    /// A container's layout is registered with it, because which components
    /// are layout cells, and whether their containment is acyclic, is judged
    /// at registration. An authored block therefore restates that layout, and
    /// is refused where it states another.
    pub(crate) fn admit_authored_layouts(
        &self,
        authored: &[UiAuthoredComponentLayout],
    ) -> Result<(), UiAuthoredLayoutDenial> {
        let mut containers = std::collections::BTreeSet::new();
        for layout in authored {
            if !containers.insert(&layout.container) {
                return Err(layout.denied(UiAuthoredLayoutCause::DuplicateDeclaration));
            }
            let registered = self
                .get(&layout.container)
                .ok_or_else(|| layout.denied(UiAuthoredLayoutCause::UnregisteredContainer))?
                .layout()
                .ok_or_else(|| layout.denied(UiAuthoredLayoutCause::ContainerHasNoLayout))?;
            if *registered != layout.layout {
                return Err(layout.denied(UiAuthoredLayoutCause::LayoutDisagreement));
            }
        }
        Ok(())
    }
}
