use ../../test.nu *

export def fixture [] {
	artifact {
		'tangram.packages.json': '{"conditions":["tangram"]}'
		'package.json': '{"type":"module","dependencies":{"alpha":"1","beta":"1"}}'
		'tangram.ts': '
			import { value, identity } from "./z-helper.js";
			import { identity as subpathIdentity } from "alpha/identity";
			import legacy from "./legacy.tg.ts";
			export default async () => {
				const dynamic = await import("alpha/identity");
				return [value(), identity === subpathIdentity, identity === dynamic.identity, legacy];
			};
		'
		'legacy.tg.ts': 'export default "legacy";'
		'z-helper.js': 'export { value, identity } from "alpha";'
		node_modules: {
			alpha: {
				'package.json': '{"name":"alpha","type":"module","dependencies":{"beta":"1"},"exports":{".":{"types":"./index.d.ts","tangram":"./index.js","default":"./fallback.js"},"./identity":"./identity.js"},"imports":{"#identity":"./identity.js"}}'
				'index.js': '
					import { increment } from "beta";
					import { identity } from "#identity";
					export { identity };
					export const value = () => increment(41);
				'
				'unreachable.js': 'import fs from "node:fs"; export default fs;'
				'identity.js': 'export const identity = {};'
				'index.d.ts': 'export { identity } from "./identity.js"; export declare const value: () => number;'
				'identity.d.ts': 'export declare const identity: object;'
				'fallback.js': 'export { identity } from "#identity"; export const value = () => 43;'
			}
			beta: {
				'package.json': '{"name":"beta","type":"module","dependencies":{"alpha":"1"},"exports":"./index.js"}'
				'index.js': 'import { identity } from "alpha/identity"; export const increment = (n) => identity ? n + 1 : 0;'
			}
		}
	}
}
