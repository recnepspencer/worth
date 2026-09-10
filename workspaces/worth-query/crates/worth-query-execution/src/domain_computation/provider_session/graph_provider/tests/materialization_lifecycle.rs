use super::{attempt, call, material, material_with_rows};
use crate::domain_computation::{
    WorthQueryGraphReadMaterial, WorthQueryGraphReadStreamAccumulator,
};

const MANY_CHUNKS: usize = 50_000;
const SMALL_STACK_BYTES: usize = 256 * 1024;

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
