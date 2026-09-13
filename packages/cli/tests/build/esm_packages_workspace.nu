use ../../test.nu *

# Package symlinks within the provisioned scope preserve installation ownership.
let server = server spawn
let root = artifact {
	'tangram.packages.json': '{"conditions":[]}'
	'package.json': '{"type":"module","dependencies":{"alpha":"1"}}'
	'tangram.ts': 'import value from "alpha"; export default () => value;'
	packages: {alpha: {
		'package.json': '{"name":"alpha","type":"module","exports":"./index.js"}'
		'index.js': 'export default 1;'
	}}
	node_modules: {alpha: {kind: symlink, path: '../packages/alpha'}}
}
assert equal ((tg build $root) | from json) 1
