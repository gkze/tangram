use {crate::Compiler, std::collections::BTreeMap, tangram_client::prelude::*, tangram_v8::Serde};

pub fn load(
	compiler: &Compiler,
	_scope: &mut v8::PinScope<'_, '_>,
	args: (Serde<tg::module::Data>,),
) -> tg::Result<String> {
	let (Serde(module),) = args;
	compiler.main_runtime_handle.clone().block_on(async move {
		let text = compiler.load_module(&module).await.map_err(
			|error| tg::error!(!error, module = ?module.without_token(), "failed to load the module"),
		)?;
		Ok(text)
	})
}

pub fn invalidated_resolutions(
	compiler: &Compiler,
	_scope: &mut v8::PinScope<'_, '_>,
	args: (Serde<tg::module::Data>,),
) -> tg::Result<bool> {
	let (Serde(module),) = args;
	compiler.main_runtime_handle.clone().block_on(async move {
		// Only path modules have lockfiles.
		let tg::module::data::Source::Path(module_path) = &module.referent.node else {
			return Ok(false);
		};

		// Revalidate package targets, including imports that previously failed to resolve.
		let resolutions = compiler
			.documents
			.get(&module)
			.map(|document| document.package_resolutions.clone())
			.unwrap_or_default();
		for (import, expected) in resolutions {
			let arg = tg::module::resolve::Arg {
				import,
				referrer: Some(module.clone()),
			};
			let current = compiler
				.handle
				.resolve_module(arg)
				.await
				.ok()
				.map(|output| output.module);
			if current != expected {
				return Ok(true);
			}
		}

		// Get or create the document.
		let document = compiler.documents.entry(module.clone()).or_default();

		// If the document doesn't have a lockfile, search for one.
		if document.lockfile.is_none() {
			let lockfile = compiler.find_lockfile_for_path(module_path).await;
			if lockfile.is_some() {
				// A lockfile exists but we haven't cached it yet. Resolutions are invalidated.
				return Ok(true);
			}
			// No lockfile exists.
			return Ok(false);
		}

		// The document has a lockfile cached. Check if the mtime has changed.
		let lockfile = document.lockfile.as_ref().unwrap();
		let lockfile_path = &lockfile.path;
		let cached_mtime = lockfile.mtime;

		// Get the current mtime.
		let current_mtime = tokio::fs::symlink_metadata(lockfile_path)
			.await
			.ok()
			.and_then(|metadata| metadata.modified().ok())
			.and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
			.map(|t| t.as_secs());

		// If the mtime could not be read, then assume no change.
		let Some(current_mtime) = current_mtime else {
			return Ok(false);
		};

		// If the mtime has changed, return true without updating the state.
		Ok(current_mtime != cached_mtime)
	})
}

pub fn resolve(
	compiler: &Compiler,
	_scope: &mut v8::PinScope<'_, '_>,
	args: (
		Serde<tg::module::Data>,
		String,
		Option<BTreeMap<String, String>>,
	),
) -> tg::Result<Serde<tg::module::Data>> {
	let (Serde(referrer), specifier, attributes) = args;
	let mut import = tg::module::Import::with_specifier_and_attributes(&specifier, attributes)
		.map_err(|error| tg::error!(!error, "failed to create the import"))?;
	import.types = true;
	compiler.main_runtime_handle.clone().block_on(async move {
		let arg = tg::module::resolve::Arg {
			import: import.clone(),
			referrer: Some(referrer.clone()),
		};
		let result = compiler.handle.resolve_module(arg).await;
		if let tg::module::data::Source::Path(path) = &referrer.referent.node
			&& path.is_absolute()
			&& tg::module::package::Import::is_package(&import)
			&& tg::module::package::Scope::try_for_path(path)?.is_some()
		{
			let mut document = compiler.documents.entry(referrer.clone()).or_default();
			document.package_resolutions.insert(
				import.clone(),
				result.as_ref().ok().map(|output| output.module.clone()),
			);
		}
		let output = result.map_err(|error| {
			tg::error!(
				source = error,
				import = ?import.without_token(),
				referrer = ?referrer.without_token(),
				"failed to resolve specifier relative to the module"
			)
		})?;
		Ok(Serde(output.module))
	})
}

pub fn validate_resolutions(
	compiler: &Compiler,
	_scope: &mut v8::PinScope<'_, '_>,
	args: (Serde<tg::module::Data>,),
) -> tg::Result<()> {
	let (Serde(module),) = args;
	compiler.main_runtime_handle.clone().block_on(async move {
		// Only path modules have lockfiles.
		let tg::module::data::Source::Path(path) = &module.referent.node else {
			return Ok(());
		};

		// Find the current lockfile.
		let lockfile = compiler.find_lockfile_for_path(path).await;

		// Update the document's lockfile state.
		if let Some(mut document) = compiler.documents.get_mut(&module) {
			document.lockfile = lockfile;
		}

		Ok(())
	})
}

pub fn version(
	compiler: &Compiler,
	_scope: &mut v8::PinScope<'_, '_>,
	args: (Serde<tg::module::Data>,),
) -> tg::Result<String> {
	let (Serde(module),) = args;
	compiler.main_runtime_handle.clone().block_on(async move {
		let version = compiler
			.get_module_version(&module)
			.await
			.map_err(|error| tg::error!(!error, "failed to get the module version"))?;
		Ok(version.to_string())
	})
}
