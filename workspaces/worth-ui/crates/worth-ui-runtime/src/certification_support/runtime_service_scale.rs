#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiRuntimeServiceScaleEvidence {
    service_neighborhoods: u64,
    commands: u64,
    focus_participants: u64,
    selection_keys: u64,
    scroll_owners: u64,
    portal_layers: u64,
    active_motion_tracks: u64,
    portal_neighborhoods_visited: u64,
    focus_participants_visited: u64,
    motion_tracks_sampled: u64,
    inactive_motion_tracks_sampled: u64,
    retained_inactive_motion_tracks: u64,
    completed_motion_terminals: u64,
    scroll_chain_depth_visited: u64,
    selection_keys_visited: u64,
    command_candidates_resolved: u64,
    proposal_requirements_visited: u64,
    unrelated_neighborhoods_touched: u64,
    terminal_resources_zero: bool,
    overlay_portal_rows: u64,
    overlay_portal_depth: u64,
    overlay_backdrop_rows: u64,
    overlay_backdrop_categories: [u64; 3],
    overlay_initial_portal_rows_read: u64,
    overlay_initial_bindings_read: u64,
    overlay_initial_backdrops_selected: u64,
    overlay_initial_relation_edges: u64,
    overlay_initial_backdrops_changed: u64,
    overlay_successor_portal_rows_read: u64,
    overlay_successor_bindings_read: u64,
    overlay_successor_backdrops_selected: u64,
    overlay_successor_relation_edges: u64,
    overlay_successor_backdrops_changed: u64,
    overlay_successor_replayed: u64,
    overlay_successor_unrelated_neighborhoods: u64,
    overlay_released_rows: u64,
}

pub fn runtime_service_scale_evidence() -> UiRuntimeServiceScaleEvidence {
    let (portal_layers, portal_neighborhoods_visited, portal_zero) =
        crate::runtime::portal::portal_scale_evidence();
    let (focus_participants, focus_participants_visited, focus_zero) =
        crate::runtime::focus::focus_scale_evidence();
    let motion = crate::runtime::motion::motion_scale_evidence();
    let (scroll_owners, scroll_chain_depth_visited, scroll_zero) =
        crate::runtime::scroll::scroll_scale_evidence();
    let (selection_keys, selection_keys_visited, selection_zero) =
        crate::runtime::selection::selection_scale_evidence();
    let (commands, command_candidates_resolved, command_zero) = command_scale_evidence();
    let proposal = crate::runtime::session::service_proposal::proposal_scale_evidence();
    let overlay = crate::runtime::overlay_composition::overlay_scale_evidence();

    UiRuntimeServiceScaleEvidence {
        // Read back from the live occupancy index, never restated as a literal.
        service_neighborhoods: proposal.neighborhoods,
        commands,
        focus_participants,
        selection_keys,
        scroll_owners,
        portal_layers,
        active_motion_tracks: motion.active_tracks,
        portal_neighborhoods_visited,
        focus_participants_visited,
        motion_tracks_sampled: motion.tracks_sampled,
        inactive_motion_tracks_sampled: motion.inactive_tracks_sampled,
        retained_inactive_motion_tracks: motion.retained_inactive_tracks,
        completed_motion_terminals: motion.completed_terminals,
        scroll_chain_depth_visited,
        selection_keys_visited,
        command_candidates_resolved,
        proposal_requirements_visited: proposal.proposal_requirements_visited,
        unrelated_neighborhoods_touched: proposal.unrelated_neighborhoods_touched,
        terminal_resources_zero: portal_zero
            && focus_zero
            && motion.terminal_resources_zero
            && scroll_zero
            && selection_zero
            && command_zero
            && proposal.terminal_census_is_zero,
        overlay_portal_rows: overlay.portal_rows as u64,
        overlay_portal_depth: overlay.portal_depth as u64,
        overlay_backdrop_rows: overlay.backdrop_rows as u64,
        overlay_backdrop_categories: [
            overlay.dark_backdrops as u64,
            overlay.colored_backdrops as u64,
            overlay.transparent_backdrops as u64,
        ],
        overlay_initial_portal_rows_read: overlay.initial.portal_stack_rows_read() as u64,
        overlay_initial_bindings_read: overlay.initial.portal_binding_entries_read() as u64,
        overlay_initial_backdrops_selected: overlay.initial.backdrop_declarations_selected() as u64,
        overlay_initial_relation_edges: overlay.initial.overlay_relation_edges_visited() as u64,
        overlay_initial_backdrops_changed: overlay.initial.backdrop_mechanics_changed() as u64,
        overlay_successor_portal_rows_read: overlay.successor.portal_stack_rows_read() as u64,
        overlay_successor_bindings_read: overlay.successor.portal_binding_entries_read() as u64,
        overlay_successor_backdrops_selected: overlay.successor.backdrop_declarations_selected()
            as u64,
        overlay_successor_relation_edges: overlay.successor.overlay_relation_edges_visited() as u64,
        overlay_successor_backdrops_changed: overlay.successor.backdrop_mechanics_changed() as u64,
        overlay_successor_replayed: overlay.successor.backdrop_commands_replayed() as u64,
        overlay_successor_unrelated_neighborhoods: overlay
            .successor
            .unrelated_neighborhoods_touched()
            as u64,
        overlay_released_rows: overlay.released_rows as u64,
    }
}

fn command_scale_evidence() -> (u64, u64, bool) {
    use crate::capability::{
        CommandDescriptor, CommandId, UiCommandKeyCode, UiCommandModifierSet,
        UiCommandShortcutSequence, UiCommandShortcutStroke, UiIntentDefinition,
        UiIntentRuntimeServiceDestination,
    };

    let mut builder = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_runtime_service_intent_definition(
            UiIntentDefinition::<CommandScaleIntent>::runtime_service(
                UiIntentRuntimeServiceDestination::InvokeCommand,
            ),
        )
        .expect("command scale destination registers");
    for index in 0..4_096 {
        let key = if index == 2_048 {
            UiCommandKeyCode::F35
        } else {
            UiCommandKeyCode::F34
        };
        builder = builder.register_command(
            CommandDescriptor::new(
                CommandId::new(format!("command.scale.n{index}"))
                    .expect("scale command identity is valid"),
                "Scale command",
            )
            .with_default_shortcut(UiCommandShortcutSequence::single(
                UiCommandShortcutStroke::logical(key, UiCommandModifierSet::none().with_primary()),
            ))
            .with_intent_destination::<CommandScaleIntent>(),
        );
    }
    let app = builder
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("production command registry compiles at scale");
    let mut state = crate::runtime::command_routing::UiCommandRoutingRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
        app.capabilities().commands(),
        crate::declaration::UiCommandRoutingPolicy::desktop(),
    );
    let stroke = UiCommandShortcutStroke::logical(
        UiCommandKeyCode::F35,
        UiCommandModifierSet::none().with_primary(),
    );
    let context = crate::runtime::command_routing::UiCommandRoutingContext::new(
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
            .expect("scale surface identity mints"),
    );
    let generation = crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity::current(
        crate::lifecycle::WorthUiActiveApplicationSessionIdentity::from_host_session_value(1),
        app.generation_identity(),
    );
    let _ = state.route_stroke(stroke, false, context, &generation);
    let (routes, _, _, visited) = state.inspect_for_certification();
    let released = state.shutdown();
    (routes as u64, visited, released == 4_096)
}

struct CommandScalePayload;
struct CommandScaleOutcome;
struct CommandScaleIntent;

impl crate::capability::UiIntentPayload for CommandScalePayload {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("command.scale.payload", 1);
    const FIELDS: crate::capability::UiIntentPayloadFieldSet =
        crate::capability::UiIntentPayloadFieldSet::EMPTY;

    fn project(
        _fields: &mut crate::capability::UiIntentPayloadProjection<Self>,
    ) -> Result<Self, crate::capability::UiIntentPayloadProjectionViolation> {
        Ok(Self)
    }
}

impl crate::capability::UiIntentProductOutcome for CommandScaleOutcome {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("command.scale.outcome", 1);
    const CONSEQUENCE_FAMILIES: crate::capability::UiIntentProductConsequenceFamilies =
        crate::capability::UiIntentProductConsequenceFamilies::NONE;

    fn into_consequences(self) -> crate::capability::UiIntentProductConsequences {
        crate::capability::UiIntentProductConsequences::none()
    }
}

impl crate::capability::UiIntent for CommandScaleIntent {
    type Payload = CommandScalePayload;
    type ProductOutcome = CommandScaleOutcome;

    const ID: crate::capability::UiIntentId =
        crate::capability::UiIntentId::stable("command.scale.intent");
    const ACCEPTED_INTERACTIONS: crate::capability::UiIntentAcceptedInteractions =
        crate::capability::UiIntentAcceptedInteractions::new(&[
            crate::capability::UiSemanticInteractionFamily::Activate,
        ]);
}

macro_rules! getters {
    ($($name:ident),+ $(,)?) => {
        $(pub const fn $name(self) -> u64 { self.$name })+
    };
}

impl UiRuntimeServiceScaleEvidence {
    getters!(
        service_neighborhoods,
        commands,
        focus_participants,
        selection_keys,
        scroll_owners,
        portal_layers,
        active_motion_tracks,
        portal_neighborhoods_visited,
        focus_participants_visited,
        motion_tracks_sampled,
        inactive_motion_tracks_sampled,
        retained_inactive_motion_tracks,
        completed_motion_terminals,
        scroll_chain_depth_visited,
        selection_keys_visited,
        command_candidates_resolved,
        proposal_requirements_visited,
        unrelated_neighborhoods_touched,
    );

    pub const fn terminal_resources_zero(self) -> bool {
        self.terminal_resources_zero
    }

    pub const fn overlay_backdrop_categories(self) -> [u64; 3] {
        self.overlay_backdrop_categories
    }

    getters!(
        overlay_portal_rows,
        overlay_portal_depth,
        overlay_backdrop_rows,
        overlay_initial_portal_rows_read,
        overlay_initial_bindings_read,
        overlay_initial_backdrops_selected,
        overlay_initial_relation_edges,
        overlay_initial_backdrops_changed,
        overlay_successor_portal_rows_read,
        overlay_successor_bindings_read,
        overlay_successor_backdrops_selected,
        overlay_successor_relation_edges,
        overlay_successor_backdrops_changed,
        overlay_successor_replayed,
        overlay_successor_unrelated_neighborhoods,
        overlay_released_rows,
    );
}
