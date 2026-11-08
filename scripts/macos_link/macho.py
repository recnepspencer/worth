"""Read the load-command geometry, without loading an executable's payload."""
from dataclasses import dataclass
from pathlib import Path
import struct


class InvalidImage(ValueError):
    pass


@dataclass(frozen=True)
class Segment:
    name: str
    address: int
    size: int
    protection: int
    maximum_protection: int = 0

    @property
    def end(self):
        return self.address + self.size


@dataclass(frozen=True)
class Image:
    kind: int
    flags: int
    segments: tuple
    tls: bool

    @property
    def mapped(self):
        return tuple(s for s in self.segments if s.size and not
                     (s.name == '__PAGEZERO' and s.address == 0
                      and s.protection == 0 and s.maximum_protection == 0))


def arm64_slices(stream, file_size):
    header = stream.read(8)
    if len(header) != 8:
        raise InvalidImage('truncated Mach-O header')
    magic, count = struct.unpack('>II', header)
    if magic not in (0xcafebabe, 0xcafebabf):
        return ((0, file_size),)
    entry_size = 20 if magic == 0xcafebabe else 32
    if count * entry_size > file_size - 8:
        raise InvalidImage('fat architecture table extends beyond the image')
    candidates = []
    for _ in range(count):
        entry = stream.read(entry_size)
        if entry_size == 20:
            cpu, subtype, offset, size, _ = struct.unpack('>5I', entry)
        else:
            cpu, subtype, offset, size, _, _ = struct.unpack('>IIQQII', entry)
        if offset + size > file_size or size < 32:
            raise InvalidImage('fat architecture slice extends beyond the image')
        if cpu == 0x100000c:
            candidates.append((subtype & 0xffffff, offset, size))
    if not candidates:
        raise InvalidImage('universal image has no arm64 slice')
    return tuple((offset, size) for _, offset, size in sorted(candidates))


def read(path):
    images = read_arm64_images(path)
    if len(images) != 1:
        raise InvalidImage('expected one arm64 executable image; multiple subtypes need separate admission')
    return images[0]


def read_arm64_images(path):
    with Path(path).open('rb') as stream:
        slices = arm64_slices(stream, Path(path).stat().st_size)
        return tuple(read_image(stream, offset, size) for offset, size in slices)


def read_image(stream, offset, slice_size):
    stream.seek(offset)
    header = stream.read(32)
    if len(header) != 32:
        raise InvalidImage('truncated Mach-O header')
    magic, cpu, _, kind, count, size, flags, _ = struct.unpack('<8I', header)
    if magic != 0xfeedfacf or cpu != 0x100000c:
        raise InvalidImage('expected a little-endian arm64 Mach-O image')
    # Load commands are bounded by the actual file, not a payload-size policy.
    if size > slice_size - 32:
        raise InvalidImage('load commands extend beyond the image')
    commands = stream.read(size)
    offset, segments, tls = 0, [], False
    for _ in range(count):
        if offset + 8 > size:
            raise InvalidImage('truncated load command')
        command, length = struct.unpack_from('<II', commands, offset)
        if length < 8 or offset + length > size:
            raise InvalidImage('invalid load-command extent')
        if command == 0x19:
            if length < 72:
                raise InvalidImage('truncated segment')
            name, address, extent, _, _, maximum, protection, sections, _ = struct.unpack_from(
                '<16sQQQQiiII', commands, offset + 8)
            if length != 72 + sections * 80 or address + extent >= 2**64:
                raise InvalidImage('invalid segment/section extent')
            segments.append(Segment(name.rstrip(b'\0').decode('ascii'), address, extent,
                                    protection, maximum))
            for section in range(sections):
                section_flags = struct.unpack_from('<I', commands,
                                                   offset + 72 + section * 80 + 64)[0]
                tls |= 0x11 <= section_flags & 0xff <= 0x15
        offset += length
    if offset != size:
        raise InvalidImage('unconsumed load commands')
    return Image(kind, flags, tuple(segments), tls)
