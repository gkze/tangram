use ../../lib/test.nu *

# A fatal TextDecoder throws a TypeError for malformed input and decodes the next call from a reset state.

let local = server spawn

let path = artifact {
	tangram.ts: '
		export default function () {
			let decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
			decoder.decode(new Uint8Array([0xe2]), { stream: true });
			let error;
			try {
				decoder.decode();
			} catch (caught) {
				error = caught instanceof TypeError;
			}
			return { error, next: decoder.decode(new Uint8Array([0xef, 0xbb, 0xbf, 0x41])) };
		}
	'
}

let output = tg build $path | from json
assert equal $output { error: true, next: "\u{feff}A" }
