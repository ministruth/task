use std::collections::BTreeMap;

use enum_as_inner::EnumAsInner;
use ffi_rpc::{
    self, abi_stable, async_trait,
    ffi_rpc_macro::{self, plugin_api},
    rmp_serde,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use skynet_api::{HyUuid, Result, service::SResult, uuid};

pub use semver;
pub mod entity;
pub mod viewer;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ID: HyUuid = HyUuid(uuid!("4adaf7d3-b877-43c3-82bd-da3689dc3920"));

#[plugin_api(TaskService)]
pub trait Service: Send + Sync {
    async fn api_version() -> Version;
    async fn create(name: String, detail: Option<String>, cb: String) -> SResult<HyUuid>;
    async fn stop(id: HyUuid) -> bool;
    async fn create_script(
        name: String,
        detail: Option<String>,
        sid: HyUuid,
    ) -> SResult<Option<HyUuid>>;
    async fn create_code(name: String, detail: Option<String>, code: String) -> SResult<HyUuid>;
}

#[plugin_api(TaskCallback)]
pub trait Callback: Send + Sync {
    async fn stop(id: HyUuid) -> bool;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, EnumAsInner)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Integer(value.into())
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

#[plugin_api(TaskScript)]
pub trait Script: Send + Sync {
    async fn call(name: String, param: BTreeMap<String, Value>)
    -> SResult<BTreeMap<String, Value>>;
}
