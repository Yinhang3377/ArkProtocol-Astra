import json

GFILE = r"C:\Users\plant\Desktop\Rust区块链\ArkProtocol-Astra\target\geiger\geiger-full.json"
OUT = r"C:\Users\plant\Desktop\extract_debug.txt"
TARGETS = ['getrandom','signal-hook-registry','secp256k1','backtrace','bytes','smallvec']

try:
    with open(GFILE,'r',encoding='utf-8') as f:
        data = json.load(f)
except Exception as e:
    with open(OUT,'w',encoding='utf-8') as f:
        f.write('FAILED TO OPEN GFILE: ' + str(e))
    raise

index = {}
for p in data.get('packages',[]):
    name = p.get('package',{}).get('id',{}).get('name')
    if not name:
        continue
    index.setdefault(name,[]).append(p)

lines = []
for t in TARGETS:
    if t not in index:
        lines.append(f'{t}: MISSING')
        continue
    for ent in index[t]:
        v = ent.get('package',{}).get('id',{}).get('version')
        used_funcs = ent.get('unsafety',{}).get('used',{}).get('functions',{})
        used_exprs = ent.get('unsafety',{}).get('used',{}).get('exprs',{})
        forbids = ent.get('unsafety',{}).get('forbids_unsafe')
        lines.append(f'{t} {v} forbids_unsafe={forbids} used.funcs={used_funcs} used.exprs={used_exprs}')

with open(OUT,'w',encoding='utf-8') as f:
    f.write('\n'.join(lines))

print('WROTE DEBUG TO ' + OUT)
