use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
use worth_query_topology_entry::{TopologyContribution, TopologySchemaBinding};
struct ConsumerSchema;
impl Fragment for ConsumerSchema {
    const NAME: &'static str = "ConsumerSchema";
    type Bindings = NoBindings;
    type Members = (TopologyContribution,);
}
impl ApplicationSchema for ConsumerSchema {
    const OWNER: &'static str = "worth.query.certification.consumer";
    const NAME: &'static str = "ConsumerSchema";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
struct ForeignSchema;
impl Fragment for ForeignSchema {
    const NAME: &'static str = "ForeignSchema";
    type Bindings = NoBindings;
    type Members = ();
}
impl ApplicationSchema for ForeignSchema {
    const OWNER: &'static str = "worth.query.certification.foreign";
    const NAME: &'static str = "ForeignSchema";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl TopologySchemaBinding for ForeignSchema {
    type PrincipalFragment = worth_query_topology_entry::PrincipalRecords;
}
fn main() {
    let _ = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration();
}
