use std::collections::BTreeMap;

const NATIVE_RESOURCE_LIMIT: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiMountedResourceCacheDenial {
    CapacityExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHeadlessMountedResourceHandle(u64);

#[derive(Debug, Default)]
pub struct WorthUiHeadlessMountedResourceCache {
    bound: Option<BoundResources>,
    next_handle: u64,
}

/// Resource handles are only meaningful for the binding they were issued under.
#[derive(Debug)]
struct BoundResources {
    binding: crate::UiSurfaceBindingGeneration,
    by_content: BTreeMap<u64, UiHeadlessMountedResourceHandle>,
}

impl WorthUiHeadlessMountedResourceCache {
    pub fn reconcile(
        &mut self,
        view: &super::UiMountedProjectionView,
    ) -> Result<(), WorthUiMountedResourceCacheDenial> {
        let binding = view.binding();
        if self
            .bound
            .as_ref()
            .is_some_and(|bound| bound.binding != binding)
        {
            self.bound = None;
        }
        let bound = self.bound.get_or_insert_with(|| BoundResources {
            binding,
            by_content: BTreeMap::new(),
        });
        for resource in view.resources().entries() {
            if bound.by_content.contains_key(&resource.content_identity()) {
                continue;
            }
            if bound.by_content.len() >= NATIVE_RESOURCE_LIMIT {
                return Err(WorthUiMountedResourceCacheDenial::CapacityExceeded);
            }
            self.next_handle = self
                .next_handle
                .checked_add(1)
                .ok_or(WorthUiMountedResourceCacheDenial::CapacityExceeded)?;
            bound.by_content.insert(
                resource.content_identity(),
                UiHeadlessMountedResourceHandle(self.next_handle),
            );
        }
        Ok(())
    }

    pub fn handle_for(&self, content_identity: u64) -> Option<UiHeadlessMountedResourceHandle> {
        self.bound
            .as_ref()
            .and_then(|bound| bound.by_content.get(&content_identity).copied())
    }

    pub fn binding(&self) -> Option<crate::UiSurfaceBindingGeneration> {
        self.bound.as_ref().map(|bound| bound.binding)
    }
}
