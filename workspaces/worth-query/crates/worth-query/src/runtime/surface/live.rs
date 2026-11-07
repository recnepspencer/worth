use std::marker::PhantomData;

use super::super::{
    WorthQueryAuthorityLane, WorthQueryDerivedPatch, WorthQueryRuntimeDeliveryBatch,
    WorthQueryRuntimeLiveSubscriptionInstallation,
};
use crate::memory_workspace::{WorthQueryLivePatch, WorthQueryLiveViewHandle};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryPatchBatch {
    pub view_name: String,
    pub live_patches: Vec<WorthQueryLivePatch>,
    pub query_delivery_batches: Vec<WorthQueryRuntimeDeliveryBatch>,
    pub derived_patch_notes: Vec<String>,
    pub derived_patches: Vec<WorthQueryDerivedPatch>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryUnrefinedLiveShape {}

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryLiveView<T = WorthQueryUnrefinedLiveShape> {
    pub(super) handle: WorthQueryLiveViewHandle,
    pub(super) authority_lane: WorthQueryAuthorityLane,
    pub(super) subscription_installation: WorthQueryRuntimeLiveSubscriptionInstallation,
    pub(super) marker: PhantomData<T>,
}

impl<T> Clone for WorthQueryLiveView<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            authority_lane: self.authority_lane,
            subscription_installation: self.subscription_installation.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> WorthQueryLiveView<T> {
    pub(in crate::runtime) fn new(
        handle: WorthQueryLiveViewHandle,
        subscription_installation: WorthQueryRuntimeLiveSubscriptionInstallation,
    ) -> Self {
        Self {
            handle,
            authority_lane: WorthQueryAuthorityLane::AuthoritativeTruth,
            subscription_installation,
            marker: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        self.handle.name()
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn subscription_installation(&self) -> &WorthQueryRuntimeLiveSubscriptionInstallation {
        &self.subscription_installation
    }
}
#[cfg(test)]
#[path = "live_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
