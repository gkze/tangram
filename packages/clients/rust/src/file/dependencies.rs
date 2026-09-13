use {crate::prelude::*, bytes::Bytes};

/// A source reference or an exact captured package dependency.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum DependencyReference {
	Captured {
		dependency: tg::graph::data::Dependency,
		reference: tg::Reference,
	},
	Reference(tg::Reference),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependenciesXattr {
	pub name: String,
	pub value: Bytes,
}

pub fn dependencies_xattrs(
	references: &[DependencyReference],
	max_value_size: usize,
) -> tg::Result<Vec<DependenciesXattr>> {
	if max_value_size == 0 {
		return Err(tg::error!(
			"the maximum xattr value size must be greater than zero"
		));
	}
	let value = serde_json::to_vec(references)
		.map_err(|error| tg::error!(!error, "failed to serialize the dependencies"))?;
	let count = value.len().div_ceil(max_value_size);
	let xattrs = value
		.chunks(max_value_size)
		.enumerate()
		.map(|(index, value)| {
			let name = if count == 1 {
				tg::file::DEPENDENCIES_XATTR_NAME.to_owned()
			} else {
				format!("{}.{index}", tg::file::DEPENDENCIES_XATTR_NAME)
			};
			let value = Bytes::copy_from_slice(value);
			DependenciesXattr { name, value }
		})
		.collect();

	Ok(xattrs)
}

pub fn deserialize_dependencies_xattr(value: &[u8]) -> tg::Result<Vec<DependencyReference>> {
	serde_json::from_slice(value)
		.map_err(|error| tg::error!(!error, "failed to deserialize the dependencies"))
}

#[must_use]
pub fn is_dependencies_xattr_name(name: &str) -> bool {
	if name == tg::file::DEPENDENCIES_XATTR_NAME {
		return true;
	}
	let Some(suffix) = name
		.strip_prefix(tg::file::DEPENDENCIES_XATTR_NAME)
		.and_then(|suffix| suffix.strip_prefix('.'))
	else {
		return false;
	};
	!suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
}

impl DependencyReference {
	#[must_use]
	pub fn new(reference: tg::Reference, dependency: Option<tg::graph::data::Dependency>) -> Self {
		if (reference.node().is_module() || reference.node().is_metadata())
			&& let Some(dependency) = dependency
		{
			Self::Captured {
				dependency,
				reference,
			}
		} else {
			Self::Reference(reference)
		}
	}

	#[must_use]
	pub fn without_tokens(self) -> Self {
		match self {
			Self::Captured {
				mut dependency,
				reference,
			} => {
				dependency.0.options.tokens = tg::authorization::Tokens::default();
				let reference = reference.without_tokens();
				Self::Captured {
					dependency,
					reference,
				}
			},
			Self::Reference(reference) => Self::Reference(reference.without_tokens()),
		}
	}
}

impl From<tg::Reference> for DependencyReference {
	fn from(reference: tg::Reference) -> Self {
		Self::Reference(reference)
	}
}

#[cfg(test)]
mod tests {
	use {super::*, std::path::PathBuf};

	#[test]
	fn captured_pointer_round_trip() {
		let reference =
			tg::Reference::with_node(tg::reference::Node::Metadata("../package.json".into()));
		let pointer = tg::graph::data::Pointer {
			graph: Some(tg::graph::Id::new(b"graph")),
			index: 3,
			kind: tg::artifact::Kind::File,
		};
		let options = tg::referent::Options {
			path: Some("node_modules/alpha/package.json".into()),
			..Default::default()
		};
		let referent = tg::Referent::new(Some(tg::graph::data::Edge::Pointer(pointer)), options);
		let dependency = tg::graph::data::Dependency(referent);
		let references = vec![DependencyReference::new(reference, Some(dependency))];
		let xattrs = dependencies_xattrs(&references, 32).unwrap();
		assert!(xattrs.len() > 1);
		let bytes = xattrs
			.into_iter()
			.flat_map(|xattr| xattr.value)
			.collect::<Vec<_>>();
		assert_eq!(deserialize_dependencies_xattr(&bytes).unwrap(), references);
	}

	#[test]
	fn json_round_trip() {
		let references = vec![tg::Reference::with_path(PathBuf::from("dependency")).into()];
		let xattrs = dependencies_xattrs(&references, 4).unwrap();
		assert!(xattrs.len() > 1);
		assert_eq!(xattrs[0].name, "user.tangram.dependencies.0");
		let value = xattrs
			.into_iter()
			.flat_map(|xattr| xattr.value)
			.collect::<Vec<_>>();
		assert_eq!(deserialize_dependencies_xattr(&value).unwrap(), references);
	}

	#[test]
	fn unsharded_round_trip() {
		let references = vec![tg::Reference::with_path(PathBuf::from("dependency")).into()];
		let xattrs = dependencies_xattrs(&references, usize::MAX).unwrap();
		assert_eq!(xattrs.len(), 1);
		assert_eq!(xattrs[0].name, "user.tangram.dependencies");
		assert_eq!(
			deserialize_dependencies_xattr(&xattrs[0].value).unwrap(),
			references
		);
	}
}
