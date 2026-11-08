"""Keep PIE mappings out of arm64 Darwin's shared-cache and fixed reservations."""
# XNU's arm64 main-executable slide: section 0..2 (32 MiB), plus <16 MiB.
# https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/vm/vm_map.c
MAX_SLIDE = 80 * 1024**2
CACHE_BASE = 0x180000000
# Above the low cache, commpage (63..64 GiB), and GPU (64..448 GiB) reservations.
HIGH_BASE = 0x8000000000
# macOS arm64 MACH_VM_MAX_ADDRESS_RAW (47-bit user address space).
USER_END = 0x7ffffe000000
PAGE = 0x4000


class UnsafeLayout(ValueError):
    pass


def unsafe_segments(image):
    return tuple(s for s in image.mapped
                 if not (s.end + MAX_SLIDE <= CACHE_BASE or (s.address >= HIGH_BASE and s.end + MAX_SLIDE <= USER_END)))


def validate(image):
    if image.kind != 2:
        raise UnsafeLayout('expected an executable')
    if not image.flags & 0x200000:
        raise UnsafeLayout('executable must retain PIE/ASLR')
    unsafe = unsafe_segments(image)
    if unsafe:
        geometry = ', '.join(f'{s.name}=[{s.address:#x},{s.end:#x})' for s in unsafe)
        raise UnsafeLayout(f'ASLR mappings overlap reserved address space: {geometry}')


def linkedit_arguments():
    return [f'-Wl,-segaddr,__LINKEDIT,{HIGH_BASE:#x}']


def relocated_arguments(image):
    if image.tls:
        raise UnsafeLayout('code/data enter the shared-cache slide range; Apple ld cannot '
                           'relocate this TLS image safely (thread-locals exceed its 4 GiB '
                           'addressing range)')
    body = [s for s in image.mapped if s.name != '__LINKEDIT']
    start = min(s.address for s in body)
    delta = HIGH_BASE - start
    end = max(s.end for s in body) + delta
    addresses = [(s.name, s.address + delta) for s in body]
    addresses.append(('__LINKEDIT', (end + PAGE - 1) // PAGE * PAGE))
    return [f'-Wl,-segaddr,{name},{address:#x}' for name, address in addresses]
