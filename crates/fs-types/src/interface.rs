//! `InterfaceVersion` and `InterfaceReq`: the one place for "name and version" and "name and need" (ADR 0045, 0077 §3).
//!
//! `semver` reads the numbers; this module adds only the name and the way it is written. A version is always three parts
//! (`tree@1.0.0`); a need is a `semver::VersionReq` in Cargo grammar with a comma between comparisons
//! (`tasks@>=0.2.0, <0.3.0`, `tree@^1.0.0`). There is no second text form. Pre-release and build parts are refused
//! (ADR 0064 no. 4).
//!
//! The need reader takes the whole Cargo grammar (`tasks@0.2`, `*`, `~`, spaces). Consumers therefore read only the
//! parsed `req().comparators`, never the text; whoever needs a strict need asks `InterfaceReq::is_full`.
//!
//! Pattern: Value object (compared by value, valid on construction).

use std::fmt;
use std::str::FromStr;

use semver::{Comparator, Op, Prerelease, Version, VersionReq};

/// Why a text is not an interface version or need.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterfaceProblem {
    /// No `@` between the name and the version.
    MissingAt,
    /// The name is empty or breaks `[a-z][a-z0-9.-]*`.
    BadName,
    /// The part after `@` is not a three-part version (`x.y.z`).
    BadVersion(String),
    /// The part after `@` is not a version need (Cargo grammar).
    BadReq(String),
}

impl fmt::Display for InterfaceProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAt => f.write_str("expected `name@version`, no `@` found"),
            Self::BadName => f.write_str("the name must match [a-z][a-z0-9.-]*"),
            Self::BadVersion(why) => write!(f, "not a version x.y.z: {why}"),
            Self::BadReq(why) => write!(f, "not a version need: {why}"),
        }
    }
}

impl std::error::Error for InterfaceProblem {}

fn split(text: &str) -> Result<(&str, &str), InterfaceProblem> {
    let (name, rest) = text.split_once('@').ok_or(InterfaceProblem::MissingAt)?;
    let mut chars = name.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    if !first_ok
        || !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
    {
        return Err(InterfaceProblem::BadName);
    }
    Ok((name, rest))
}

/// What an interface IS: a name and a `semver::Version`, written `name@x.y.z`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InterfaceVersion {
    name: String,
    version: Version,
}

impl InterfaceVersion {
    /// The name of the interface.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The version.
    #[must_use]
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// The default need of a consumer built against this version (the generator's preset): for `0.0.N` it is
    /// `>=0.0.N, <0.1.0` (decision 23.11.2026, the `0.0.x` line stays compatible); from `0.1` on it is the Cargo caret
    /// (`^0.2.1`, `^1.2.0`).
    #[must_use]
    pub fn default_req(&self) -> InterfaceReq {
        let v = &self.version;
        let cmp = |op, major, minor, patch| Comparator {
            op,
            major,
            minor: Some(minor),
            patch: Some(patch),
            pre: Prerelease::EMPTY,
        };
        let comparators = if v.major == 0 && v.minor == 0 {
            vec![cmp(Op::GreaterEq, 0, 0, v.patch), cmp(Op::Less, 0, 1, 0)]
        } else {
            vec![cmp(Op::Caret, v.major, v.minor, v.patch)]
        };
        InterfaceReq {
            name: self.name.clone(),
            req: VersionReq { comparators },
        }
    }
}

impl FromStr for InterfaceVersion {
    type Err = InterfaceProblem;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (name, version) = split(text)?;
        let version =
            Version::parse(version).map_err(|e| InterfaceProblem::BadVersion(e.to_string()))?;
        // ADR 0064 no. 4: only `x.y.z`; no pre-release, no build metadata (same rule as `parse_version` in fs-interfaces).
        if !version.pre.is_empty() || !version.build.is_empty() {
            return Err(InterfaceProblem::BadVersion(
                "pre-release and build parts are not allowed".to_owned(),
            ));
        }
        Ok(Self {
            name: name.to_owned(),
            version,
        })
    }
}

impl fmt::Display for InterfaceVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// What something NEEDS: a name and a `semver::VersionReq`, written `name@<req>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InterfaceReq {
    name: String,
    req: VersionReq,
}

impl InterfaceReq {
    /// The name of the interface.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The need.
    #[must_use]
    pub fn req(&self) -> &VersionReq {
        &self.req
    }

    /// True when every comparator names minor and patch (`>=0.2.0`, `^1.2.3`): no partial versions, no `*`.
    #[must_use]
    pub fn is_full(&self) -> bool {
        !self.req.comparators.is_empty()
            && self
                .req
                .comparators
                .iter()
                .all(|c| c.op != Op::Wildcard && c.minor.is_some() && c.patch.is_some())
    }

    /// Same name and the version meets the need.
    #[must_use]
    pub fn matches(&self, offered: &InterfaceVersion) -> bool {
        self.name == offered.name && self.req.matches(&offered.version)
    }
}

impl FromStr for InterfaceReq {
    type Err = InterfaceProblem;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (name, req) = split(text)?;
        let req = VersionReq::parse(req).map_err(|e| InterfaceProblem::BadReq(e.to_string()))?;
        if req.comparators.iter().any(|c| !c.pre.is_empty()) {
            return Err(InterfaceProblem::BadReq(
                "pre-release parts are not allowed".to_owned(),
            ));
        }
        Ok(Self {
            name: name.to_owned(),
            req,
        })
    }
}

impl fmt::Display for InterfaceReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.req)
    }
}
