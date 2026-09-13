use ../../test.nu *

# Nested module metadata preserves the installation dependency declarations.
let server = server spawn
let inner = artifact {
	'tangram.packages.json': '{"conditions":[]}'
	'package.json': '{"type":"module","dependencies":{"alpha":"1"}}'
	'tangram.ts': 'import value from "alpha"; export default () => value;'
	node_modules: {alpha: {
		'package.json': '{"name":"alpha","exports":"./dist/index.js","dependencies":{"beta":"1"}}'
		dist: {'package.json':'{"type":"module"}', 'index.js':'import value from "beta"; export default value;'}
	}, beta: {'package.json':'{"name":"beta","type":"module","exports":"./index.js"}', 'index.js':'export default 3;'}}
}
assert equal ((tg build $inner) | from json) 3
