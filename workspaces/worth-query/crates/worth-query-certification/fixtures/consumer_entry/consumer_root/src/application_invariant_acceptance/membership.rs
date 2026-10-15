//! Separate schema and installed program for currentness membership-edit courts.
//! Baseline ConsumerSchema and ConsumerProgram do not opt into this capability.
use worth_query_decl::facade::worth_query_application;
use worth_query_parameter_entry::{ParameterContribution, ParameterSchemaBinding};
use worth_query_topology_entry::{
    MembershipContribution, MembershipSchemaBinding, TopologyContribution, TopologySchemaBinding,
};
worth_query_application! {
    pub MembershipSchema {
        owner: "worth.query.certification.membership",
        version: (1, 0),
        contributions: [TopologyContribution, ParameterContribution, MembershipContribution],
    }
}
impl TopologySchemaBinding for MembershipSchema {}
impl ParameterSchemaBinding for MembershipSchema {}
impl MembershipSchemaBinding for MembershipSchema {}
mod installation;
mod program;
pub(super) use installation::{install, install_program};
pub(crate) use program::{Fanout, FanoutRoot, MembershipProgram, MembershipRoot};
