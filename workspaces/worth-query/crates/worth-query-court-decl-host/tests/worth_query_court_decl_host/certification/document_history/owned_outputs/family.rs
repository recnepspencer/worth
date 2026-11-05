//! Two model-scoped families, each split into Initial and Preserve producers that
//! publish exactly one derived output owned by its source model (SourceScopeCascade).
use super::*;
#[macro_use]
mod phase;
mod readiness;
use readiness::readiness_query;
#[derive(Clone, Debug)]
pub enum Denial {
    InitialHasPriorOutput,
    PreserveMissingOutput,
}
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "owned-outputs.producer-denial.v1" });
worth_query_structured_value_binding!(pub ResultBinding for () { identity: "owned-outputs.producer-result.v1" });
pub type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, Schema>;
macro_rules! family {
    ($module:ident, $ownership:ident, $direction:ident, $source_entity:ident, $source_facts:ident, $source_key:ident, $source_value:ident, $source_query:ident, $profile:literal, $initial:literal, $preserve:literal, $initial_op:ident, $preserve_op:ident,
     $entity:ident, $facts:ident, $key:ident, $value:ident, $of:ident, $tree:ident, $derive:expr) => {
        pub mod $module {
            use super::super::schema::{
                $source_entity as SourceEntity, $source_facts as SourceFacts,
                $source_key as SourceKey, $source_value as SourceValue,
            };
            use super::*;
            type Scope = ApplicationMutationFieldScope<
                Schema,
                SourceEntity,
                SourceFacts,
                SourceKey,
                String,
                ReadOnly,
                NoApplicationUnit,
            >;
            const PROFILE: &str = $profile;
            pub fn output_key(row: &query::Row) -> String {
                format!("{}:{}", PROFILE, row.key)
            }
            pub fn derived(value: u64) -> u64 {
                ($derive)(value)
            }
            pub struct Family;
            impl WorthQueryProducerOutputFamily<Schema> for Family {
                type Source = query::$source_query::Binding;
                type Entity = $entity;
                const IDENTITY: &'static str = PROFILE;
                const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[
                    WorthQueryProducerApplicability::new(
                        PROFILE,
                        WorthQueryProducerLifecyclePosture::Initial,
                    ),
                    WorthQueryProducerApplicability::new(
                        PROFILE,
                        WorthQueryProducerLifecyclePosture::Preserve,
                    ),
                ];
                fn profile_kind(_: &query::Row) -> &'static str {
                    PROFILE
                }
            }
            #[derive(Clone)]
            pub struct Demand(pub String);
            impl WorthQueryApplicationOutputDemand<Schema> for Demand {
                type OutputFamily = Family;
                fn source_intent(&self) -> query::$source_query::Read {
                    query::$source_query::Read(self.0.clone())
                }
            }
            phase!(
                initial,
                $ownership,
                $direction,
                $source_query,
                $initial_op,
                false,
                Initial,
                WorthQueryCreateOutput,
                create_output,
                $initial,
                $entity,
                $facts,
                $key,
                $value,
                $of
            );
            phase!(
                preserve,
                $ownership,
                $direction,
                $source_query,
                $preserve_op,
                true,
                Preserve,
                WorthQueryPreserveOutput,
                preserve_output,
                $preserve,
                $entity,
                $facts,
                $key,
                $value,
                $of
            );
            pub fn declare(
                schema: ApplicationSchemaDeclarationBuilder<Schema>,
            ) -> ApplicationSchemaDeclarationBuilder<Schema> {
                preserve::declare(initial::declare(schema))
            }
            pub fn contracts(
                contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
                contracts.producer::<initial::Producer>()?;
                contracts.producer::<preserve::Producer>()?;
                contracts.conditional::<initial::Readiness>()?;
                contracts.conditional::<preserve::Readiness>()?;
                Ok(())
            }
            pub fn install(
                setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
                install_with_derivation(setup, derived)
            }
            pub fn install_with_derivation(
                setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
                derive: fn(u64) -> u64,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
                setup.handler::<initial::Binding, _>(initial::Handler { derive })?;
                setup.handler::<preserve::Binding, _>(preserve::Handler { derive })?;
                setup.producer::<initial::Producer>(initial::Provider)?;
                setup.producer::<preserve::Producer>(preserve::Provider)?;
                setup.conditional::<initial::Readiness>(())?;
                setup.conditional::<preserve::Readiness>(())
            }
            /// Demand and settle this family for one model on the request's branch.
            pub fn settle(host: &world::Host, request: &Request<'_, '_, '_>, model: &str) {
                let mut operation = request
                    .demand(Demand(model.into()))
                    .retain_program_outputs::<program::Program, program::$tree>(host)
                    .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                for _ in 0..128 {
                    match operation.advance(request) {
                        WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
                        WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                            settled.deliver_current(request).unwrap();
                            settled
                                .close(request)
                                .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                            return;
                        }
                        WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                            panic!("{denial:?}")
                        }
                        WorthQueryProgramOutputOperationAdvance::Terminal(t) => {
                            panic!("{:?}", t.denial())
                        }
                    }
                }
                panic!("{} did not settle", PROFILE);
            }
        }
    };
}
family!(
    first,
    SourceScopeCascade,
    output,
    Model,
    ModelFacts,
    ModelKey,
    ModelValue,
    producer_model,
    "first",
    "owned-outputs.first.initial",
    "owned-outputs.first.preserve",
    FirstInitial,
    FirstPreserve,
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf,
    FirstTree,
    |value: u64| value * 10
);
family!(
    second,
    SourceScopeCascade,
    output,
    Model,
    ModelFacts,
    ModelKey,
    ModelValue,
    producer_model,
    "second",
    "owned-outputs.second.initial",
    "owned-outputs.second.preserve",
    SecondInitial,
    SecondPreserve,
    SecondOutput,
    SecondFacts,
    SecondKey,
    SecondValue,
    SecondOf,
    SecondTree,
    |value: u64| value + 1000
);

family!(
    child,
    Independent,
    none,
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    member_public,
    "child",
    "owned-outputs.child.initial",
    "owned-outputs.child.preserve",
    ChildInitial,
    ChildPreserve,
    ChildOutput,
    ChildFacts,
    ChildKey,
    ChildValue,
    ChildOf,
    ChildTree,
    |value: u64| value * 100
);

family!(
    restricted,
    Independent,
    none,
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    member,
    "restricted",
    "owned-outputs.restricted.initial",
    "owned-outputs.restricted.preserve",
    RestrictedInitial,
    RestrictedPreserve,
    ChildOutput,
    ChildFacts,
    ChildKey,
    ChildValue,
    ChildOf,
    RestrictedTree,
    |value: u64| value * 100
);

family!(
    upstream,
    SourceScopeCascade,
    output,
    Model,
    ModelFacts,
    ModelKey,
    ModelValue,
    model,
    "upstream",
    "owned-outputs.upstream.initial",
    "owned-outputs.upstream.preserve",
    UpstreamInitial,
    UpstreamPreserve,
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    UpstreamOf,
    UpstreamTree,
    |value: u64| value * 5
);
family!(
    downstream,
    SourceScopeCascade,
    source,
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    member_public,
    "downstream",
    "owned-outputs.downstream.initial",
    "owned-outputs.downstream.preserve",
    DownstreamInitial,
    DownstreamPreserve,
    ChildOutput,
    ChildFacts,
    ChildKey,
    ChildValue,
    ChildOf,
    DownstreamTree,
    |value: u64| value * 100
);

family!(
    restricted_owned,
    SourceScopeCascade,
    source,
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    member,
    "restricted-owned",
    "owned-outputs.restricted-owned.initial",
    "owned-outputs.restricted-owned.preserve",
    RestrictedOwnedInitial,
    RestrictedOwnedPreserve,
    ChildOutput,
    ChildFacts,
    ChildKey,
    ChildValue,
    ChildOf,
    RestrictedOwnedTree,
    |value: u64| value * 100
);
