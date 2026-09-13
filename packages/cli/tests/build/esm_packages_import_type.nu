use ../../test.nu *

# Import-type expressions in external declarations produce normal type diagnostics.
let server = server spawn
let root = artifact {
	'tangram.packages.json': '{"conditions":[]}'
	'package.json': '{"type":"module","dependencies":{"alpha":"1"}}'
	'tangram.ts': 'import type { Thing } from "alpha"; const thing: Thing = {value:"wrong"}; export default () => thing;'
	node_modules: {alpha: {
		'package.json': '{"name":"alpha","type":"module","exports":{"types":"./index.d.ts"}}'
		'index.d.ts': 'export type Thing = import("./other.js").Thing;'
		'other.d.ts': 'export interface Thing {value:number;}'
	}}
}
let output = tg check $root | complete
failure $output
snapshot --normalize --redact $root $output.stderr r#'
	error Type 'string' is not assignable to type 'number'.
	   ╭─[<redacted>/tangram.ts:1:59]
	 1 │ import type { Thing } from "alpha"; const thing: Thing = {value:"wrong"}; export default () => thing;
	   ·                                                           ──┬──
	   ·                                                             ╰── Type 'string' is not assignable to type 'number'.
	   ╰────
	error an error occurred
	-> type checking failed

'#
