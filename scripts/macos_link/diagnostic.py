"""Best-effort diagnosis after a real launch refusal; preserves its outcome."""
import sys
from .macho import InvalidImage, read
from .placement import UnsafeLayout, LoaderBoundsUnavailable, validate_historical


def diagnose(path):
    if sys.platform != 'darwin':
        return None
    try:
        image = read(path)
        validate_historical(image)
    except LoaderBoundsUnavailable as error:
        return f'MACOS_EXEC_LAYOUT: {path}: {error}; layout cannot be classified. No retry was attempted.'
    except UnsafeLayout as error:
        return (f'MACOS_EXEC_LAYOUT: {path}: {error}. This layout can be refused with '
                'OS_REASON_EXEC/MAP_EXEC_FAILURE (#67). Rebuild with the declared '
                'arm64 Darwin linker; no retry was attempted.')
    except (OSError, InvalidImage, ValueError):
        pass
    return None
