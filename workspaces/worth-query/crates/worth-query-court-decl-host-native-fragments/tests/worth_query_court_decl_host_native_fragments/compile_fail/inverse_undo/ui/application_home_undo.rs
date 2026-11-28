use worth_query_decl::facade::application_program::ApplicationProgramDefinition;
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_host::facade::WorthQueryApplicationHome;

fn undo<Schema, Program>()
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    let _ = WorthQueryApplicationHome::<Schema, Program>::undo;
}

fn main() {}
