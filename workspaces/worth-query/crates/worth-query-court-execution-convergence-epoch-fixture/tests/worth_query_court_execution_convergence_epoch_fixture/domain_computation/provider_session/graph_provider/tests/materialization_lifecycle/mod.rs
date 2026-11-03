use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::graph_provider::tests::support::attempt;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::graph_provider::tests::support::call;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::graph_provider::tests::support::material;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::graph_provider::tests::support::material_with_rows;
use worth_query_execution::facade::domain_computation::WorthQueryGraphReadMaterial;
use worth_query_execution::court_doors_for_test::domain_computation::provider_session::graph_provider::stream_evidence::WorthQueryGraphReadStreamAccumulator;

pub const MANY_CHUNKS: usize = 50_000;
pub const SMALL_STACK_BYTES: usize = 256 * 1024;

#[test]
fn partial_stream_and_sealed_product_drop_iteratively_on_a_small_stack() {
    let thread = std::thread::Builder::new()
        .name("graph-materialization-drop".to_owned())
        .stack_size(SMALL_STACK_BYTES)
        .spawn(|| {
            let attempt = attempt();
            let call = call(&attempt, "iterative-drop");

            let mut partial = WorthQueryGraphReadStreamAccumulator::new(&call);
            for _ in 0..MANY_CHUNKS {
                partial.admit_chunk(WorthQueryGraphReadMaterial::new([]));
            }
            drop(partial);

            let mut sealed = WorthQueryGraphReadStreamAccumulator::new(&call);
            sealed.admit_chunk(material("first"));
            for _ in 0..MANY_CHUNKS {
                sealed.admit_chunk(WorthQueryGraphReadMaterial::new([]));
            }
            sealed.admit_chunk(material("last"));
            let evidence = sealed.finish(&call);
            let expected = material_with_rows(["first", "last"]);
            assert!(evidence.product().rows().eq(expected.rows().iter()));
            drop(evidence);
        })
        .expect("the small-stack teardown court must start");

    thread
        .join()
        .expect("bounded materialization teardown must not recurse through chunks");
}
