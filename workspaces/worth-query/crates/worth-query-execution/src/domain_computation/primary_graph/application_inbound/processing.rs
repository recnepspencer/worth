//! The inbound processing state: installed verifiers, inbound custody, installed-transport
//! completion custody and the maintenance turn.
use std::collections::BTreeMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use crate::domain_computation::application_aftermath::WorthQueryInboundCustody;

use super::inbound_occurrence::WorthQueryInstalledInboundVerifier;
use super::inbound_publication::InstalledTransportCompletionCustody;

type InstalledVerifiers = BTreeMap<String, Arc<WorthQueryInstalledInboundVerifier>>;

pub(in crate::domain_computation::primary_graph) struct WorthQueryInboundProcessing {
    verifiers: Mutex<InstalledVerifiers>,
    custody: Arc<Mutex<WorthQueryInboundCustody>>,
    completions: Mutex<InstalledTransportCompletionCustody>,
    maintenance_turn: AtomicU64,
}

impl Default for WorthQueryInboundProcessing {
    fn default() -> Self {
        Self {
            verifiers: Mutex::new(BTreeMap::new()),
            custody: Arc::new(Mutex::new(Default::default())),
            completions: Mutex::new(Default::default()),
            maintenance_turn: AtomicU64::new(0),
        }
    }
}

impl WorthQueryInboundProcessing {
    pub(in crate::domain_computation::primary_graph) fn verifiers(
        &self,
    ) -> &Mutex<InstalledVerifiers> {
        &self.verifiers
    }

    pub(in crate::domain_computation::primary_graph) fn custody(
        &self,
    ) -> &Arc<Mutex<WorthQueryInboundCustody>> {
        &self.custody
    }

    pub(in crate::domain_computation::primary_graph) fn completions(
        &self,
    ) -> &Mutex<InstalledTransportCompletionCustody> {
        &self.completions
    }

    pub(in crate::domain_computation::primary_graph) fn maintenance_turn(&self) -> &AtomicU64 {
        &self.maintenance_turn
    }

    /// Moves the verifiers out for ordered teardown.
    pub(in crate::domain_computation::primary_graph) fn take_verifiers(
        &mut self,
    ) -> InstalledVerifiers {
        std::mem::take(
            self.verifiers
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }

    /// Moves the completion custody out for ordered teardown.
    pub(in crate::domain_computation::primary_graph) fn take_completions(
        &mut self,
    ) -> InstalledTransportCompletionCustody {
        std::mem::take(
            self.completions
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }
}
