"""Establish the installed arm64 loader's mapped extent from its Mach-O image."""
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
import platform
import sys
from .macho import InvalidImage, read_arm64_images
from .refusal import LoaderBoundsUnavailable

# XNU vm_map.c:21934-21944, vm_map_get_max_loader_aslr_slide_pages:
# (1 << (22 - page_shift)) pages. The actual random slide is strictly below it.
# https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/vm/vm_map.c
LOADER_SLIDE_BOUND = 1 << 22


@dataclass(frozen=True)
class LoaderBounds:
    slide: int
    extent: int


def read_bounds(path):
    try:
        images = read_arm64_images(path)
        for image in images:
            if image.kind != 7 or not image.mapped or min(s.address for s in image.mapped) != 0:
                raise InvalidImage('expected zero-based arm64 MH_DYLINKER images')
        # Bound every installed arm64 subtype, including authenticated variants;
        # selecting one subtype heuristically cannot establish the loader bound.
        return LoaderBounds(LOADER_SLIDE_BOUND, max(s.end for image in images for s in image.mapped))
    except (OSError, InvalidImage, ValueError) as error:
        raise LoaderBoundsUnavailable(f'cannot establish arm64 dyld bounds from {path}: {error}') from error


@lru_cache(maxsize=1)
def host_loader():
    if sys.platform != 'darwin' or platform.machine() != 'arm64':
        raise LoaderBoundsUnavailable('arm64 Darwin loader bounds require an arm64 macOS host')
    return read_bounds(Path('/usr/lib/dyld'))
