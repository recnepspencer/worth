#![forbid(unsafe_code)]

mod python;
#[cfg(test)]
mod tests;
mod typescript;

pub use python::{
    render_python_product_protocol, render_python_remote_client, WorthServerPythonClientPackage,
};
pub use typescript::render_typescript_product_protocol;

use worth_server::WorthServerProductProtocolCatalog;

pub fn render_python_client_package(
    catalog: &WorthServerProductProtocolCatalog,
) -> WorthServerPythonClientPackage {
    WorthServerPythonClientPackage::new(
        render_python_product_protocol(catalog),
        render_python_remote_client(),
    )
}
