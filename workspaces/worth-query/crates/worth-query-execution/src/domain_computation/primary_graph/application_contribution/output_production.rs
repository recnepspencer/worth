//! Output production's own state: the demand registry, recovered outputs, installed producers,
//! their signal and readiness routes, and the producer attempt ordinal.
//!
//! Field order is drop order: producers, then routes, then demands and recovered outputs, the
//! sequence the runtime's teardown has always kept.
use std::sync::atomic::AtomicU64;

use crate::domain_computation::primary_graph::application_output_demand::{
    WorthQueryOutputDemandRegistry, WorthQueryRecoveredOutputs,
};

use super::{
    WorthQueryInstalledApplicationProducerRegistry, WorthQueryInstalledOutputProducerRoutes,
    WorthQueryInstalledOutputReadinessRoutes,
};

mod court_doors_for_test;

pub struct WorthQueryOutputProduction<Schema> {
    producers: WorthQueryInstalledApplicationProducerRegistry<Schema>,
    producer_routes: WorthQueryInstalledOutputProducerRoutes,
    readiness_routes: WorthQueryInstalledOutputReadinessRoutes,
    demands: WorthQueryOutputDemandRegistry,
    recovered: WorthQueryRecoveredOutputs,
    next_producer_attempt: AtomicU64,
}

impl<Schema> WorthQueryOutputProduction<Schema> {
    pub(in crate::domain_computation::primary_graph) fn new(
        producer_routes: WorthQueryInstalledOutputProducerRoutes,
        readiness_routes: WorthQueryInstalledOutputReadinessRoutes,
    ) -> Self {
        Self {
            producers: Default::default(),
            producer_routes,
            readiness_routes,
            demands: WorthQueryOutputDemandRegistry::default(),
            recovered: Default::default(),
            next_producer_attempt: AtomicU64::new(1),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn install_producers(
        &mut self,
        producers: WorthQueryInstalledApplicationProducerRegistry<Schema>,
    ) {
        self.producers = producers;
    }

    pub(in crate::domain_computation::primary_graph) fn restore_recovered(
        &mut self,
        recovered: WorthQueryRecoveredOutputs,
    ) {
        self.recovered = recovered;
    }

    pub(in crate::domain_computation::primary_graph) fn producers(
        &self,
    ) -> &WorthQueryInstalledApplicationProducerRegistry<Schema> {
        &self.producers
    }

    pub(in crate::domain_computation::primary_graph) fn producer_routes(
        &self,
    ) -> &WorthQueryInstalledOutputProducerRoutes {
        &self.producer_routes
    }

    pub(in crate::domain_computation::primary_graph) fn readiness_routes(
        &self,
    ) -> &WorthQueryInstalledOutputReadinessRoutes {
        &self.readiness_routes
    }

    pub(in crate::domain_computation::primary_graph) fn demands(
        &self,
    ) -> &WorthQueryOutputDemandRegistry {
        &self.demands
    }

    pub(in crate::domain_computation::primary_graph) fn recovered(
        &self,
    ) -> &WorthQueryRecoveredOutputs {
        &self.recovered
    }

    pub(in crate::domain_computation::primary_graph) fn next_producer_attempt(&self) -> &AtomicU64 {
        &self.next_producer_attempt
    }
}
