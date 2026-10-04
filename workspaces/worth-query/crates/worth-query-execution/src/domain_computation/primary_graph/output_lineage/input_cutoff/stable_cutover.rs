//! Total stable lineage installation after exact source and Product checks.

use std::sync::{Arc, OnceLock};

use worth_runtime_world::facade::{
    CurrentProductHead, ProductBranchCurrentnessFailure, ProductBranchObservation,
    RuntimeWorldObservationPort,
};

use super::super::{
    invalidation::{
        CurrentSettlementRegistrationCleanup, InvalidationEditAdmission,
        PreparedCurrentSettlementRegistration,
    },
    partition_index::PreparedStablePartitionLocator,
    prepared_slot::{denial, tree_work},
    RecordedGeneration, RecordedOutput,
};
use super::{
    generation_append::PreparedGenerationAppend,
    stable_publication::PreparedStableLineagePublication,
};
use crate::domain_computation::{
    authorization::{
        WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    },
    primary_graph::{
        WorthQueryAdmittedApplicationOperation, WorthQueryOutputDemandDenial,
        WorthQueryOutputDemandDenialKind as Kind,
    },
};

pub(in crate::domain_computation::primary_graph) enum StablePublicationStop {
    Demand(WorthQueryOutputDemandDenial),
    RequestAuthority(WorthQueryOperationAuthorizationDenial),
}

impl From<WorthQueryOutputDemandDenial> for StablePublicationStop {
    fn from(stop: WorthQueryOutputDemandDenial) -> Self {
        Self::Demand(stop)
    }
}

/// Minted after the exact current source image and its immutable lineage row
/// have been installed together inside the World owner's currentness scope,
/// or for the row a retained observation selects, which its reader verifies
/// at that observation before using it.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct PublishedStableLineage {
    cell: Arc<OnceLock<RecordedOutput>>,
    observation: ProductBranchObservation,
}

struct PreparedStableCutover<'lane, 'selected> {
    publication: PreparedStableLineagePublication<'lane, 'selected>,
    registration: PreparedCurrentSettlementRegistration<'selected>,
    generation: PreparedGenerationAppend,
    locator: PreparedStablePartitionLocator,
    published_cell: Arc<OnceLock<RecordedOutput>>,
    observation: ProductBranchObservation,
}

enum StableInstallation<'lane, 'selected> {
    Installed {
        published: PublishedStableLineage,
        source_cleanup: CurrentSettlementRegistrationCleanup,
        retired_generation: Option<RecordedGeneration>,
        retired_locator: Option<Arc<OnceLock<usize>>>,
        address: super::PreparedStableLineageAddress<'lane, 'selected>,
        facts: Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>,
    },
    Stopped {
        prepared: PreparedStableCutover<'lane, 'selected>,
        reason: StableInstallationStop,
    },
}

enum StableInstallationStop {
    SourceMoved,
    RequestAuthority(WorthQueryOperationAuthorizationDenialKind),
}

impl<'lane, 'selected> PreparedStableLineagePublication<'lane, 'selected> {
    pub(in crate::domain_computation::primary_graph) fn publish<
        Binding: 'static,
        Schema,
        Operation,
        Input,
        Scope,
    >(
        mut self,
        operation: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        observation: &ProductBranchObservation,
        world: &RuntimeWorldObservationPort,
        registration: PreparedCurrentSettlementRegistration<'selected>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PublishedStableLineage, StablePublicationStop> {
        let scope = operation.operation_scope_binding();
        admission
            .charge_external_work(12)
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        if !Arc::ptr_eq(registration.identity(), &self.address.identity)
            || !std::ptr::eq(
                registration.selected_source(),
                self.address.verified.selected,
            )
            || self.address.coordinate.occurrence != observation.lifecycle_incarnation()
            || self.address.coordinate.generation != observation.reference_generation().get()
            || !self.address._coordination.admits(observation)
        {
            return Err(denial(Kind::PublicationStale).into());
        }
        let comparison = u64::try_from(CurrentProductHead::comparison_work_bound(observation))
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        admission
            .charge_external_work(comparison)
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        let owner = Arc::clone(&self.address.owner);
        let mut lineage = owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // The occurrence lane excludes its own competing producer, while this
        // final locked selection also excludes retirement and other writers.
        let revalidation = (|| {
            lineage.drain_cancelled_slots(admission)?;
            let actual = lineage.prior_input_cutoff_candidate::<Binding>(scope, observation, self.address.partition, admission)
                .map_err(|stop| match stop {
                    worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted { .. }
                    | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => denial(Kind::WorkBudgetExceeded),
                    _ => denial(Kind::RetainedBasisUnavailable),
                })?;
            if !actual.as_ref().is_some_and(|actual| {
                Arc::ptr_eq(&actual.cell, &self.address.verified.candidate.cell)
            }) {
                return Err(denial(Kind::PublicationStale));
            }
            admission
                .charge_external_work(
                    tree_work::<worth_runtime_world::facade::ProductBranchIncarnation>(
                        lineage.live_occurrences.len(),
                    )
                    .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
                )
                .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
            if !lineage
                .live_occurrences
                .contains(&self.address.coordinate.occurrence)
            {
                return Err(denial(Kind::PublicationStale));
            }
            Ok(())
        })();
        if let Err(stop) = revalidation {
            drop(lineage);
            return Err(stop.into());
        }
        let generation = match lineage.prepare_stable_generation(
            &self.address.source,
            self.address.coordinate,
            self.address.expected_record_count,
            self.address
                .retained_capacity
                .as_mut()
                .expect("prepared alias retains its capacity"),
            admission,
        ) {
            Ok(prepared) => prepared,
            Err(stop) => {
                drop(lineage);
                return Err(stop.into());
            }
        };
        let locator = match lineage.partition_index.prepare_stable_locator(
            &self.address.source,
            self.address.coordinate,
            self.address.partition,
            self.address
                .retained_capacity
                .as_mut()
                .expect("prepared alias retains its capacity"),
            admission,
        ) {
            Ok(prepared) => prepared,
            Err(stop) => {
                let removed = lineage.abandon_stable_generation(
                    &self.address.source,
                    self.address.coordinate,
                    &generation,
                );
                drop(lineage);
                drop((removed, generation));
                return Err(stop.into());
            }
        };
        // All custody is an explicit callback argument. If World refuses entry,
        // dropping the unused callback cannot drop prepared state under locks.
        let argument = PreparedStableCutover {
            published_cell: Arc::clone(&self.address.record_cell),
            observation: observation.clone(),
            publication: self,
            registration,
            generation,
            locator,
        };
        // Time and cancellation authority belongs to the admitted operation.
        // Revalidate after all storage preparation, before either owner effect.
        if let Err(stop) = operation.validate_current_authority() {
            lineage.partition_index.abandon_stable_locator(
                &argument.publication.address.source,
                argument.publication.address.coordinate,
                argument.publication.address.partition,
                &argument.locator,
            );
            let removed = lineage.abandon_stable_generation(
                &argument.publication.address.source,
                argument.publication.address.coordinate,
                &argument.generation,
            );
            drop(lineage);
            drop((removed, argument));
            return Err(StablePublicationStop::RequestAuthority(stop));
        }
        let result =
            world.while_product_branch_current(observation, argument, |argument, _current| {
                // Product lock acquisition may have waited past cancellation
                // or expiry. Sample the same owner before the source CAS; carry
                // its Copy stop out before constructing the allocating denial.
                if let Some(kind) = operation.current_authority_stop_kind() {
                    return StableInstallation::Stopped {
                        prepared: argument,
                        reason: StableInstallationStop::RequestAuthority(kind),
                    };
                }
                let PreparedStableCutover {
                    mut publication,
                    registration,
                    generation,
                    locator,
                    published_cell,
                    observation,
                } = argument;
                let cleanup = match registration.install() {
                    Ok(cleanup) => cleanup,
                    Err(stopped) => {
                        let (_, registration) = stopped.into_parts();
                        return StableInstallation::Stopped {
                            prepared: PreparedStableCutover {
                                publication,
                                registration,
                                generation,
                                locator,
                                published_cell,
                                observation,
                            },
                            reason: StableInstallationStop::SourceMoved,
                        };
                    }
                };
                // No admission, allocation, owner reread, or fallible preparation
                // follows the source CAS. These cells and buffers were reserved.
                publication.recorded._retained_capacity =
                    publication.address.retained_capacity.take();
                assert!(publication
                    .address
                    .record_cell
                    .set(publication.recorded)
                    .is_ok());
                let retired_generation = lineage.publish_stable_generation(
                    &publication.address.source,
                    publication.address.coordinate,
                    Arc::clone(&publication.address.record_cell),
                    generation,
                );
                let retired_locator = lineage.partition_index.publish_stable_locator(
                    &publication.address.source,
                    publication.address.coordinate,
                    publication.address.partition,
                    publication.address.expected_record_count,
                    locator,
                );
                StableInstallation::Installed {
                    published: PublishedStableLineage {
                        cell: published_cell,
                        observation,
                    },
                    source_cleanup: cleanup,
                    retired_generation,
                    retired_locator,
                    address: publication.address,
                    facts: publication.facts,
                }
            });
        let (stopped, reason) = match result {
            Ok(StableInstallation::Installed {
                published,
                source_cleanup,
                retired_generation,
                retired_locator,
                address,
                facts,
            }) => {
                drop(lineage);
                drop((
                    source_cleanup,
                    retired_generation,
                    retired_locator,
                    address,
                    facts,
                ));
                return Ok(published);
            }
            Ok(StableInstallation::Stopped { prepared, reason }) => (prepared, reason),
            Err(ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(argument))
            | Err(ProductBranchCurrentnessFailure::AdmissionDenied { argument, .. })
            | Err(ProductBranchCurrentnessFailure::PreparationDenied(argument)) => {
                (argument, StableInstallationStop::SourceMoved)
            }
        };
        lineage.partition_index.abandon_stable_locator(
            &stopped.publication.address.source,
            stopped.publication.address.coordinate,
            stopped.publication.address.partition,
            &stopped.locator,
        );
        let removed = lineage.abandon_stable_generation(
            &stopped.publication.address.source,
            stopped.publication.address.coordinate,
            &stopped.generation,
        );
        drop(lineage);
        drop((removed, stopped));
        Err(match reason {
            StableInstallationStop::SourceMoved => denial(Kind::PublicationStale).into(),
            StableInstallationStop::RequestAuthority(kind) => {
                StablePublicationStop::RequestAuthority(
                    WorthQueryOperationAuthorizationDenial::new(kind, operation.operation()),
                )
            }
        })
    }
}

mod view;
