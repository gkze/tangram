use {
	crate::prelude::*,
	oxc_resolver::{FileMetadata, FileSystem, ResolveError, ResolveOptions, ResolverGeneric},
	std::{
		collections::{BTreeMap, BTreeSet},
		io,
		path::{Path, PathBuf},
	},
};

pub const FILE_NAME: &str = "tangram.packages.json";

#[derive(Clone, Debug)]
pub struct Scope {
	conditions: Vec<String>,
	root: PathBuf,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Import {
	pub specifier: String,
	pub types: bool,
}

#[derive(Debug)]
pub struct Output {
	pub metadata: BTreeSet<PathBuf>,
	pub path: PathBuf,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
	conditions: Vec<String>,
}

#[derive(Default, serde::Deserialize)]
struct Package {
	#[serde(default)]
	dependencies: BTreeMap<String, String>,
	#[serde(rename = "type")]
	kind: Option<String>,
	name: Option<String>,
	#[serde(default, rename = "optionalDependencies")]
	optional_dependencies: BTreeMap<String, String>,
	#[serde(default, rename = "peerDependencies")]
	peer_dependencies: BTreeMap<String, String>,
}

struct Fs {
	root: PathBuf,
}

impl Scope {
	pub fn try_for_path(path: &Path) -> tg::Result<Option<Self>> {
		for directory in path.parent().into_iter().flat_map(Path::ancestors) {
			let config = directory.join(FILE_NAME);
			let contents = std::fs::symlink_metadata(&config).and_then(|_| {
				let root = directory.canonicalize()?;
				let fs = Fs { root };
				fs.read(&config)
			});
			match contents {
				Ok(bytes) => {
					let config: Config = serde_json::from_slice(&bytes)
						.map_err(|error| tg::error!(!error, "invalid package scope"))?;
					if config
						.conditions
						.iter()
						.any(|condition| matches!(condition.as_str(), "types" | "require"))
					{
						return Err(tg::error!(
							"runtime conditions cannot include types or require"
						));
					}
					let root = directory.canonicalize().map_err(|error| {
						tg::error!(!error, "failed to canonicalize the package scope")
					})?;
					return Ok(Some(Self {
						conditions: config.conditions,
						root,
					}));
				},
				Err(error) if error.kind() == io::ErrorKind::NotFound => (),
				Err(error) => return Err(tg::error!(!error, "failed to read the package scope")),
			}
			if tg::module::ROOT_MODULE_FILE_NAMES
				.iter()
				.any(|name| directory.join(name).is_file())
			{
				break;
			}
		}
		Ok(None)
	}

	pub fn canonicalize(&self, path: &Path) -> tg::Result<PathBuf> {
		let fs = Fs {
			root: self.root.clone(),
		};
		fs.check(path).map_err(|error| {
			tg::error!(
				!error,
				"the module path is outside its package scope or unavailable"
			)
		})
	}

	pub fn metadata(&self, path: &Path) -> tg::Result<BTreeSet<PathBuf>> {
		let (manifest, _) = self.package(path)?;
		let (owner, _) = self.owner(path)?;
		Ok(BTreeSet::from([self.root.join(FILE_NAME), manifest, owner]))
	}

	pub fn kind(&self, path: &Path) -> tg::Result<Option<tg::module::Kind>> {
		if path
			.file_name()
			.and_then(|name| name.to_str())
			.is_some_and(|name| name.ends_with(".d.ts") || name.ends_with(".d.mts"))
		{
			return Ok(Some(tg::module::Kind::Dts));
		}
		match path.extension().and_then(|extension| extension.to_str()) {
			Some("mjs") => Ok(Some(tg::module::Kind::Js)),
			Some("js") => {
				let (_, package) = self.package(path)?;
				Ok((package.kind.as_deref() == Some("module")).then_some(tg::module::Kind::Js))
			},
			_ => Ok(None),
		}
	}

	pub fn resolve(
		&self,
		referrer: &Path,
		import: &tg::module::Import,
	) -> tg::Result<Option<Output>> {
		if !Import::is_package(import) {
			return Ok(None);
		}
		let specifier = &import.specifier;
		if tg::module::is_module_path(referrer) && tg::module::is_module_path(Path::new(specifier))
		{
			return Ok(None);
		}
		if specifier.starts_with('/')
			|| specifier.contains(':')
			|| specifier.contains('?')
			|| specifier.contains('\\')
			|| (!specifier.starts_with('#') && specifier.contains('#'))
		{
			return Err(tg::error!(%specifier, "unsupported package module specifier"));
		}
		let (manifest, package) = self.owner(referrer)?;
		if !specifier.starts_with('.') && !specifier.starts_with('#') {
			let name = package_name(specifier)?;
			if package.name.as_deref() != Some(name) && !declares(&package, name) {
				if tg::module::is_module_path(referrer) && import.reference.node().is_specifier() {
					return Ok(None);
				}
				return Err(tg::error!(%specifier, "the package dependency is not declared"));
			}
		}

		// Delegate package maps to Oxc while bounding every filesystem operation.
		let mut condition_names = self.conditions.clone();
		condition_names.push("import".into());
		if import.types {
			condition_names.push("types".into());
		}
		let options = ResolveOptions {
			builtin_modules: true,
			condition_names,
			extension_alias: if import.types {
				vec![
					(".js".into(), vec![".d.ts".into(), ".js".into()]),
					(".mjs".into(), vec![".d.mts".into(), ".mjs".into()]),
					(
						".mts".into(),
						vec![".mts".into(), ".d.mts".into(), ".mjs".into()],
					),
					(
						".ts".into(),
						vec![".ts".into(), ".d.ts".into(), ".js".into()],
					),
				]
			} else {
				Vec::new()
			},
			extensions: Vec::new(),
			fully_specified: true,
			main_fields: if import.types {
				vec!["types".into(), "typings".into(), "main".into()]
			} else {
				vec!["main".into()]
			},
			main_files: Vec::new(),
			..Default::default()
		};
		let fs = Fs {
			root: self.root.clone(),
		};
		let resolver = ResolverGeneric::new_with_file_system(fs, options);
		let mut context = oxc_resolver::ResolveContext::default();
		let resolution = resolver
			.resolve_with_context(referrer.parent().unwrap(), specifier, None, &mut context)
			.map_err(
				|error| tg::error!(!error, %specifier, "failed to resolve the package module"),
			)?;
		let path = resolution.path().to_owned();
		let (target_manifest, target_package) = self.owner(&path)?;
		if target_manifest != manifest && (specifier.starts_with('.') || specifier.starts_with('#'))
		{
			let name = target_package
				.name
				.as_deref()
				.ok_or_else(|| tg::error!("the target package has no name"))?;
			if !declares(&package, name) {
				return Err(tg::error!("the target package dependency is not declared"));
			}
		}
		let kind = self.kind(&path)?;
		if kind != Some(tg::module::Kind::Js)
			&& !(import.types && kind == Some(tg::module::Kind::Dts))
		{
			return Err(
				tg::error!(path = %path.display(), "the package target is not a supported ESM module"),
			);
		}
		let mut metadata: BTreeSet<_> = context
			.file_dependencies
			.into_iter()
			.filter(|path| path.file_name().is_some_and(|name| name == "package.json"))
			.collect();
		metadata.extend([self.root.join(FILE_NAME), manifest, target_manifest]);
		metadata.extend(self.metadata(&path)?);
		metadata.extend(self.metadata(referrer)?);
		Ok(Some(Output { metadata, path }))
	}

	fn owner(&self, path: &Path) -> tg::Result<(PathBuf, Package)> {
		// Nested package metadata controls module interpretation, not installation ownership.
		for directory in path
			.parent()
			.into_iter()
			.flat_map(Path::ancestors)
			.take_while(|path| path.starts_with(&self.root))
		{
			let manifest = directory.join("package.json");
			let fs = Fs {
				root: self.root.clone(),
			};
			let bytes = match fs.read(&manifest) {
				Ok(bytes) => bytes,
				Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
				Err(error) => return Err(tg::error!(!error, "failed to read the package owner")),
			};
			let package: Package = serde_json::from_slice(&bytes)
				.map_err(|error| tg::error!(!error, "invalid package metadata"))?;
			let installed = directory.parent().is_some_and(|parent| {
				parent
					.file_name()
					.is_some_and(|name| name == "node_modules")
					|| (parent
						.file_name()
						.is_some_and(|name| name.to_string_lossy().starts_with('@'))
						&& parent.parent().is_some_and(|parent| {
							parent
								.file_name()
								.is_some_and(|name| name == "node_modules")
						}))
			});
			if directory == self.root || installed || package.name.is_some() {
				return Ok((manifest, package));
			}
		}
		Err(tg::error!(
			"the module requires a package owner within its scope"
		))
	}

	fn package(&self, path: &Path) -> tg::Result<(PathBuf, Package)> {
		let fs = Fs {
			root: self.root.clone(),
		};
		for directory in path
			.parent()
			.into_iter()
			.flat_map(Path::ancestors)
			.take_while(|path| path.starts_with(&self.root))
		{
			let path = directory.join("package.json");
			match fs.read(&path) {
				Ok(bytes) => {
					let package = serde_json::from_slice(&bytes)
						.map_err(|error| tg::error!(!error, "invalid package metadata"))?;
					return Ok((path, package));
				},
				Err(error) if error.kind() == io::ErrorKind::NotFound => (),
				Err(error) => {
					return Err(tg::error!(!error, "failed to read the package metadata"));
				},
			}
		}
		Err(tg::error!(
			"the module requires package.json within its scope"
		))
	}
}

impl Import {
	#[must_use]
	pub fn is_package(import: &tg::module::Import) -> bool {
		import.kind.is_none()
			&& import.reference.options() == &tg::reference::Options::default()
			&& !matches!(
				import.reference.node(),
				tg::reference::Node::Id(_) | tg::reference::Node::Pointer(_)
			)
	}

	#[must_use]
	pub fn reference(import: &tg::module::Import) -> tg::Reference {
		let import = Self {
			specifier: import.specifier.clone(),
			types: import.types,
		};
		tg::Reference::with_node(tg::reference::Node::Module(import))
	}
}

impl Fs {
	fn check(&self, path: &Path) -> io::Result<PathBuf> {
		if !path.starts_with(&self.root) {
			return Err(io::ErrorKind::NotFound.into());
		}
		let path = path.canonicalize()?;
		if !path.starts_with(&self.root) {
			return Err(io::Error::new(
				io::ErrorKind::PermissionDenied,
				"the path escapes the package scope",
			));
		}
		Ok(path)
	}
}

impl FileSystem for Fs {
	fn new() -> Self {
		panic!("a package filesystem requires an explicit scope")
	}
	fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
		std::fs::read(self.check(path)?)
	}
	fn read_to_string(&self, path: &Path) -> io::Result<String> {
		std::fs::read_to_string(self.check(path)?)
	}
	fn metadata(&self, path: &Path) -> io::Result<FileMetadata> {
		std::fs::metadata(self.check(path)?).map(Into::into)
	}
	fn symlink_metadata(&self, path: &Path) -> io::Result<FileMetadata> {
		self.check(path)?;
		std::fs::symlink_metadata(path).map(Into::into)
	}
	fn read_link(&self, path: &Path) -> Result<PathBuf, ResolveError> {
		self.check(path)?;
		std::fs::read_link(path).map_err(Into::into)
	}
	fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
		self.check(path)
	}
}

fn declares(package: &Package, name: &str) -> bool {
	package.dependencies.contains_key(name)
		|| package.optional_dependencies.contains_key(name)
		|| package.peer_dependencies.contains_key(name)
}

fn package_name(specifier: &str) -> tg::Result<&str> {
	let mut parts = specifier.split('/');
	let first = parts.next().unwrap_or_default();
	if first.is_empty() || first == "." || first == ".." {
		return Err(tg::error!("invalid package name"));
	}
	if first.starts_with('@') {
		let second = parts
			.next()
			.filter(|part| !part.is_empty())
			.ok_or_else(|| tg::error!("invalid scoped package name"))?;
		Ok(&specifier[..first.len() + 1 + second.len()])
	} else {
		Ok(first)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn fixture() -> tempfile::TempDir {
		let directory = tempfile::tempdir().unwrap();
		write(directory.path(), FILE_NAME, r#"{"conditions":["tangram"]}"#);
		write(directory.path(), "tangram.ts", "");
		write(
			directory.path(),
			"package.json",
			r#"{"type":"module","dependencies":{"alpha":"1","beta":"1"}}"#,
		);
		write(
			directory.path(),
			"node_modules/alpha/package.json",
			r##"{"name":"alpha","type":"module","dependencies":{"beta":"1"},"exports":{".":{"types":"./index.d.ts","tangram":"./index.js","default":"./fallback.js"},"./feature/*":"./lib/*.js","./private":null},"imports":{"#internal":"./lib/internal.js","#dependency":"beta"}}"##,
		);
		for path in ["index.js", "fallback.js", "index.d.ts", "lib/internal.js"] {
			write(directory.path(), &format!("node_modules/alpha/{path}"), "");
		}
		write(
			directory.path(),
			"node_modules/beta/package.json",
			r#"{"name":"beta","type":"module","exports":"./index.js"}"#,
		);
		write(directory.path(), "node_modules/beta/index.js", "");
		directory
	}

	fn write(root: &Path, path: &str, text: &str) {
		let path = root.join(path);
		std::fs::create_dir_all(path.parent().unwrap()).unwrap();
		std::fs::write(path, text).unwrap();
	}

	fn resolve(root: &Path, referrer: &str, specifier: &str, types: bool) -> tg::Result<Output> {
		let path = root.join(referrer).canonicalize().unwrap();
		let scope = Scope::try_for_path(&path)?.unwrap();
		let mut import = tg::module::Import::with_specifier_and_attributes(specifier, None)?;
		import.types = types;
		scope
			.resolve(&path, &import)?
			.ok_or_else(|| tg::error!("expected a package import"))
	}

	#[test]
	fn typescript_declaration_extensions() {
		let directory = fixture();
		let root = directory.path().canonicalize().unwrap();
		for (extension, target) in [("ts", "d.ts"), ("mts", "d.mts")] {
			let path = format!("node_modules/alpha/value.{target}");
			write(&root, &path, "export declare const value: number;");
			let specifier = format!("./value.{extension}");
			let referrer = "node_modules/alpha/index.d.ts";
			assert_eq!(
				resolve(&root, referrer, &specifier, true).unwrap().path,
				root.join(&path)
			);
			assert!(resolve(&root, referrer, &specifier, false).is_err());
		}
		for extension in ["cts", "cjs", "d.cts"] {
			write(&root, &format!("node_modules/alpha/value.{extension}"), "");
			for types in [false, true] {
				assert!(
					resolve(
						&root,
						"node_modules/alpha/index.d.ts",
						&format!("./value.{extension}"),
						types
					)
					.is_err()
				);
			}
		}
	}

	#[test]
	fn package_maps_and_conditions() {
		let directory = fixture();
		let root = directory.path().canonicalize().unwrap();
		for (referrer, specifier, types, expected) in [
			("tangram.ts", "alpha", false, "node_modules/alpha/index.js"),
			("tangram.ts", "alpha", true, "node_modules/alpha/index.d.ts"),
			(
				"tangram.ts",
				"alpha/feature/internal",
				false,
				"node_modules/alpha/lib/internal.js",
			),
			(
				"node_modules/alpha/index.js",
				"alpha",
				false,
				"node_modules/alpha/index.js",
			),
			(
				"node_modules/alpha/index.js",
				"#internal",
				false,
				"node_modules/alpha/lib/internal.js",
			),
			(
				"node_modules/alpha/index.js",
				"#dependency",
				false,
				"node_modules/beta/index.js",
			),
			(
				"node_modules/alpha/index.js",
				"./lib/internal.js",
				false,
				"node_modules/alpha/lib/internal.js",
			),
			(
				"node_modules/alpha/index.js",
				"beta",
				false,
				"node_modules/beta/index.js",
			),
		] {
			let output = resolve(&root, referrer, specifier, types).unwrap();
			assert_eq!(output.path, root.join(expected));
			assert!(output.metadata.contains(&root.join(FILE_NAME)));
			assert!(
				output
					.metadata
					.contains(&root.join("node_modules/alpha/package.json"))
			);
		}
	}

	#[test]
	fn rejects_unavailable_and_undeclared_capabilities() {
		let directory = fixture();
		let root = directory.path();
		for specifier in [
			"alpha/private",
			"alpha/../beta/index.js",
			"node:fs",
			"https://example.com/a.js",
			"../outside.js",
			"./tangram.ts?query",
			"missing",
		] {
			assert!(
				resolve(root, "tangram.ts", specifier, false).is_err(),
				"{specifier}"
			);
		}
		assert!(resolve(root, "node_modules/beta/index.js", "alpha", false).is_err());
		write(
			root,
			"node_modules/beta/package.json",
			r#"{"name":"beta","type":"commonjs","exports":"./index.js"}"#,
		);
		assert!(resolve(root, "tangram.ts", "beta", false).is_err());
	}

	#[test]
	fn installation_ownership() {
		let directory = fixture();
		let root = directory.path().canonicalize().unwrap();
		write(
			&root,
			"node_modules/alpha/dist/package.json",
			r#"{"type":"module"}"#,
		);
		write(&root, "node_modules/alpha/dist/index.js", "");
		let output = resolve(&root, "node_modules/alpha/dist/index.js", "beta", false).unwrap();
		assert_eq!(output.path, root.join("node_modules/beta/index.js"));
		assert!(
			output
				.metadata
				.contains(&root.join("node_modules/alpha/package.json"))
		);
		assert!(
			output
				.metadata
				.contains(&root.join("node_modules/alpha/dist/package.json"))
		);
		std::fs::create_dir(root.join("packages")).unwrap();
		std::fs::rename(root.join("node_modules/alpha"), root.join("packages/alpha")).unwrap();
		std::os::unix::fs::symlink("../packages/alpha", root.join("node_modules/alpha")).unwrap();
		let output = resolve(&root, "tangram.ts", "alpha", false).unwrap();
		assert_eq!(output.path, root.join("packages/alpha/index.js"));
	}

	#[test]
	fn metadata_reference_roundtrip() {
		let reference =
			tg::Reference::with_node(tg::reference::Node::Metadata("../package.json".into()));
		assert_eq!(
			reference.to_string().parse::<tg::Reference>().unwrap(),
			reference
		);
		assert!(!reference.is_solvable());
	}

	#[test]
	fn metadata_and_condition_changes_are_observed() {
		let directory = fixture();
		let root = directory.path();
		assert!(
			resolve(root, "tangram.ts", "alpha", false)
				.unwrap()
				.path
				.ends_with("index.js")
		);
		write(root, FILE_NAME, r#"{"conditions":[]}"#);
		assert!(
			resolve(root, "tangram.ts", "alpha", false)
				.unwrap()
				.path
				.ends_with("fallback.js")
		);
		write(
			root,
			"node_modules/alpha/package.json",
			r#"{"name":"alpha","type":"module","exports":"./lib/internal.js"}"#,
		);
		assert!(
			resolve(root, "tangram.ts", "alpha", false)
				.unwrap()
				.path
				.ends_with("lib/internal.js")
		);
	}

	#[test]
	fn rejects_symlink_escape_and_ambient_packages() {
		let directory = fixture();
		let outside = tempfile::tempdir().unwrap();
		write(outside.path(), "index.js", "");
		std::fs::remove_file(directory.path().join("node_modules/beta/index.js")).unwrap();
		std::os::unix::fs::symlink(
			outside.path().join("index.js"),
			directory.path().join("node_modules/beta/index.js"),
		)
		.unwrap();
		assert!(resolve(directory.path(), "tangram.ts", "beta", false).is_err());
		write(directory.path(), "nested/tangram.ts", "");
		assert!(
			Scope::try_for_path(&directory.path().join("nested/tangram.ts"))
				.unwrap()
				.is_none()
		);
	}

	#[test]
	fn relative_targets_are_exact_and_package_main_is_supported() {
		let directory = fixture();
		let root = directory.path();
		assert!(resolve(root, "node_modules/alpha/index.js", "./lib/internal", false).is_err());
		assert!(resolve(root, "node_modules/alpha/index.js", "./lib", false).is_err());
		write(
			root,
			"node_modules/alpha/package.json",
			r#"{"name":"alpha","type":"module","exports":"./lib/internal"}"#,
		);
		assert!(resolve(root, "tangram.ts", "alpha", false).is_err());
		write(
			root,
			"node_modules/alpha/package.json",
			r#"{"name":"alpha","type":"module","main":"./index.js"}"#,
		);
		assert!(
			resolve(root, "tangram.ts", "alpha", false)
				.unwrap()
				.path
				.ends_with("alpha/index.js")
		);
	}

	#[test]
	fn does_not_search_parent_node_modules() {
		let directory = tempfile::tempdir().unwrap();
		let root = directory.path().join("scope");
		write(&root, FILE_NAME, r#"{"conditions":[]}"#);
		write(&root, "tangram.ts", "");
		write(&root, "package.json", r#"{"dependencies":{"alpha":"1"}}"#);
		write(
			directory.path(),
			"node_modules/alpha/package.json",
			r#"{"type":"module","exports":"./index.js"}"#,
		);
		write(directory.path(), "node_modules/alpha/index.js", "");
		assert!(resolve(&root, "tangram.ts", "alpha", false).is_err());
	}

	#[test]
	fn module_reference_roundtrip() {
		for specifier in [
			"alpha/feature",
			"@scope/package",
			"#internal",
			"./relative.js",
		] {
			for types in [false, true] {
				let mut import =
					tg::module::Import::with_specifier_and_attributes(specifier, None).unwrap();
				import.types = types;
				let reference = Import::reference(&import);
				assert_eq!(
					reference.to_string().parse::<tg::Reference>().unwrap(),
					reference
				);
				assert!(!reference.is_solvable());
			}
		}
	}
}
