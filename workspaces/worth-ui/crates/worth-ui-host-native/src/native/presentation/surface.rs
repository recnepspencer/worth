use crate::native::graphics::{
    UiNativeBackendRetainedTarget, UiNativeBackendSurfaceHandle, UiNativeBackendSurfaceMechanics,
};

pub(crate) struct UiNativePresentationSurface {
    pub(crate) mechanics: UiNativeBackendSurfaceMechanics,
    pub(crate) scale_factor: f64,
    pub(crate) extent: [u32; 2],
    pub(crate) generation: u64,
    pub(crate) suspended: bool,
}

pub(crate) struct UiNativeOwnedPresentationSurface {
    state: Box<UiNativePresentationSurface>,
    owners: UiNativePresentationSurfaceOwners,
    basis_generation: u64,
    /// Advances whenever the surface drops or replaces its retained target,
    /// so retained paint can tell which target its pixels are complete in.
    target_generation: u64,
    occluded: bool,
    surface_suspensions: u64,
    targetless_surface_suspensions: u64,
}

pub(crate) struct UiNativePresentationSurfaceOwners {
    pub(crate) surface: crate::native::UiNativeResourceOwner,
    pub(crate) retained_target: Option<crate::native::UiNativeResourceOwner>,
}

impl UiNativeOwnedPresentationSurface {
    pub(crate) fn new(
        state: UiNativePresentationSurface,
        owners: UiNativePresentationSurfaceOwners,
    ) -> Self {
        Self {
            state: Box::new(state),
            owners,
            basis_generation: 1,
            target_generation: 1,
            occluded: false,
            surface_suspensions: 0,
            targetless_surface_suspensions: 0,
        }
    }

    pub(crate) const fn state(&self) -> &UiNativePresentationSurface {
        &self.state
    }

    pub(crate) const fn basis_generation(&self) -> u64 {
        self.basis_generation
    }

    pub(crate) const fn target_generation(&self) -> u64 {
        self.target_generation
    }

    pub(crate) fn observe_occlusion(&mut self, occluded: bool) -> Result<bool, ()> {
        if self.occluded == occluded {
            return Ok(false);
        }
        if !occluded {
            self.basis_generation = self.basis_generation.checked_add(1).ok_or(())?;
        }
        self.occluded = occluded;
        Ok(true)
    }

    pub(crate) fn replace_surface(
        &mut self,
        successor: UiNativeBackendSurfaceHandle,
        successor_owner: crate::native::UiNativeResourceOwner,
        registry: &mut crate::native::UiNativeResourceRegistry,
    ) -> Result<(), ()> {
        let successor_generation = self.state.generation.checked_add(1).ok_or(())?;
        let predecessor = self.state.replace_surface(successor);
        self.state.generation = successor_generation;
        let predecessor_owner = std::mem::replace(&mut self.owners.surface, successor_owner);
        drop(predecessor);
        registry.release(predecessor_owner)
    }

    pub(crate) fn replace_target(
        &mut self,
        successor: UiNativeBackendRetainedTarget,
        successor_owner: crate::native::UiNativeResourceOwner,
        registry: &mut crate::native::UiNativeResourceRegistry,
    ) -> Result<(), ()> {
        self.target_generation = self.target_generation.checked_add(1).ok_or(())?;
        let predecessor = self.state.replace_target(successor);
        let predecessor_owner = self.owners.retained_target.replace(successor_owner);
        drop(predecessor);
        if let Some(predecessor_owner) = predecessor_owner {
            registry.release(predecessor_owner)?;
        }
        Ok(())
    }

    pub(crate) fn suspend(
        &mut self,
        scale_factor: f64,
        extent: [u32; 2],
        registry: &mut crate::native::UiNativeResourceRegistry,
    ) -> Result<(), ()> {
        let surface_suspensions = self.surface_suspensions.checked_add(1).ok_or(())?;
        let targetless_surface_suspensions = self
            .targetless_surface_suspensions
            .checked_add(1)
            .ok_or(())?;
        self.state.scale_factor = scale_factor;
        self.state.extent = extent;
        self.state.suspended = true;
        self.target_generation = self.target_generation.checked_add(1).ok_or(())?;
        let target = self.state.take_target();
        drop(target);
        if let Some(owner) = self.owners.retained_target.take() {
            registry.release(owner)?;
        }
        self.surface_suspensions = surface_suspensions;
        self.targetless_surface_suspensions = targetless_surface_suspensions;
        Ok(())
    }

    pub(crate) const fn surface_suspensions(&self) -> u64 {
        self.surface_suspensions
    }

    pub(crate) const fn targetless_surface_suspensions(&self) -> u64 {
        self.targetless_surface_suspensions
    }

    pub(crate) fn replace_basis(
        &mut self,
        successor: UiNativeBackendRetainedTarget,
        successor_owner: crate::native::UiNativeResourceOwner,
        scale_factor: f64,
        extent: [u32; 2],
        device: &wgpu::Device,
        registry: &mut crate::native::UiNativeResourceRegistry,
    ) -> Result<(), ()> {
        let successor_basis_generation = self.basis_generation.checked_add(1).ok_or(())?;
        self.state.commit_basis(scale_factor, extent, device);
        self.replace_target(successor, successor_owner, registry)?;
        self.basis_generation = successor_basis_generation;
        self.occluded = false;
        Ok(())
    }

    pub(crate) fn close(mut self, registry: &mut crate::native::UiNativeResourceRegistry) {
        let target = self.state.take_target();
        drop(target);
        drop(self.state);
        let mut owners = vec![self.owners.surface];
        owners.extend(self.owners.retained_target);
        registry
            .release_all(owners)
            .expect("presentation-surface owners remain exact");
    }
}
