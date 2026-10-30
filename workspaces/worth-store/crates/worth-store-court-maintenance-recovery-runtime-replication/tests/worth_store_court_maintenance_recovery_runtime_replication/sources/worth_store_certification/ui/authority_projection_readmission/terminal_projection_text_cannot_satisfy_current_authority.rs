use worth_store_aspect_native::StoreTerminalProjectionText;
use worth_store_authority::StoreCurrentAuthorityWitness;

fn require_current_authority(_: StoreCurrentAuthorityWitness) {}

fn main() {
    let projection_text =
        StoreTerminalProjectionText::new_terminal_projection_text("terminal projection");
    require_current_authority(projection_text);
}
