"""Use native sccache statistics without attribution from concurrent lanes."""
from contextlib import contextmanager
import json
import socket
import subprocess


@contextmanager
def isolated_cache(directory, environment):
    # The shared daemon never stops, changes configuration or loses statistics.
    # This finite proof uses its own cache because disk storage admits one owner.
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    assert port != 4226
    environment = environment.copy()
    environment["SCCACHE_SERVER_PORT"] = str(port)
    environment["SCCACHE_DIR"] = str(directory / "cache")
    environment["SCCACHE_IDLE_TIMEOUT"] = "60"
    environment.pop("SCCACHE_CLIENT_SIDE", None)
    environment.pop("SCCACHE_LOG", None)
    start = subprocess.run(["sccache", "--start-server"], capture_output=True, text=True, env=environment)
    assert start.returncode == 0, start.stderr
    try:
        yield environment
    finally:
        # The endpoint is the private port successfully started above.
        subprocess.run(["sccache", "--stop-server"], capture_output=True, env=environment, timeout=30)


def statistics(environment):
    result = subprocess.run(["sccache", "--show-stats", "--stats-format", "json"],
                            capture_output=True, text=True, check=True, env=environment)
    return json.loads(result.stdout)["stats"]
