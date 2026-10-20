use super::{
    WorthQueryPublishedApplicationQueryReleasePosture as Basis,
    WorthQueryPublishedApplicationQueryResultBufferRelease as Buffer,
    WorthQueryPublishedApplicationQueryTerminalRelease as Release,
};

#[test]
fn aggregate_requires_every_terminal_release_axis() {
    let released = Release {
        application_basis: Basis::Released,
        graph_read_basis: Basis::Released,
        result_buffer: Buffer::Released {
            limit_bytes: Some(64),
            peak_bytes: 32,
        },
        released_graph_capacity_reservation_count: 1,
    };
    assert!(released.resources_released());

    assert!(!Release {
        application_basis: Basis::ReleaseFailed,
        ..released
    }
    .resources_released());
    assert!(!Release {
        graph_read_basis: Basis::ReleaseFailed,
        ..released
    }
    .resources_released());
    assert!(!Release {
        result_buffer: Buffer::Missing,
        ..released
    }
    .resources_released());
    assert!(!Release {
        result_buffer: Buffer::ReleaseFailed {
            limit_bytes: Some(64),
            peak_bytes: 32,
        },
        ..released
    }
    .resources_released());
    assert!(!Release {
        released_graph_capacity_reservation_count: 0,
        ..released
    }
    .resources_released());
    assert!(!Release {
        released_graph_capacity_reservation_count: 2,
        ..released
    }
    .resources_released());
}
