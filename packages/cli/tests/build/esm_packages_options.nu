use ../../test.nu *

# An explicit source attribute retains its target alongside a package import of the same name.
let server = server spawn
let root = artifact {
	'tangram.packages.json': '{"conditions":[]}'
	'package.json': '{"type":"module","dependencies":{"alpha":"1"}}'
	'tangram.ts': 'import ordinary from "alpha"; import overridden from "alpha" with { source: "./override.tg.ts" }; export default () => [ordinary, overridden];'
	'override.tg.ts': 'export default 2;'
	node_modules: {alpha: {
		'package.json': '{"name":"alpha","type":"module","exports":"./index.js"}'
		'index.js': 'export default 1;'
	}}
}
assert equal ((tg build $root) | from json) [1 2]
