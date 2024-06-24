
/// Version of the MVS SDK
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MVSVersion(u32);

impl std::fmt::Display for MVSVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}-test.{}", self.main(), self.sub(), self.rev(), self.test())
    }
}

impl MVSVersion {
    pub fn main(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub fn sub(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn rev(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn test(&self) -> u8 {
        self.0 as u8
    }

    pub fn as_semver(&self) -> semver::Version {
        use semver::*;

        Version {
            major: self.main() as u64,
            minor: self.sub() as u64,
            patch: self.rev() as u64,
            //pre: Prerelease::new(&format!("test.{}", self.test)).unwrap(),
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        }
    }
}

impl From<u32> for MVSVersion {
    fn from(version: u32) -> Self {
        MVSVersion(version)
    }
}

impl From<MVSVersion> for u32 {
    fn from(version: MVSVersion) -> Self {
        version.0
    }
}

impl From<MVSVersion> for semver::Version {
    fn from(version: MVSVersion) -> Self {
        version.as_semver()
    }
}