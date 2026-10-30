//! The existing installed program, real principal, handlers and fresh Host reads.
use super::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc, Mutex,
};
mod seed;

pub(super) struct World {
    pub application: support::Application,
    authority: WorthQueryApplicationInvariantProjectionAuthority<CheckpointSchema>,
    events: Arc<Mutex<Vec<PlanarInvariantReadEvent>>>,
    command: u64,
}
impl World {
    pub fn new(rings: usize, coupling: &[(String, String)]) -> Self {
        let events = Arc::new(Mutex::new(Vec::new()));
        let capture = events.clone();
        let configuration = TopologyConfiguration {
            final_feature: crate::planar_final_feature_runtime(),
            setup_calls: Arc::new(AtomicUsize::new(0)),
            invariant_calls: Arc::new(AtomicUsize::new(0)),
            invariant_probe: Arc::new(AtomicUsize::new(0)),
            invariant_read_observer: Some(Arc::new(move |event| {
                capture.lock().unwrap().push(event)
            })),
            producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
            producer_domain_denial: Arc::new(AtomicBool::new(false)),
            producer_contacts: None,
        };
        let mut authority = None;
        // Bootstrap authors up to 30,006 bodies and their real source topology.
        // Both locality sizes use the same original accounted request funding;
        // the fixed split's measured business reads exclude that construction.
        let policy = worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                None,
                Some(1_000_000_000),
            ),
        );
        let limits = support::limits_with_policy(
            support::DEFAULT_RETAINED_COMMITS,
            16,
            64,
            support::invalidation(),
            policy,
        );
        let application =
            worth_query_host::facade::application_contribution::with_bootstrap_advancement(
                support::observation::POPULATION_BOOTSTRAP_POLICY,
                |phase| {
                    support::install_configured_in_bootstrap::<CheckpointProgram>(
                        &phase,
                        Default::default(),
                        limits,
                        |graph| {
                            authority = Some(graph.retain_invariant_projection_authority());
                            seed::rings(graph, rings, coupling);
                        },
                        configuration,
                    )
                },
            )
            .expect("the original population setup request opens")
            .expect("population setup leaves the World's request policy unchanged");
        events.lock().unwrap().clear();
        Self {
            application,
            authority: authority.unwrap(),
            events,
            command: 0,
        }
    }

    pub fn couple(
        &mut self,
        from: String,
        to: String,
        linked: bool,
    ) -> Vec<PlanarInvariantReadEvent> {
        self.events.lock().unwrap().clear();
        self.command += 1;
        let (scope, principal) = authenticate(&self.application);
        let request = self.application.request(&principal, &scope);
        let outcome = request
            .advanced()
            .mutate(PlanarIslandCoupling {
                scope_key: from,
                target_key: to,
                linked,
            })
            .idempotency(&self.command)
            .execute_in_program(
                &self.application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("the actual coupling operation is admitted");
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "coupling edit must publish: {outcome:?}"
        );
        std::mem::take(&mut *self.events.lock().unwrap())
    }

    pub fn replace_anchor(&mut self) {
        use worth_query_consumer_values::{PlanarVertex, PlanarVertexReplacement};
        self.command += 1;
        let (scope, principal) = authenticate(&self.application);
        let request = self.application.request(&principal, &scope);
        let source = request
            .query(PlanarRead {
                body_key: key(0, "c"),
            })
            .execute()
            .unwrap();
        let outcome = request
            .advanced()
            .mutate(VertexReplacement {
                scope_key: key(0, "c"),
                replacement: PlanarVertexReplacement {
                    retired_key: key(0, "a"),
                    next_key: key(0, "b"),
                    replacement: PlanarVertex {
                        body_key: key(0, "replacement"),
                        x: length(1),
                        y: length(1),
                    },
                },
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&self.command)
            .execute_in_program(
                &self.application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "least-member replacement must publish: {outcome:?}"
        );
    }

    pub fn members(&self, keys: &[String]) -> BTreeMap<String, Member> {
        self.project(|reader| {
            keys.iter()
                .map(|key| {
                    (
                        key.clone(),
                        reader
                            .resolve_entity(BodyKey::reference(), key.clone())
                            .unwrap(),
                    )
                })
                .collect()
        })
    }

    pub fn groups(&self, keys: &[String]) -> Vec<Group> {
        self.project(|reader| {
            let mut covered = BTreeSet::new();
            let mut groups = Vec::new();
            for key in keys {
                let member = reader
                    .resolve_entity(BodyKey::reference(), key.clone())
                    .unwrap();
                if covered.contains(&member) {
                    continue;
                }
                let component = reader
                    .component_of(PlanarIslands::reference(), &member)
                    .unwrap();
                let members: BTreeSet<_> = component.members().collect();
                covered.extend(members.iter().cloned());
                groups.push(Group {
                    name: component.identity().clone(),
                    members,
                });
            }
            groups.sort_by(|a, b| a.members.first().cmp(&b.members.first()));
            groups
        })
    }

    pub fn history(&self, name: &Name) -> History {
        self.project(|reader| {
            let lineage = reader
                .component_lineage(PlanarIslands::reference(), name)
                .unwrap();
            let mut events: Vec<_> = lineage
                .events()
                .map(|event| (event.kind(), *event.from().bytes(), *event.to().bytes()))
                .collect();
            events.sort();
            History {
                events,
                latest: lineage.latest_identity().map(|id| *id.bytes()),
                retired: lineage.retired_at().is_some(),
            }
        })
    }

    fn project<T>(
        &self,
        read: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, CheckpointSchema>,
        ) -> T,
    ) -> T {
        let completed = self
            .application
            .with_host_advancement(|phase| {
                self.authority
                    .project(&phase, read, ExecutionAllocationPolicy::SystemAllocation)
                    .unwrap()
            })
            .expect("fresh Host phase admits privileged inspection");
        let (value, snapshot, _) = completed.into_parts();
        drop(snapshot);
        value
    }
}
impl Drop for World {
    fn drop(&mut self) {
        assert!(self.application.close_live_delivery().owner_terminal());
        self.application.close_conditional_runtime().unwrap();
    }
}
