use ../../lib/test.nu *

# TextDecoder supports only UTF-8 and rejects the labels of other encodings with a RangeError.

let local = server spawn

let path = artifact {
	tangram.ts: '
		export default function () {
			return ["latin1", "utf-16le", "shift_jis", "replacement", " utf8"].map((label) => {
				try {
					new TextDecoder(label);
					return "supported";
				} catch (error) {
					return error instanceof RangeError ? "RangeError" : "other";
				}
			});
		}
	'
}

let output = tg build $path | from json
assert equal $output ["RangeError" "RangeError" "RangeError" "RangeError" "RangeError"]
