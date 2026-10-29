#[allow(unused_imports)] use crate::child_dispatch::*;
use {std::collections::BTreeMap};
use {std::path::Path};

#[test]
fn journey_child_role() {
    let Ok(role) = std::env::var("WORTH_STORE_C4_CHILD_ROLE") else {
        return;
    };
    let root = std::env::var_os("WORTH_STORE_C4_CHILD_ROOT")
        .map(std::path::PathBuf::from)
        .expect("child role requires a root");
    match role.as_str() {
        #[cfg(all())]
        "namespace-writer" | "namespace-reopener" => {
            crate::physical_media_journeys::namespace_discovery::run_child(&role, &root)
        }
        #[cfg(all())]
        "mutation-contender" | "unrelated-inheritance-probe" | "post-death-successor" => {
            crate::physical_media_journeys::mutation_contention::run_child(&role, &root)
        }
        #[cfg(all())]
        "faulted-media-writer" | "fault-reopener" => {
            crate::physical_media_journeys::partial_effects::run_child(&role, &root)
        }
        _ => panic!("unknown C.4 child role: {role}"),
    }
}
