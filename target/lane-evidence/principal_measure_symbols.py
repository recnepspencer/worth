"""Read only the built image/PDB; no test body or product mutation."""
import ctypes as c,ctypes.wintypes as w,json,sys
from pathlib import Path
k=c.WinDLL('kernel32');d=c.WinDLL('dbghelp',use_last_error=True);P=c.c_void_p;k.GetCurrentProcess.restype=P;process=k.GetCurrentProcess()
d.SymSetOptions.argtypes=[w.DWORD];d.SymInitializeW.argtypes=[P,w.LPCWSTR,w.BOOL];d.SymLoadModuleExW.argtypes=[P,P,w.LPCWSTR,w.LPCWSTR,c.c_ulonglong,w.DWORD,P,w.DWORD];d.SymLoadModuleExW.restype=c.c_ulonglong
class Line(c.Structure):_fields_=[('size',w.DWORD),('key',P),('number',w.DWORD),('file',c.c_char_p),('address',c.c_ulonglong)]
d.SymGetLineFromAddr64.argtypes=[P,c.c_ulonglong,c.POINTER(w.DWORD),c.POINTER(Line)]
callbacktype=c.WINFUNCTYPE(w.BOOL,P,w.ULONG,P);d.SymEnumSymbols.argtypes=[P,c.c_ulonglong,c.c_char_p,callbacktype,P]
meta=json.loads(Path(sys.argv[1]).read_text());exe=next(v['binary-path'] for v in meta['rust-binaries'].values())
d.SymSetOptions(0x10|0x200|0x4|0x2);assert d.SymInitializeW(process,str(Path(exe).parent),False)
base=d.SymLoadModuleExW(process,None,exe,None,0x100000000,0,None,0);assert base,c.get_last_error();print('BASE',hex(base),flush=True)

symbols=[]
@callbacktype
def collect(raw,size,user):
 import struct
 b=c.string_at(raw,84);length=struct.unpack_from('<I',b,76)[0];name=c.string_at(raw+84,length).decode(errors='replace');address=struct.unpack_from('<Q',b,56)[0];length=struct.unpack_from('<I',b,28)[0]
 symbols.append({'name':name,'rva':address-base,'size':length})
 return True
for mask in [b'*KernelContext::checkpoint',b'*region_load::load_full',b'*materialization_funding::work',b'*required_symbols*',b'*BTreeSet*insert*Symbol*',b'*BTreeSet*Symbol*insert*']:
 assert d.SymEnumSymbols(process,base,mask,collect,None),c.get_last_error()
unique={x['rva']:x for x in symbols};symbols=list(unique.values())
Path('target/lane-evidence/principal-measure-symbols.json').write_text(json.dumps(symbols,indent=2),encoding='utf-8')
for s in symbols:
 if s['name'].startswith(('worth_execution::backend::meter::KernelContext::checkpoint','worth_relational::storage::residency')) or ('BTreeSet' in s['name'] and 'Symbol' in s['name'] and 'insert' in s['name']): print(s)
