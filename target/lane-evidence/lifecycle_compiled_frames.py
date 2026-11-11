"""Read static x64 frame sizes from the existing compiler/linker PE unwind data."""
import bisect
import ctypes as c
import json
from pathlib import Path
import struct
import sys

class Frames:
    def __init__(self, path):
        self.data = Path(path).read_bytes()
        pe = self.u32(0x3c)
        assert self.data[pe:pe+4] == b'PE\0\0'
        count = self.u16(pe+6)
        optional = pe+24
        assert self.u16(optional) == 0x20b, 'x64 PE32+ only'
        self.base = struct.unpack_from('<Q', self.data, optional+24)[0]
        section = optional+self.u16(pe+20)
        self.sections = [struct.unpack_from('<IIII', self.data, section+40*i+8)
                         for i in range(count)]
        rva, size = struct.unpack_from('<II', self.data, optional+112+3*8)
        offset = self.offset(rva)
        self.functions = [struct.unpack_from('<III', self.data, offset+i)
                          for i in range(0, size, 12)]
        self.starts = [row[0] for row in self.functions]
    def u16(self, offset):
        return struct.unpack_from('<H', self.data, offset)[0]
    def u32(self, offset):
        return struct.unpack_from('<I', self.data, offset)[0]
    def offset(self, rva):
        for virtual_size, virtual, raw_size, raw in self.sections:
            if virtual <= rva < virtual+max(virtual_size, raw_size):
                return raw+rva-virtual
        raise ValueError(('RVA outside sections', hex(rva)))
    def unwind(self, rva, seen=()):
        assert rva not in seen, 'cyclic unwind chain'
        p = self.offset(rva)
        version_flags, _, count, _ = self.data[p:p+4]
        assert version_flags & 7 in (1, 2)
        flags = version_flags >> 3
        total, slot = 0, 0
        while slot < count:
            code = self.data[p+4+slot*2+1]
            operation, info = code & 15, code >> 4
            slot += 1
            if operation == 0:
                total += 8
            elif operation == 1:
                if info == 0:
                    total += self.u16(p+4+slot*2)*8
                    slot += 1
                else:
                    assert info == 1
                    total += self.u32(p+4+slot*2)
                    slot += 2
            elif operation == 2:
                total += info*8+8
            elif operation in (3, 6):
                pass
            elif operation in (4, 8):
                slot += 1
            elif operation in (5, 9):
                slot += 2
            elif operation == 10:
                total += 40+8*info
            else:
                raise ValueError(('unsupported unwind operation', operation))
        if flags & 4:
            chain = p+4+((count+1)//2)*4
            total += self.unwind(self.u32(chain+8), (*seen, rva))
        return total
    def frame(self, rva):
        index = bisect.bisect_right(self.starts, rva)-1
        if index < 0:
            return None
        begin, end, unwind = self.functions[index]
        if rva >= end:
            return None
        return self.unwind(unwind)

if __name__ == '__main__':
    root = Path(__file__).resolve().parents[2]
    metadata, binary, name = sys.argv[1:]
    values = json.loads(Path(metadata).read_text())['rust-binaries'].values()
    found = [v for v in values if v['binary-name'] == binary]
    assert len(found) == 1
    executable = Path(found[0]['binary-path'])
    pe = Frames(executable)
    k, d = c.WinDLL('kernel32'), c.WinDLL('dbghelp')
    k.GetCurrentProcess.restype = c.c_void_p
    process = k.GetCurrentProcess()
    d.SymInitializeW.argtypes = [c.c_void_p, c.c_wchar_p, c.c_int]
    d.SymLoadModuleExW.argtypes = [c.c_void_p, c.c_void_p, c.c_wchar_p, c.c_wchar_p,
                                 c.c_ulonglong, c.c_uint, c.c_void_p, c.c_uint]
    d.SymLoadModuleExW.restype = c.c_ulonglong
    d.SymSetOptions(0x2|0x10|0x200)
    assert d.SymInitializeW(process, str(executable.parent), False)
    loaded = d.SymLoadModuleExW(process, None, str(executable), None, pe.base, 0, None, 0)
    assert loaded
    callback = c.WINFUNCTYPE(c.c_int, c.c_void_p, c.c_uint, c.c_void_p)
    rows, errors = [], []
    @callback
    def collect(symbol, size, context):
        length = c.c_uint.from_address(symbol+76).value
        symbol_name = c.string_at(symbol+84, length).decode(errors='replace')
        rva = c.c_ulonglong.from_address(symbol+56).value-loaded
        try:
            frame = pe.frame(rva)
            if frame is not None:
                rows.append({'name': symbol_name, 'rva': rva, 'frame_bytes': frame})
        except (ValueError, AssertionError) as error:
            errors.append({'name': symbol_name, 'error': str(error)})
        return 1
    d.SymEnumSymbols.argtypes = [c.c_void_p, c.c_ulonglong, c.c_char_p, callback, c.c_void_p]
    assert d.SymEnumSymbols(process, loaded, b'*', collect, None)
    result = {'executable': str(executable), 'metadata': metadata,
              'source': 'compiler-emitted x64 PE unwind metadata; saved registers included; return address excluded',
              'frames': rows, 'decode_errors': errors}
    output = root / 'target/lane-evidence' / (name+'.json')
    output.write_text(json.dumps(result, indent=2), encoding='utf-8')
    print('Named compiler frames:', len(rows), 'decode errors:', len(errors), flush=True)
    for row in sorted(rows, key=lambda r: r['frame_bytes'], reverse=True)[:35]:
        print(row['frame_bytes'], row['name'], flush=True)
