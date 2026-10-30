use std::sync::Arc;

use super::{WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantExecutionFailure};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryInvariantStateLocator {
    family: Arc<str>,
    identity: Arc<str>,
}

impl WorthQueryInvariantStateLocator {
    pub fn new(
        family: impl Into<Arc<str>>,
        identity: impl Into<Arc<str>>,
    ) -> Result<Self, WorthQueryInvariantExecutionFailure> {
        let family = canonical(family)?;
        let identity = canonical(identity)?;
        Ok(Self { family, identity })
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
}

pub struct WorthQueryAdmittedInvariantStateLoadPlan {
    identity: Arc<str>,
    locators: Arc<[WorthQueryInvariantStateLocator]>,
}

impl WorthQueryAdmittedInvariantStateLoadPlan {
    pub(crate) fn admit(
        identity: impl Into<Arc<str>>,
        locators: impl IntoIterator<Item = WorthQueryInvariantStateLocator>,
        allowed_families: &[String],
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
    ) -> Result<Self, WorthQueryInvariantExecutionFailure> {
        let mut locators = locators.into_iter().collect::<Vec<_>>();
        let mut stopped = None;
        locators.sort_by(|left, right| {
            if stopped.is_none() {
                stopped = super::request_control::check_live(request).err();
            }
            left.cmp(right)
        });
        if let Some(stop) = stopped {
            return Err(stop);
        }
        super::request_control::check_live(request)?;
        locators.dedup();
        // Empty plans are lawful for emit-only / outbox-only commits (R8.55):
        // there are no proposed graph mutation facts to load, and the provider
        // still validates the registered scaffolding batch under the same
        // invariant owner path.
        for locator in &locators {
            super::request_control::check_live(request)?;
            let mut admitted = false;
            for family in allowed_families {
                super::request_control::check_live(request)?;
                if family == locator.family() {
                    admitted = true;
                    break;
                }
            }
            if !admitted {
                return Err(failure(
                    WorthQueryInvariantExecutionDenialKind::UndeclaredStateLoadFamily,
                ));
            }
        }
        Ok(Self {
            identity: identity.into(),
            locators: locators.into(),
        })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn locators(&self) -> &[WorthQueryInvariantStateLocator] {
        &self.locators
    }
}

fn canonical(value: impl Into<Arc<str>>) -> Result<Arc<str>, WorthQueryInvariantExecutionFailure> {
    let value = value.into();
    if value.trim().is_empty() || value.trim() != value.as_ref() {
        Err(failure(
            WorthQueryInvariantExecutionDenialKind::ProviderRejected,
        ))
    } else {
        Ok(value)
    }
}

fn failure(kind: WorthQueryInvariantExecutionDenialKind) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(kind, "invariant state-load admission denied")
}
