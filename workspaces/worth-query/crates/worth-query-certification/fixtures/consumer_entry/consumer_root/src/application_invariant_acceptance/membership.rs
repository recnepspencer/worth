//! Separate schema and installed program for currentness membership-edit courts.
//! Baseline ConsumerSchema and ConsumerProgram do not opt into this capability.
use worth_query_parameter_entry::{ParameterContribution, ParameterSchemaBinding};
use worth_query_topology_entry::{
    MembershipContribution, MembershipSchemaBinding, TopologyContribution, TopologySchemaBinding,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MembershipSchema;
impl worth_query_decl::facade::application_schema::fragment::Fragment for MembershipSchema {
    const NAME: &'static str = "MembershipSchema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (
        TopologyContribution,
        ParameterContribution,
        MembershipContribution,
    );
}
impl worth_query_decl::facade::application_schema::ApplicationSchema for MembershipSchema {
    const OWNER: &'static str = "worth.query.certification.membership";
    const NAME: &'static str = "MembershipSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclaration<Self>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        worth_query_decl::facade::application_schema::fragment::visit_correction(
            Self, source, visitor,
        )
    }
}
impl worth_query_decl::facade::application_schema::ApplicationSchemaComposition
    for MembershipSchema
{
    type Contributions = (
        TopologyContribution,
        ParameterContribution,
        MembershipContribution,
    );
}

impl TopologySchemaBinding for MembershipSchema {
    type PrincipalFragment = worth_query_topology_entry::PrincipalRecords;
}
impl ParameterSchemaBinding for MembershipSchema {}
impl MembershipSchemaBinding for MembershipSchema {}
mod installation;
mod program;
pub(super) use installation::{install, install_program, install_with_isolated_cycle};
pub(crate) use program::{Fanout, FanoutRoot, MembershipProgram, MembershipRoot};
