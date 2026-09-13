use ../../test.nu *
use ../lib/esm.nu

# Filesystem package imports execute through both evaluator engines.
let server = server spawn
let path = esm fixture
let entry = $path | path join 'filesystem.tg.ts'
'import run from "./tangram.ts"; export default async () => { tg.assert(JSON.stringify(await run()) === JSON.stringify([42, true, true, "legacy"])); };' | save $entry
let module = 'tg.module(' + ({kind: ts, referent: {node: $entry}} | to json --raw) + ')'
tg js --engine v8 --export default $module
tg js --engine quickjs --export default $module
