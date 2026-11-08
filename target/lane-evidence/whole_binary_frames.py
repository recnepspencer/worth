"""Inventory all large PE unwind functions, with exact-address PDB attribution."""
import ctypes as c,ctypes.wintypes as w,json,sys,gc,hashlib,re,collections
from pathlib import Path
e=Path(__file__).resolve().parent
sys.path.insert(0,str(e));from lifecycle_compiled_frames import Frames
sys.path.insert(0,str(e/'rust_demangler-1.0-py3-none-any.whl'))
from rust_demangler import demangle
from rust_demangler.rust_v0 import Printer,Parser
def lifetime(self,lt):
    if lt==0:self.out+="'_";return
    depth=self.bound_lifetime_depth-lt
    if depth<0:raise ValueError('invalid lifetime index')
    self.out+="'"+(chr(ord('a')+depth) if depth<26 else '_'+str(depth))
Printer.print_lifetime_from_index=lifetime
def rust_linkage(name):
    return name.startswith(('_R','_ZN')) or bool(re.match(r'^R[INMXYNC]',name))
def defining_crate(name,display):
    if name.startswith('_R') or re.match(r'^R[INMXYNC]',name):
        parser=Parser(name[name.index('R')+1:],0)
        def owner(p):
            tag=p.next_func()
            if tag=='C':p.disambiguator();ident=p.ident();ident.display();return ident.disp
            if tag=='N':p.namespace();return owner(p)
            if tag in ('M','X'):p.disambiguator();return owner(p)
            if tag=='I':return owner(p)
            if tag=='B':return owner(p.backref())
            if tag=='Y':p.skip_type();return owner(p)
            raise ValueError(('unhandled defining path',tag,name))
        return owner(parser)
    prefix=display.lstrip('<').split('::')[0]
    if prefix.startswith(('worth_','bank','invariant_reads','shared_execution','required_feature')):return prefix
    trait=re.search(r' as ((?:worth_|bank)[a-zA-Z0-9_]+)::',display)
    return trait[1] if trait else prefix
k=c.WinDLL('kernel32',use_last_error=True);d=c.WinDLL('dbghelp',use_last_error=True)
k.GetCurrentProcess.restype=c.c_void_p
d.SymInitializeW.argtypes=[c.c_void_p,w.LPCWSTR,w.BOOL]
d.SymLoadModuleExW.argtypes=[c.c_void_p,c.c_void_p,w.LPCWSTR,w.LPCWSTR,c.c_ulonglong,w.DWORD,c.c_void_p,w.DWORD];d.SymLoadModuleExW.restype=c.c_ulonglong
d.SymCleanup.argtypes=[c.c_void_p]
callback=c.WINFUNCTYPE(c.c_int,c.c_void_p,c.c_uint,c.c_void_p)
d.SymEnumSymbols.argtypes=[c.c_void_p,c.c_ulonglong,c.c_char_p,callback,c.c_void_p]

def canonical(name):
    # Legacy MSVC names are already demangled by DbgHelp. Rust v0 needs its own parser.
    original=name
    if re.match(r'^R[INMXYNC]',name):name='_'+name
    if name.startswith(('_R','_ZN')):
        try:name=demangle(name)
        except Exception as error:return None,str(error)
    # Remove generic arguments only, retaining all namespace/impl/closure components.
    def strip_generics(value):
        result=[];i=0
        while i<len(value):
            if value[i]!='<':result.append(value[i]);i+=1;continue
            start=i;depth=1;i+=1
            while i<len(value) and depth:
                if value[i]=='<':depth+=1
                elif value[i]=='>' and value[i-1]!='-':depth-=1
                i+=1
            qualifier=start==0 or value[start-1] in ' (,&'
            if qualifier:result.append('<'+strip_generics(value[start+1:i-1])+'>')
        return ''.join(result).replace('::::','::')
    value=strip_generics(name)
    if value.startswith('<'):
        depth=0
        for i,char in enumerate(value):
            depth+=(char=='<')-(char=='>' and value[i-1]!='-')
            if depth==0:
                if ' as ' not in value[1:i]:value=value[1:i]+value[i+1:]
                break
    value=re.sub(r'::impl\$\d+','',value)
    value=re.sub(r'::h[0-9a-f]{16}$','',value)
    # V0's [crate disambiguator] does not belong to semantic symbol identity.
    value=re.sub(r'\[[0-9a-f]+\]','',value)
    value=value.rstrip(':')
    value=re.sub(r'::\{closure(?::)?#(\d+)\}',r'::closure$\1',value)
    value=re.sub(r'<([^<>]+)>',lambda m:m.group(0) if ' as ' in m.group(1) else m.group(1),value)
    return value,None

def scan(label,exe):
    exe=Path(exe);pe=Frames(str(exe));large={};errors=[]
    for begin,end,unwind in pe.functions:
        try:size=pe.unwind(unwind)
        except Exception as error:errors.append({'rva':begin,'error':str(error)});continue
        if size>=32768:large[begin]={'rva':begin,'frame_bytes':size,'names':[]}
    process=k.GetCurrentProcess();d.SymSetOptions(0x2|0x10|0x200)
    assert d.SymInitializeW(process,str(exe.parent),False)
    loaded=d.SymLoadModuleExW(process,None,str(exe),None,pe.base,0,None,0);assert loaded
    @callback
    def collect(symbol,size,context):
        address=c.c_ulonglong.from_address(symbol+56).value-loaded
        if address in large:
            length=c.c_uint.from_address(symbol+76).value
            raw=c.string_at(symbol+84,length).decode(errors='replace')
            if raw not in large[address]['names']:large[address]['names'].append(raw)
        return 1
    assert d.SymEnumSymbols(process,loaded,b'*',collect,None)
    assert d.SymCleanup(process)
    result={'binary':label,'exe':str(exe),'size':exe.stat().st_size,
      'mtime_ns':exe.stat().st_mtime_ns,'unwind_function_count':len(pe.functions),
      'large_functions':list(large.values()),'unwind_errors':errors}
    print(label,'functions',len(pe.functions),'large',len(large),'unresolved',sum(not v['names'] for v in large.values()),'errors',len(errors),flush=True)
    del pe;gc.collect();return result

def main():
    manifest,out=sys.argv[1:];inputs=json.loads(Path(manifest).read_text());binaries=[]
    for label,path in inputs.items():binaries.append(scan(label,path))
    by_path={};failures=[];unresolved=[]
    for binary in binaries:
        for row in binary['large_functions']:
            if not row['names']:unresolved.append({'binary':binary['binary'],**row})
            for symbol in row['names']:
                name,error=canonical(symbol)
                if error:failures.append({'symbol':symbol,'error':error,'binary':binary['binary']});continue
                if not any(re.search(r'(^|::|<)(worth_|bank|invariant_reads|shared_execution|required_feature)',name) for _ in [0]):continue
                key=name
                item=by_path.setdefault(key,{'symbol_path':key,'max_frame':0,'binaries':[], 'symbols':[]})
                item['max_frame']=max(item['max_frame'],row['frame_bytes'])
                if binary['binary'] not in item['binaries']:item['binaries'].append(binary['binary'])
                item['symbols'].append({'binary':binary['binary'],'rva':row['rva'],'bytes':row['frame_bytes'],'name':symbol})
    rows=sorted(by_path.values(),key=lambda r:(-r['max_frame'],r['symbol_path']))
    result={'source':'all compiler PE unwind entries, saved registers included; return address excluded; exact-address PDB symbols only',
      'binaries':binaries,'functions':rows,'demangle_failures':failures,'unresolved':unresolved,
      'bands':{'>=128KiB':sum(r['max_frame']>=131072 for r in rows),'64-128KiB':sum(65536<=r['max_frame']<131072 for r in rows),'32-64KiB':sum(32768<=r['max_frame']<65536 for r in rows)}}
    Path(out).write_text(json.dumps(result,indent=2),encoding='utf-8')
    print('DEDUPLICATED',len(rows),result['bands'],'demangle failures',len(failures),'unresolved',len(unresolved),flush=True)
if __name__=='__main__':main()
