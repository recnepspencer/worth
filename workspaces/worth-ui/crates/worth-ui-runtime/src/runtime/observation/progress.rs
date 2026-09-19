#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum UiObservationProgressKey {
    AuthoredSource(Box<str>),
    HostViewport(u64),
    HostDeviceScale(u64),
    PointerPresence(worth_ui_host_contract::UiHostPointerIdentity),
    Measurement(u64),
    Query(worth_ui_query_binding::WorthUiCollectionChangeSourceReference),
    QueryProjection(worth_ui_query_binding::WorthUiQueryViewIdentity),
    IntentPosture,
    CommittedScrollExtent,
    CommittedPortalAnchor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct UiObservationProgress {
    key: UiObservationProgressKey,
    owner_order: u64,
}

impl UiObservationProgress {
    pub(super) fn authored_source(provider: &str, owner_order: u64) -> Self {
        Self {
            key: UiObservationProgressKey::AuthoredSource(provider.into()),
            owner_order,
        }
    }

    pub(super) const fn host_viewport(host_session: u64, owner_order: u64) -> Self {
        Self {
            key: UiObservationProgressKey::HostViewport(host_session),
            owner_order,
        }
    }

    pub(super) const fn host_device_scale(host_session: u64, owner_order: u64) -> Self {
        Self {
            key: UiObservationProgressKey::HostDeviceScale(host_session),
            owner_order,
        }
    }

    pub(super) const fn pointer_presence(
        pointer: worth_ui_host_contract::UiHostPointerIdentity,
        owner_order: u64,
    ) -> Self {
        Self {
            key: UiObservationProgressKey::PointerPresence(pointer),
            owner_order,
        }
    }

    pub(super) const fn measurement(source_identity: u64, owner_order: u64) -> Self {
        Self {
            key: UiObservationProgressKey::Measurement(source_identity),
            owner_order,
        }
    }

    pub(super) fn query(
        source: &worth_ui_query_binding::WorthUiCollectionChangeSourceReference,
        owner_order: u64,
    ) -> Self {
        Self {
            key: UiObservationProgressKey::Query(source.clone()),
            owner_order,
        }
    }

    pub(super) fn query_projection(
        projection: &worth_ui_query_binding::WorthUiQueryViewIdentity,
        owner_order: u64,
    ) -> Self {
        Self {
            key: UiObservationProgressKey::QueryProjection(projection.clone()),
            owner_order,
        }
    }

    pub(super) const fn intent_posture(owner_order: u64) -> Self {
        Self {
            key: UiObservationProgressKey::IntentPosture,
            owner_order,
        }
    }

    pub(super) const fn committed_scroll_extent(owner_order: u64) -> Self {
        Self {
            key: UiObservationProgressKey::CommittedScrollExtent,
            owner_order,
        }
    }

    pub(super) const fn committed_portal_anchor(owner_order: u64) -> Self {
        Self {
            key: UiObservationProgressKey::CommittedPortalAnchor,
            owner_order,
        }
    }

    pub(super) fn key(&self) -> &UiObservationProgressKey {
        &self.key
    }

    pub(super) const fn owner_order(&self) -> u64 {
        self.owner_order
    }
}
