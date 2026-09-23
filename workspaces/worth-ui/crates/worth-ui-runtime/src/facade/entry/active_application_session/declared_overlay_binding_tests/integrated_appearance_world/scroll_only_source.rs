/// The shared component/region geometry with no Portal or Motion declarations.
pub(in super::super) fn scroll_only_source() -> String {
    let mut source =
        "surface workspace.surface.overlay {}\nsurface workspace.surface.secondary {}\n".to_owned();
    for (index, component) in super::COMPONENTS.iter().enumerate() {
        let role = if index == 0 {
            "overlay.content".to_owned()
        } else {
            format!("overlay.content{index}")
        };
        source.push_str(&super::role_source(component, &role));
        source.push_str(&format!("component {component} {{ appearance {{ role {role} }} region workspace.region.primary {{ sizing workspace.sizing.mosaic_support; }} }}\n"));
    }
    source
}
