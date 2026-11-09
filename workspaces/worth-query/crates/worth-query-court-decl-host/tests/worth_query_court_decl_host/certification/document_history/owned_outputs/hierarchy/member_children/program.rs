//! Two installed connections differ only in the child's public/current-permission source.
use super::super::program::{Child, ChildInput, Output, Parent, RootConnection};
use super::*;
macro_rules! child_program {
    ($module:ident, $family:ident, $identity:literal, $connection:literal) => {
        pub mod $module {
            use super::*;
            pub struct Discover;
            impl ApplicationConnectionIdentity for Discover {
                const IDENTITY: &'static str = $connection;
            }
            impl ApplicationOccurrenceConnectionBinding<Schema, Parent, Child> for Discover {}
            pub type ChildConnection =
                ApplicationConnectionRef<Schema, Parent, Output, Child, ChildInput, Discover>;
            impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
                type RootDemand = first::Demand;
                type Discovery = query::members::Read;
                type Demand = $family::Demand;
                const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
                fn discovery_from_root(
                    root: &first::Demand,
                ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
                    Ok(query::members::Read(root.0.clone()))
                }
                fn demands_from_discovery(
                    row: &query::members::Members,
                ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
                    Ok(row
                        .members
                        .iter()
                        .map(|member| $family::Demand(member.key.clone()))
                        .collect())
                }
            }
            pub type Tree = ApplicationDemandOutputGraph<
                RootConnection,
                ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
            >;
            pub struct Program;
            impl ApplicationProgramDefinition<Schema> for Program {
                type Contributions = (Contribution,);
                type Outputs = ApplicationProgramOutputs<Tree>;
                type Rules = ApplicationRuleLeaf;
                const IDENTITY: ApplicationProgramIdentity =
                    ApplicationProgramIdentity::new($identity);
                fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                    super::super::super::program::Program::feature_specs()
                }
            }
        }
    };
}
child_program!(
    public,
    downstream,
    "owned-outputs.member-children.public",
    "owned-outputs.member-children.public-discovery"
);
child_program!(
    protected,
    restricted_owned,
    "owned-outputs.member-children.protected",
    "owned-outputs.member-children.protected-discovery"
);
