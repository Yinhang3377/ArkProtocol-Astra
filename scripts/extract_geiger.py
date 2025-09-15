import json
from pathlib import Path
p = Path('target') / 'geiger' / 'geiger-full.json'
if not p.exists():
    print('MISSING', p)
    raise SystemExit(2)
try:
    d = json.loads(p.read_text(encoding='utf-8'))
except Exception as e:
    print('LOADERROR', e)
    raise
names = ['getrandom','signal-hook-registry','secp256k1','backtrace','bytes','smallvec']
found = {}
for pkg in d.get('packages',[]):
    name = pkg.get('package',{}).get('id',{}).get('name')
    if name in names:
        u = pkg.get('unsafety',{})
        found[name] = u

for n in names:
    print('---')
    print(n)
    if n in found:
        u = found[n]
        used = u.get('used',{})
        forbids = u.get('forbids_unsafe')
        print('forbids_unsafe:', forbids)
        for k,v in used.items():
            print(k,':',v)
    else:
        print('NOT FOUND')
