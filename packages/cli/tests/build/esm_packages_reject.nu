use ../../test.nu *

# Computed imports are rejected before capturing an incomplete package graph.
let server = server spawn
let computed = artifact {
	'tangram.packages.json': '{"conditions":[]}'
	'package.json': '{"type":"module"}'
	'tangram.ts': 'const target = "./ordinary.js"; export default () => import(target);'
	'ordinary.js': 'export default 42;'
}
let output = tg checkin $computed | complete
failure $output
snapshot --normalize --redact $computed $output.stderr '
	error an error occurred
	-> failed to analyze a package module
	   diagnostic = The argument to the import function must be a string literal.

'
