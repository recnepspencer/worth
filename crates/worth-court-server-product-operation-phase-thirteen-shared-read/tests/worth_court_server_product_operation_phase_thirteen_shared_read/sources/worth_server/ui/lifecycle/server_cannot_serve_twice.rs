use worth_server::{surfaces::WorthNativeSurface, WorthServer, WorthServerConfig};

fn main() {
    let server = WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .build()
                .unwrap(),
        )
        .register_surface(WorthNativeSurface::disabled())
        .build()
        .unwrap();

    let _first_serve = server.serve();
    let _second_serve = server.serve();
}
