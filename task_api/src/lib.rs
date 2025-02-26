use ffi_rpc::{
    self, abi_stable, async_trait, bincode,
    ffi_rpc_macro::{self, plugin_api},
};
use semver::Version;
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
}

#[plugin_api(TaskCallback)]
pub trait Callback: Send + Sync {
    async fn stop(id: HyUuid) -> bool;
}
