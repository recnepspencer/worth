use worth_query_decl::facade::WorthManagedIndex;

#[derive(WorthManagedIndex)]
#[worth_index(
    schema = Schema,
    entity = Entity,
    source = Source,
    probe = u64,
    kind = interval_1d(start(), end()),
    refinement = Some(ApplicationManagedIndexRefinement::row_local("example.exact", 1)),
)]
struct StockIndex;

fn main() {}
