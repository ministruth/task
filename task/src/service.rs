use skynet_api::{
    ffi_rpc::{self, async_trait, bincode, ffi_rpc_macro::plugin_impl_trait, registry::Registry},
    sea_orm::{ActiveModelTrait, Set},
    service::SResult,
    HyUuid,
};
use skynet_api_task::{entity::tasks, semver::Version, TaskCallback};

use crate::{Plugin, PLUGIN_INSTANCE};

#[plugin_impl_trait]
impl skynet_api_task::Service for Plugin {
    async fn api_version(&self, _: &Registry) -> Version {
        Version::parse(skynet_api_task::VERSION).unwrap()
    }

    async fn create(
        &self,
        _: &Registry,
        name: String,
        detail: Option<String>,
        cb: String,
    ) -> SResult<HyUuid> {
        self.runtime.block_on(async {
            let m = tasks::ActiveModel {
                name: Set(name),
                detail: Set(detail),
                ..Default::default()
            }
            .insert(PLUGIN_INSTANCE.db.get().unwrap())
            .await?;
            self.cb.insert(m.id, cb);
            Ok(m.id)
        })
    }

    async fn stop(&self, r: &Registry, id: HyUuid) -> bool {
        let x = self.cb.get(&id).map(|x| x.to_owned());
        match x {
            Some(x) => match r.get(&x) {
                Some(x) => {
                    let x: TaskCallback = x.into();
                    x.stop(r, &id).await
                }
                None => false,
            },
            None => false,
        }
    }
}
