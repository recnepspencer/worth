#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostServiceGeometryDenial {
    EmptyPhysicalExtent,
    PhysicalEdgeOrder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHostPhysicalPixelGeometry {
    surface: crate::UiHostSurfaceIdentity,
    binding: crate::UiSurfaceBindingGeneration,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHostPhysicalPixelGeometryInput {
    pub surface: crate::UiHostSurfaceIdentity,
    pub binding: crate::UiSurfaceBindingGeneration,
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl UiHostPhysicalPixelGeometry {
    pub fn observed_by_host(
        input: UiHostPhysicalPixelGeometryInput,
    ) -> Result<Self, UiHostServiceGeometryDenial> {
        if input.left == input.right || input.top == input.bottom {
            return Err(UiHostServiceGeometryDenial::EmptyPhysicalExtent);
        }
        if input.left > input.right || input.top > input.bottom {
            return Err(UiHostServiceGeometryDenial::PhysicalEdgeOrder);
        }
        Ok(Self {
            surface: input.surface,
            binding: input.binding,
            left: input.left,
            top: input.top,
            right: input.right,
            bottom: input.bottom,
        })
    }

    pub const fn edges(self) -> [u32; 4] {
        [self.left, self.top, self.right, self.bottom]
    }

    pub const fn surface(self) -> crate::UiHostSurfaceIdentity {
        self.surface
    }

    pub const fn binding(self) -> crate::UiSurfaceBindingGeneration {
        self.binding
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UiHostPhysicalPixelGeometry, UiHostPhysicalPixelGeometryInput, UiHostServiceGeometryDenial,
    };

    #[test]
    fn host_physical_geometry_keeps_its_observed_edges() {
        let surface = crate::UiHostSurfaceIdentity::mint_unbound().unwrap();
        let binding = crate::UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let physical =
            UiHostPhysicalPixelGeometry::observed_by_host(UiHostPhysicalPixelGeometryInput {
                surface,
                binding,
                left: 5,
                top: 6,
                right: 25,
                bottom: 14,
            })
            .expect("ordered physical geometry");

        assert_eq!(physical.edges(), [5, 6, 25, 14]);
        assert_eq!(physical.surface(), surface);
        assert_eq!(physical.binding(), binding);
    }

    #[test]
    fn host_geometry_rejects_invalid_values_before_transport() {
        let surface = crate::UiHostSurfaceIdentity::mint_unbound().unwrap();
        let binding = crate::UiSurfaceBindingGeneration::mint_unbound().unwrap();
        assert_eq!(
            UiHostPhysicalPixelGeometry::observed_by_host(UiHostPhysicalPixelGeometryInput {
                surface,
                binding,
                left: 8,
                top: 2,
                right: 4,
                bottom: 6,
            }),
            Err(UiHostServiceGeometryDenial::PhysicalEdgeOrder)
        );
    }
}
