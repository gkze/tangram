use ../../test.nu *

# A type-only re-export does not require a runtime package target.
let server = server spawn
let types = artifact {
	'tangram.packages.json': '{"conditions":[]}'
	'package.json': '{"type":"module","dependencies":{"types-only":"1"}}'
	'tangram.ts': 'export { type Only } from "types-only"; export default () => 42;'
	node_modules: {
		'types-only': {
			'package.json': '{"name":"types-only","exports":{"types":"./index.d.ts"}}'
			'index.d.ts': 'export interface Only { value: number; }'
		}
	}
}
let output = tg build $types
assert equal ($output | from json) 42
