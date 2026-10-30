use worth_foundational::{
    FoundationalMergeScope, FoundationalSelectedAspectLocus,
    FoundationalSelectedAspectRequestEntry, FoundationalSelectedNodeLocus,
};

fn requires_selected_aspect(_: FoundationalSelectedAspectRequestEntry) {}

fn main() {
    let node = FoundationalSelectedNodeLocus::new("gear").unwrap();
    let aspect = FoundationalSelectedAspectLocus::new("teeth").unwrap();

    requires_selected_aspect(node.clone());
    let _ = FoundationalMergeScope::selected_aspects([node]);
    let _ = FoundationalMergeScope::selected_nodes([FoundationalSelectedAspectRequestEntry::new(
        node, aspect,
    )]);
}
