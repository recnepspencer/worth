//! Install the proof's declared application rule and seed its authorized account.
use super::schema::*;
use crate::{Count, Parameter};
use worth_query_consumer_values::PositiveCount;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::domain::WorthQueryInstalledPrincipalBinding;

use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::*;

pub(super) struct World {
    pub(super) application: WorthQueryPrimaryGraphApplicationRuntime<LeaseSchema>,
    pub(super) binding: WorthQueryInstalledPrincipalBinding<
        LeaseSchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    >,
    pub(super) invariant: WorthQueryApplicationInvariantProjectionAuthority<LeaseSchema>,
}

pub(super) fn install() -> World {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use worth_query_host::facade::application_installation::{
        in_memory, WorthQueryInMemoryApplicationLimits,
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let mut invariant = None;
    let mut binding = None;
    let limits = WorthQueryInMemoryApplicationLimits::new(
        super::resources::world_resources(),
        WorthQueryApplicationCandidateResourceProfile::physical_resources(4096, 8192).unwrap(),
        WorthQueryApplicationQueryResourceProfile::bounded(4096, 4096, 4096, 32).unwrap(),
        SignalConditionalEvaluationBudget::development(),
    );
    let application = in_memory(
        LeaseSchema::declaration().unwrap(),
        (Arc::clone(&calls), ()),
        limits,
        |_phase, bootstrap, schema| {
            let principal = schema
                .principal_binding(IdentityBinding::reference())
                .unwrap();
            bootstrap.bind_principal(
                &principal,
                WorthQueryApplicationPrincipalKey::new("principal-0").unwrap(),
                1,
                super::authentication::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            bootstrap.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Parameter::reference(),
                    WorthQueryApplicationEntityKey::new("parameter-1").unwrap(),
                )
                .field(Count::reference(), PositiveCount::new(7).unwrap()),
            )?;
            bootstrap.bind_relation(WorthQueryApplicationRelationSeed::new(
                ParameterOwner::reference(),
                "owner-1",
                WorthQueryApplicationEntityKey::new("principal-0").unwrap(),
                WorthQueryApplicationEntityKey::new("parameter-1").unwrap(),
            ))?;
            invariant = Some(bootstrap.retain_invariant_projection_authority());
            binding = Some(principal);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "the shipped parameter contribution configured its rule"
    );
    World {
        application,
        binding: binding.unwrap(),
        invariant: invariant.unwrap(),
    }
}
