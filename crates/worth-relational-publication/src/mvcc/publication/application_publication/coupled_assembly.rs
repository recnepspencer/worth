pub mod adoption;
pub mod assembly_failure;
pub mod authored_writes;
pub mod chain;
pub mod completion;
pub mod continuation {
    pub mod application;
    pub mod custody;
    pub mod round;
}
pub mod contribution;
pub mod corruption_commands;
pub mod disjoint_aspect_patches;
pub mod disjoint_field_patches;
pub mod fresh_references;
pub mod input_issuance;
pub mod inputs;
pub mod local_keys;
pub mod managed_source_alignment {
    pub mod assessment;
    pub mod reach;
    pub mod relevance;
}
pub mod net_plan {
    //! Incremental C-to-final assembly keeps allocation custody separate from effects.
    pub mod creator_custody;
    pub use creator_custody::CreatorCustody;
}
pub mod parent_read_footprint;
pub mod part_naming;
pub mod participant_admission;
pub mod patch_target;
pub mod producer_posture;
pub mod refusals;
pub mod retained_assembly;
pub mod roster_completion;
pub mod route_selection;
pub mod source_effects;
pub mod succession {
    pub mod adoption;
    pub mod guards;
    pub mod indexes;
    pub mod ledger;
    pub mod route;
    pub mod segment;
}
pub mod suffix_scope;
pub mod touch_projection;
pub mod writer_custody;
