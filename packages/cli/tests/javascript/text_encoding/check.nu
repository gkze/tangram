use ../../lib/test.nu *

# The runtime declarations type the TextEncoder and TextDecoder globals without the DOM library.

let local = server spawn

let path = artifact {
	tangram.ts: '
		export default function () {
			let encoder = new TextEncoder();
			let bytes: Uint8Array = encoder.encode("hello");
			let result: TextEncoderEncodeIntoResult = encoder.encodeInto("hi", bytes);
			let decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: false });
			let text: string = decoder.decode(new DataView(bytes.buffer), { stream: false });
			return [text, result.read, decoder.encoding, decoder.fatal, decoder.ignoreBOM];
		}
	'
}

let output = do { cd $path; tg check . } | complete
success $output
