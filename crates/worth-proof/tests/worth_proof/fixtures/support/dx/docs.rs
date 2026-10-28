#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentationDefaultPathAudit {
    suite: &'static str,
    readme_teaches_pleasant_first: bool,
    readme_teaches_raw_escape_hatch: bool,
    readme_includes_scoped_default_lane: bool,
    happy_path_workflow_includes_raw_equivalent: bool,
    happy_path_workflow_uses_raw_import: bool,
    checked_workflow_teaches_pleasant_and_raw: bool,
    runtime_readmission_workflow_teaches_pleasant_and_raw: bool,
    fixed_join_workflow_teaches_pleasant_and_raw: bool,
    family_lowering_workflow_teaches_pleasant_and_raw: bool,
    authoring_workflow_teaches_pleasant_and_raw: bool,
    low_level_workflow_names_raw_escape_hatch: bool,
    low_level_workflow_uses_raw_import: bool,
    recipes_feature_teaches_pleasant_and_raw: bool,
    checked_transitions_feature_teaches_pleasant_and_raw: bool,
    runtime_readmission_feature_teaches_pleasant_and_raw: bool,
    ready_join_feature_teaches_pleasant_and_raw: bool,
    family_lowering_feature_teaches_pleasant_and_raw: bool,
    boundary_readmission_feature_teaches_pleasant_and_raw: bool,
    artifact_feature_declares_dx_posture: bool,
    assumption_basis_feature_declares_dx_posture: bool,
    freshness_feature_declares_dx_posture: bool,
    family_symbol_resolution_feature_declares_dx_posture: bool,
    family_lifecycle_actions_feature_declares_dx_posture: bool,
    fixed_shape_collections_feature_declares_dx_posture: bool,
    fork_and_join_feature_declares_dx_posture: bool,
    readiness_gates_feature_declares_dx_posture: bool,
    proof_markers_feature_declares_dx_posture: bool,
    proven_vectors_feature_declares_dx_posture: bool,
    structural_facts_feature_declares_dx_posture: bool,
    transition_outcomes_feature_declares_dx_posture: bool,
    witnesses_feature_declares_dx_posture: bool,
    authority_and_workflow_contracts_are_explicit: bool,
}

impl DocumentationDefaultPathAudit {
    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn readme_teaches_pleasant_first(&self) -> bool {
        self.readme_teaches_pleasant_first
    }

    pub fn readme_teaches_raw_escape_hatch(&self) -> bool {
        self.readme_teaches_raw_escape_hatch
    }

    pub fn readme_includes_scoped_default_lane(&self) -> bool {
        self.readme_includes_scoped_default_lane
    }

    pub fn happy_path_workflow_includes_raw_equivalent(&self) -> bool {
        self.happy_path_workflow_includes_raw_equivalent
    }

    pub fn happy_path_workflow_uses_raw_import(&self) -> bool {
        self.happy_path_workflow_uses_raw_import
    }

    pub fn checked_workflow_teaches_pleasant_and_raw(&self) -> bool {
        self.checked_workflow_teaches_pleasant_and_raw
    }

    pub fn runtime_readmission_workflow_teaches_pleasant_and_raw(&self) -> bool {
        self.runtime_readmission_workflow_teaches_pleasant_and_raw
    }

    pub fn fixed_join_workflow_teaches_pleasant_and_raw(&self) -> bool {
        self.fixed_join_workflow_teaches_pleasant_and_raw
    }

    pub fn family_lowering_workflow_teaches_pleasant_and_raw(&self) -> bool {
        self.family_lowering_workflow_teaches_pleasant_and_raw
    }

    pub fn authoring_workflow_teaches_pleasant_and_raw(&self) -> bool {
        self.authoring_workflow_teaches_pleasant_and_raw
    }

    pub fn low_level_workflow_names_raw_escape_hatch(&self) -> bool {
        self.low_level_workflow_names_raw_escape_hatch
    }

    pub fn low_level_workflow_uses_raw_import(&self) -> bool {
        self.low_level_workflow_uses_raw_import
    }

    pub fn recipes_feature_teaches_pleasant_and_raw(&self) -> bool {
        self.recipes_feature_teaches_pleasant_and_raw
    }

    pub fn checked_transitions_feature_teaches_pleasant_and_raw(&self) -> bool {
        self.checked_transitions_feature_teaches_pleasant_and_raw
    }

    pub fn runtime_readmission_feature_teaches_pleasant_and_raw(&self) -> bool {
        self.runtime_readmission_feature_teaches_pleasant_and_raw
    }

    pub fn ready_join_feature_teaches_pleasant_and_raw(&self) -> bool {
        self.ready_join_feature_teaches_pleasant_and_raw
    }

    pub fn family_lowering_feature_teaches_pleasant_and_raw(&self) -> bool {
        self.family_lowering_feature_teaches_pleasant_and_raw
    }

    pub fn boundary_readmission_feature_teaches_pleasant_and_raw(&self) -> bool {
        self.boundary_readmission_feature_teaches_pleasant_and_raw
    }

    pub fn artifact_feature_declares_dx_posture(&self) -> bool {
        self.artifact_feature_declares_dx_posture
    }

    pub fn assumption_basis_feature_declares_dx_posture(&self) -> bool {
        self.assumption_basis_feature_declares_dx_posture
    }

    pub fn freshness_feature_declares_dx_posture(&self) -> bool {
        self.freshness_feature_declares_dx_posture
    }

    pub fn family_symbol_resolution_feature_declares_dx_posture(&self) -> bool {
        self.family_symbol_resolution_feature_declares_dx_posture
    }

    pub fn family_lifecycle_actions_feature_declares_dx_posture(&self) -> bool {
        self.family_lifecycle_actions_feature_declares_dx_posture
    }

    pub fn fixed_shape_collections_feature_declares_dx_posture(&self) -> bool {
        self.fixed_shape_collections_feature_declares_dx_posture
    }

    pub fn fork_and_join_feature_declares_dx_posture(&self) -> bool {
        self.fork_and_join_feature_declares_dx_posture
    }

    pub fn readiness_gates_feature_declares_dx_posture(&self) -> bool {
        self.readiness_gates_feature_declares_dx_posture
    }

    pub fn proof_markers_feature_declares_dx_posture(&self) -> bool {
        self.proof_markers_feature_declares_dx_posture
    }

    pub fn proven_vectors_feature_declares_dx_posture(&self) -> bool {
        self.proven_vectors_feature_declares_dx_posture
    }

    pub fn structural_facts_feature_declares_dx_posture(&self) -> bool {
        self.structural_facts_feature_declares_dx_posture
    }

    pub fn transition_outcomes_feature_declares_dx_posture(&self) -> bool {
        self.transition_outcomes_feature_declares_dx_posture
    }

    pub fn witnesses_feature_declares_dx_posture(&self) -> bool {
        self.witnesses_feature_declares_dx_posture
    }

    pub fn authority_and_workflow_contracts_are_explicit(&self) -> bool {
        self.authority_and_workflow_contracts_are_explicit
    }
}

const README: &str = include_str!("../../../../../README.md");
const HAPPY_PATH_WORKFLOW: &str =
    include_str!("../../../../../docs/workflows/happy-path-recipe-progression.md");
const CHECKED_WORKFLOW: &str =
    include_str!("../../../../../docs/workflows/checked-recipe-progression.md");
const RUNTIME_READMISSION_WORKFLOW: &str =
    include_str!("../../../../../docs/workflows/runtime-readmission.md");
const FIXED_JOIN_WORKFLOW: &str = include_str!("../../../../../docs/workflows/fixed-arity-join.md");
const FAMILY_LOWERING_WORKFLOW: &str =
    include_str!("../../../../../docs/workflows/composition-family-lowering.md");
const AUTHORING_WORKFLOW: &str =
    include_str!("../../../../../docs/workflows/authoring-a-new-proof-flow.md");
const LOW_LEVEL_WORKFLOW: &str =
    include_str!("../../../../../docs/workflows/when-to-stay-low-level.md");
const RECIPES_FEATURE: &str = include_str!("../../../../../docs/features/recipes-and-stages.md");
const CHECKED_TRANSITIONS_FEATURE: &str =
    include_str!("../../../../../docs/features/checked-transitions.md");
const RUNTIME_READMISSION_FEATURE: &str =
    include_str!("../../../../../docs/features/runtime-readmission.md");
const READY_JOIN_FEATURE: &str = include_str!("../../../../../docs/features/ready-recipe-join.md");
const FAMILY_LOWERING_FEATURE: &str =
    include_str!("../../../../../docs/features/deterministic-family-lowering.md");
const BOUNDARY_READMISSION_FEATURE: &str =
    include_str!("../../../../../docs/features/boundary-readmission.md");
const ARTIFACT_FEATURE: &str = include_str!("../../../../../docs/features/artifact.md");
const ASSUMPTION_BASIS_FEATURE: &str =
    include_str!("../../../../../docs/features/assumption-basis.md");
const FRESHNESS_FEATURE: &str =
    include_str!("../../../../../docs/features/freshness-and-downgrade.md");
const FAMILY_SYMBOL_RESOLUTION_FEATURE: &str =
    include_str!("../../../../../docs/features/family-symbol-resolution.md");
const FAMILY_LIFECYCLE_ACTIONS_FEATURE: &str =
    include_str!("../../../../../docs/features/family-lifecycle-actions.md");
const FIXED_SHAPE_COLLECTIONS_FEATURE: &str =
    include_str!("../../../../../docs/features/fixed-shape-collections.md");
const FORK_AND_JOIN_FEATURE: &str = include_str!("../../../../../docs/features/fork-and-join.md");
const READINESS_GATES_FEATURE: &str =
    include_str!("../../../../../docs/features/preconstruction-and-readiness-gates.md");
const PROOF_MARKERS_FEATURE: &str =
    include_str!("../../../../../docs/features/proof-markers-and-sets.md");
const PROVEN_VECTORS_FEATURE: &str = include_str!("../../../../../docs/features/proven-vectors.md");
const STRUCTURAL_FACTS_FEATURE: &str =
    include_str!("../../../../../docs/features/structural-facts.md");
const TRANSITION_OUTCOMES_FEATURE: &str =
    include_str!("../../../../../docs/features/transition-outcomes.md");
const WITNESSES_FEATURE: &str = include_str!("../../../../../docs/features/witnesses.md");
const AUTHORITY_AND_WORKFLOW_CONTRACTS: &str =
    include_str!("../../../../../docs/features/authority-and-workflow-contracts.md");

pub fn documentation_default_path_audit() -> DocumentationDefaultPathAudit {
    DocumentationDefaultPathAudit {
        suite: "pleasant_lane_documentation_default_path_audit",
        readme_teaches_pleasant_first: README.contains("Most consumers should start with:")
            && README.contains("use worth_proof::prelude::*;"),
        readme_teaches_raw_escape_hatch: README.contains("## Raw Escape Hatch")
            && README.contains("use worth_proof::raw::*;"),
        readme_includes_scoped_default_lane: README.contains("Common scoped-default lane:")
            && README.contains("proof_flow()"),
        happy_path_workflow_includes_raw_equivalent: HAPPY_PATH_WORKFLOW
            .contains("## Pleasant Lane First")
            && HAPPY_PATH_WORKFLOW.contains("## Equivalent Raw Surface"),
        happy_path_workflow_uses_raw_import: HAPPY_PATH_WORKFLOW
            .contains("use worth_proof::raw::*;"),
        checked_workflow_teaches_pleasant_and_raw: CHECKED_WORKFLOW
            .contains("## Pleasant Lane First")
            && CHECKED_WORKFLOW.contains("## Equivalent Raw Surface")
            && CHECKED_WORKFLOW.contains("use worth_proof::prelude::*;")
            && CHECKED_WORKFLOW.contains("use worth_proof::raw::*;"),
        runtime_readmission_workflow_teaches_pleasant_and_raw: RUNTIME_READMISSION_WORKFLOW
            .contains("## Pleasant Lane First")
            && RUNTIME_READMISSION_WORKFLOW.contains("## Equivalent Raw Surface")
            && RUNTIME_READMISSION_WORKFLOW.contains(".readmit_with(")
            && RUNTIME_READMISSION_WORKFLOW.contains("use worth_proof::raw::*;"),
        fixed_join_workflow_teaches_pleasant_and_raw: FIXED_JOIN_WORKFLOW
            .contains("## Pleasant Lane First")
            && FIXED_JOIN_WORKFLOW.contains("## Equivalent Raw Surface")
            && FIXED_JOIN_WORKFLOW.contains("join_ready(")
            && FIXED_JOIN_WORKFLOW.contains("compose_ready("),
        family_lowering_workflow_teaches_pleasant_and_raw: FAMILY_LOWERING_WORKFLOW
            .contains("## Pleasant Lane First")
            && FAMILY_LOWERING_WORKFLOW.contains("## Equivalent Raw Surface")
            && FAMILY_LOWERING_WORKFLOW.contains("family_pair(")
            && FAMILY_LOWERING_WORKFLOW.contains(".lower_by("),
        authoring_workflow_teaches_pleasant_and_raw: AUTHORING_WORKFLOW
            .contains("## Pleasant Lane First")
            && AUTHORING_WORKFLOW.contains("## Equivalent Raw Surface")
            && AUTHORING_WORKFLOW.contains("use worth_proof::prelude::*;")
            && AUTHORING_WORKFLOW.contains("use worth_proof::raw::*;"),
        low_level_workflow_names_raw_escape_hatch: LOW_LEVEL_WORKFLOW
            .contains("## Raw Escape Hatch")
            && LOW_LEVEL_WORKFLOW.contains("use worth_proof::raw::*;"),
        low_level_workflow_uses_raw_import: LOW_LEVEL_WORKFLOW.contains("use worth_proof::raw::*;"),
        recipes_feature_teaches_pleasant_and_raw: RECIPES_FEATURE
            .contains("## Pleasant Lane First")
            && RECIPES_FEATURE.contains("## Equivalent Raw Surface")
            && RECIPES_FEATURE.contains("recipe(\"payload\")"),
        checked_transitions_feature_teaches_pleasant_and_raw: CHECKED_TRANSITIONS_FEATURE
            .contains("## Pleasant Lane First")
            && CHECKED_TRANSITIONS_FEATURE.contains("## Equivalent Raw Surface")
            && CHECKED_TRANSITIONS_FEATURE.contains(".try_resolve_ready(")
            && CHECKED_TRANSITIONS_FEATURE.contains("ProofOutcome"),
        runtime_readmission_feature_teaches_pleasant_and_raw: RUNTIME_READMISSION_FEATURE
            .contains("## Pleasant Lane First")
            && RUNTIME_READMISSION_FEATURE.contains("## Equivalent Raw Surface")
            && RUNTIME_READMISSION_FEATURE.contains(".readmit_with("),
        ready_join_feature_teaches_pleasant_and_raw: READY_JOIN_FEATURE
            .contains("## Pleasant Lane First")
            && READY_JOIN_FEATURE.contains("## Equivalent Raw Surface")
            && READY_JOIN_FEATURE.contains("join_ready(")
            && READY_JOIN_FEATURE.contains("compose_join_ready_recipe_pair("),
        family_lowering_feature_teaches_pleasant_and_raw: FAMILY_LOWERING_FEATURE
            .contains("## Pleasant Lane First")
            && FAMILY_LOWERING_FEATURE.contains("## Equivalent Raw Surface")
            && FAMILY_LOWERING_FEATURE.contains("family_pair(")
            && FAMILY_LOWERING_FEATURE.contains("lower_deterministic_family_pair("),
        boundary_readmission_feature_teaches_pleasant_and_raw: BOUNDARY_READMISSION_FEATURE
            .contains("## Pleasant Lane First")
            && BOUNDARY_READMISSION_FEATURE.contains("## Equivalent Raw Surface")
            && BOUNDARY_READMISSION_FEATURE.contains(".readmit_with(")
            && BOUNDARY_READMISSION_FEATURE.contains("readmit_with_authority"),
        artifact_feature_declares_dx_posture: ARTIFACT_FEATURE.contains("## DX Posture")
            && ARTIFACT_FEATURE.contains("use worth_proof::raw::*;"),
        assumption_basis_feature_declares_dx_posture: ASSUMPTION_BASIS_FEATURE
            .contains("## DX Posture")
            && ASSUMPTION_BASIS_FEATURE.contains("use worth_proof::raw::*;"),
        freshness_feature_declares_dx_posture: FRESHNESS_FEATURE.contains("## DX Posture")
            && FRESHNESS_FEATURE.contains("Staleness And Rebind"),
        family_symbol_resolution_feature_declares_dx_posture: FAMILY_SYMBOL_RESOLUTION_FEATURE
            .contains("## DX Posture")
            && FAMILY_SYMBOL_RESOLUTION_FEATURE.contains("sym(...)")
            && FAMILY_SYMBOL_RESOLUTION_FEATURE.contains("resolve_family_symbol"),
        family_lifecycle_actions_feature_declares_dx_posture: FAMILY_LIFECYCLE_ACTIONS_FEATURE
            .contains("## DX Posture")
            && FAMILY_LIFECYCLE_ACTIONS_FEATURE.contains("create(...)")
            && FAMILY_LIFECYCLE_ACTIONS_FEATURE.contains("use worth_proof::raw::*;"),
        fixed_shape_collections_feature_declares_dx_posture: FIXED_SHAPE_COLLECTIONS_FEATURE
            .contains("## DX Posture")
            && FIXED_SHAPE_COLLECTIONS_FEATURE.contains("pair(...)")
            && FIXED_SHAPE_COLLECTIONS_FEATURE.contains("use worth_proof::raw::*;")
            && FIXED_SHAPE_COLLECTIONS_FEATURE
                .contains("DisjointPair::try_from_disjoint(left, right)")
            && FIXED_SHAPE_COLLECTIONS_FEATURE.contains("equal inputs are rejected"),
        fork_and_join_feature_declares_dx_posture: FORK_AND_JOIN_FEATURE.contains("## DX Posture")
            && FORK_AND_JOIN_FEATURE.contains("join_ready(...)")
            && FORK_AND_JOIN_FEATURE.contains("use worth_proof::raw::*;"),
        readiness_gates_feature_declares_dx_posture: READINESS_GATES_FEATURE
            .contains("## DX Posture")
            && READINESS_GATES_FEATURE.contains("try_resolve_ready(...)")
            && READINESS_GATES_FEATURE.contains("use worth_proof::raw::*;"),
        proof_markers_feature_declares_dx_posture: PROOF_MARKERS_FEATURE.contains("## DX Posture")
            && PROOF_MARKERS_FEATURE.contains("use worth_proof::raw::*;"),
        proven_vectors_feature_declares_dx_posture: PROVEN_VECTORS_FEATURE
            .contains("## DX Posture")
            && PROVEN_VECTORS_FEATURE.contains("use worth_proof::raw::*;"),
        structural_facts_feature_declares_dx_posture: STRUCTURAL_FACTS_FEATURE
            .contains("## DX Posture")
            && STRUCTURAL_FACTS_FEATURE.contains("family_pair(...).lower_by(...)"),
        transition_outcomes_feature_declares_dx_posture: TRANSITION_OUTCOMES_FEATURE
            .contains("## DX Posture")
            && TRANSITION_OUTCOMES_FEATURE.contains("ProofOutcome")
            && TRANSITION_OUTCOMES_FEATURE.contains("use worth_proof::raw::*;"),
        witnesses_feature_declares_dx_posture: WITNESSES_FEATURE.contains("## DX Posture")
            && WITNESSES_FEATURE.contains("proof_flow()")
            && WITNESSES_FEATURE.contains("use worth_proof::raw::*;"),
        authority_and_workflow_contracts_are_explicit: AUTHORITY_AND_WORKFLOW_CONTRACTS
            .contains("## Authority Boundary Catalog")
            && AUTHORITY_AND_WORKFLOW_CONTRACTS.contains("## Supported Workflow Catalog")
            && AUTHORITY_AND_WORKFLOW_CONTRACTS.contains("counterfeit")
            && AUTHORITY_AND_WORKFLOW_CONTRACTS.contains("checked DisjointPair construction")
            && AUTHORITY_AND_WORKFLOW_CONTRACTS.contains("trust-boundary bridging"),
    }
}
