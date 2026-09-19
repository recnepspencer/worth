use std::sync::Mutex;

use worth_query_decl::facade::application_program::{
    ApplicationExternalInputProvider, ApplicationExternalInputResolution,
};
use worth_query_topology_entry::EditPlanar;

use crate::ConsumerSchema;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NeutralExternalDenial {
    Removed,
    Changed,
    Invalid,
}

pub(crate) struct NeutralExternalProvider {
    state: Mutex<Option<(u64, bool)>>,
}

impl NeutralExternalProvider {
    pub(crate) const fn new(revision: u64) -> Self {
        Self {
            state: Mutex::new(Some((revision, true))),
        }
    }

    pub(crate) fn change(&self, revision: u64) {
        *self.state.lock().expect("neutral provider poisoned") = Some((revision, true));
    }

    pub(crate) fn remove(&self) {
        *self.state.lock().expect("neutral provider poisoned") = None;
    }

    pub(crate) fn invalidate(&self) {
        if let Some((_, valid)) = self
            .state
            .lock()
            .expect("neutral provider poisoned")
            .as_mut()
        {
            *valid = false;
        }
    }
}

impl ApplicationExternalInputProvider<ConsumerSchema, EditPlanar> for NeutralExternalProvider {
    const IDENTITY: &'static str = "worth.query.certification.neutral-external-input.v1";
    type Selection = &'static str;
    type Values = u64;
    type Revision = u64;
    type Provenance = &'static str;
    type Denial = NeutralExternalDenial;

    fn resolve(
        &self,
        _: &&'static str,
    ) -> Result<ApplicationExternalInputResolution<u64, u64, &'static str>, Self::Denial> {
        match *self.state.lock().expect("neutral provider poisoned") {
            None => Err(NeutralExternalDenial::Removed),
            Some((_, false)) => Err(NeutralExternalDenial::Invalid),
            Some((revision, true)) => Ok(ApplicationExternalInputResolution::new(
                revision,
                revision,
                "neutral-catalog",
            )),
        }
    }

    fn validate_revision(&self, _: &&'static str, revision: &u64) -> Result<(), Self::Denial> {
        match *self.state.lock().expect("neutral provider poisoned") {
            None => Err(NeutralExternalDenial::Removed),
            Some((_, false)) => Err(NeutralExternalDenial::Invalid),
            Some((current, true)) if current != *revision => Err(NeutralExternalDenial::Changed),
            Some(_) => Ok(()),
        }
    }
}
