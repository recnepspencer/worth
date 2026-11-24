use crate::publication::bundle::PublicationBundle;
use crate::replay::data::RelationalReplayRecord;
use crate::storage::substrate::owned_state::RuntimeOwnedState;
use std::collections::VecDeque;
use std::sync::Arc;

const RETIRED_BUNDLE_BACKLOG_LIMIT: usize = 8;

type SharedBundle = Arc<PublicationBundle<RelationalReplayRecord>>;

/// The published bundle and its bounded retirement backlog.
///
/// Bundles are held by shared ownership so an observer can carry the current
/// bundle out of the publication lock without copying it.
#[derive(Debug, Clone, Default)]
struct PublishedBundles {
    latest: Option<SharedBundle>,
    retired: VecDeque<SharedBundle>,
}

#[derive(Debug)]
pub(crate) struct PublicationSubsystem {
    pub(crate) diagnostics: crate::publication::diagnostic_store::RelationalDiagnosticArtifactStore,
    bundles: RuntimeOwnedState<PublishedBundles>,
    pub(crate) post_commit_consumer: Arc<dyn crate::publication::authority::PostCommitConsumer>,
}

impl Default for PublicationSubsystem {
    fn default() -> Self {
        Self {
            diagnostics: Default::default(),
            bundles: RuntimeOwnedState::default(),
            post_commit_consumer: crate::publication::authority::production_post_commit_consumer(),
        }
    }
}

impl PublicationSubsystem {
    pub(crate) fn latest_bundle(&self) -> Option<SharedBundle> {
        self.bundles.read().latest.clone()
    }

    pub(crate) fn replace_latest_bundle(&self, bundle: PublicationBundle<RelationalReplayRecord>) {
        let mut bundles = self.bundles.write();
        if let Some(previous) = bundles.latest.replace(Arc::new(bundle)) {
            bundles.retired.push_back(previous);
            if bundles.retired.len() > RETIRED_BUNDLE_BACKLOG_LIMIT {
                let _ = bundles.retired.pop_front();
            }
        }
    }

    pub(crate) fn fork(&self) -> Self {
        Self {
            diagnostics: self.diagnostics.detached_owner_snapshot(),
            bundles: RuntimeOwnedState::new(PublishedBundles {
                latest: self.latest_bundle(),
                retired: VecDeque::new(),
            }),
            post_commit_consumer: Arc::clone(&self.post_commit_consumer),
        }
    }
}
