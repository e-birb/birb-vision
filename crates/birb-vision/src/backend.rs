use std::{collections::HashMap, fmt::Debug, sync::Arc};

use birb_vision_core::{anyhow::{self, anyhow}, context::VisionContext};
use serde::{Deserialize, Serialize};


#[derive(Clone)]
pub struct BackendRegistry {
    packages: HashMap<String, Arc<BackendPackage>>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Register a new backend provider.
    ///
    /// If the provider already exists, this function will return an error.
    pub fn register(
        &mut self,
        package: BackendPackage,
    ) -> Result<(), anyhow::Error> {
        let identifier = package.identifier.clone();
        self.packages
            .insert(
                identifier.clone(),
                Arc::new(package),
            ).is_none().then(|| ())
            .ok_or(anyhow!("Backend provider for type {:?} already exists", identifier).into())
    }

    /// Build a new backend for the given type name.
    pub fn get_backend(&self, type_name: impl AsRef<str>) -> Option<ContextProviderResult> {
        self.packages
            .get(type_name.as_ref())
            .map(|p| p.build_backend())
    }

    pub fn all_packages(&self) -> HashMap<String, Arc<BackendPackage>> {
        self.packages
            .clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackendPackageInfo {
    pub identifier: String,
    pub display_name: String,
    pub description: Option<String>,
}

pub struct BackendPackage {
    pub identifier: String,
    pub display_name: String,
    pub logo: Option<LogoImageFile>,
    pub description: Option<String>,
    pub builder: Box<dyn Fn() -> ContextProviderResult + Send + Sync + 'static>,
}

impl Debug for BackendPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendPackage")
            .field("identifier", &self.identifier)
            .field("display_name", &self.display_name)
            .field("logo", &self.logo)
            .field("description", &self.description)
            .finish()
    }
}

impl BackendPackage {
    pub fn from_builder_fn<T: VisionContext>(builder: impl Fn() -> Result<T, anyhow::Error> + Send + Sync + 'static) -> Self {
        let identifier = std::any::type_name::<T>().to_string();

        let builder = Box::new(move || -> ContextProviderResult {
            builder().map(|backend| Box::new(backend) as Box<dyn VisionContext>)
        });

        Self {
            identifier: identifier.clone(),
            display_name: identifier,
            logo: None,
            description: None,
            builder: Box::new(builder),
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_logo(mut self, logo: LogoImageFile) -> Self {
        self.logo = Some(logo);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn build_backend(&self) -> ContextProviderResult {
        (self.builder)()
    }

    pub fn info(&self) -> BackendPackageInfo {
        BackendPackageInfo {
            identifier: self.identifier.clone(),
            display_name: self.display_name.clone(),
            description: self.description.clone(),
        }
    }
}

type ContextProviderResult = Result<Box<dyn VisionContext>, anyhow::Error>;

pub enum LogoImageFile {
    Path(String),
    StaticBytes(&'static [u8]),
    Bytes(Arc<Vec<u8>>),
}

impl Debug for LogoImageFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogoImageFile::Path(path) => path.fmt(f),
            LogoImageFile::StaticBytes(_) => f.debug_tuple("LogoImageFile::StaticBytes").finish(),
            LogoImageFile::Bytes(_) => f.debug_tuple("LogoImageFile::Bytes").finish(),
        }
    }
}