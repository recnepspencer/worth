use worth_query_decl::facade::{
    application_schema::{
        ApplicationSchemaContributionAuthoring, ApplicationSchemaDeclarationBuilder,
    },
    worth_query_application_schema,
};
use worth_query_topology_entry::{TopologyContribution, TopologySchemaBinding};

worth_query_application_schema! {
    schema ConsumerSchema {
        owner: "worth.query.certification.consumer",
        version: (1, 0),
        members: |schema| { schema }
    }
}

worth_query_application_schema! {
    schema ForeignSchema {
        owner: "worth.query.certification.foreign",
        version: (1, 0),
        members: |schema| { schema }
    }
}

impl TopologySchemaBinding for ForeignSchema {}

fn main() {
    let _ = ApplicationSchemaDeclarationBuilder::<ConsumerSchema>::for_schema()
        .contributions()
        .register::<TopologyContribution>();
}
