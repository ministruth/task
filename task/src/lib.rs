use std::{path::PathBuf, sync::OnceLock};

use actix_cloud::{
    actix_web::web::Data,
    i18n::{i18n, Locale},
    router::CSRFType,
    state::{GlobalState, ServerHandle},
    tokio::runtime::Runtime,
};
use dashmap::DashMap;
use migration::migrator::Migrator;
use sea_orm_migration::MigratorTrait;
use skynet_api::{
    ffi_rpc::{
        self,
        abi_stable::prefix_type::PrefixTypeTrait,
        async_ffi, async_trait, bincode,
        ffi_rpc_macro::{
            plugin_impl_call, plugin_impl_instance, plugin_impl_root, plugin_impl_trait,
        },
        registry::Registry,
    },
    permission::{IDTypes::PermManagePluginID, PermChecker, PERM_READ, PERM_WRITE},
    plugin::{PluginStatus, Request, Response},
    request::{Method, Router, RouterType},
    route,
    sea_orm::DatabaseConnection,
    service::{SResult, Service, SKYNET_SERVICE},
    uuid, HyUuid, MenuItem, Skynet,
};
use skynet_api_task::{viewer::tasks::TaskViewer, ID};

mod api;
mod migration;
mod service;

include!(concat!(env!("OUT_DIR"), "/response.rs"));

#[plugin_impl_instance(|| Plugin{
    cb: Default::default(),
    db: Default::default(),
    state: Default::default(),
    runtime: Runtime::new().unwrap(),
})]
#[plugin_impl_root]
#[plugin_impl_call(skynet_api::plugin::api::PluginApi, skynet_api_task::Service)]
struct Plugin {
    cb: DashMap<HyUuid, String>,
    db: OnceLock<DatabaseConnection>,
    state: OnceLock<Data<GlobalState>>,
    runtime: Runtime,
}

#[plugin_impl_trait]
impl skynet_api::plugin::api::PluginApi for Plugin {
    async fn on_load(
        &self,
        reg: &Registry,
        mut skynet: Skynet,
        _runtime_path: PathBuf,
    ) -> SResult<Skynet> {
        self.runtime.block_on(async {
            let server: Service = reg.get(SKYNET_SERVICE).unwrap().into();
            skynet.logger.plugin_start(server);

            let db = skynet.get_db().await?;
            Migrator::up(&db, None).await?;
            let _ = self.db.set(db);

            TaskViewer::clean_running(self.db.get().unwrap()).await?;

            let _ = skynet.insert_menu(
                MenuItem {
                    id: HyUuid(uuid!("ee689b2e-beaa-43ac-837d-466cad5ff999")),
                    plugin: Some(ID),
                    name: format!("{ID}.menu.task"),
                    path: format!("/plugin/{ID}/"),
                    checker: PermChecker::new_entry(
                        skynet.default_id[PermManagePluginID],
                        PERM_READ,
                    ),
                    ..Default::default()
                },
                1,
                Some(HyUuid(uuid!("d00d36d0-6068-4447-ab04-f82ce893c04e"))),
            );
            let locale = Locale::new(skynet.config.lang.clone()).add_locale(i18n!("locales"));
            let state = GlobalState {
                config: Default::default(),
                logger: None,
                locale,
                server: ServerHandle::default(),
            }
            .build();
            let _ = self.state.set(state);
            Ok(skynet)
        })
    }

    async fn on_register(&self, _: &Registry, skynet: Skynet, mut r: Vec<Router>) -> Vec<Router> {
        r.extend(vec![
            Router {
                path: format!("/plugins/{ID}/tasks"),
                method: Method::Get,
                route: RouterType::Http(ID, String::from("api::get_all")),
                checker: PermChecker::new_entry(skynet.default_id[PermManagePluginID], PERM_READ),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/tasks"),
                method: Method::Delete,
                route: RouterType::Http(ID, String::from("api::delete_completed")),
                checker: PermChecker::new_entry(skynet.default_id[PermManagePluginID], PERM_WRITE),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/tasks/{{tid}}/output"),
                method: Method::Get,
                route: RouterType::Http(ID, String::from("api::get_output")),
                checker: PermChecker::new_entry(skynet.default_id[PermManagePluginID], PERM_READ),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/tasks/{{tid}}/stop"),
                method: Method::Post,
                route: RouterType::Http(ID, String::from("api::stop")),
                checker: PermChecker::new_entry(skynet.default_id[PermManagePluginID], PERM_WRITE),
                csrf: CSRFType::Header,
            },
        ]);
        r
    }

    async fn on_route(&self, reg: &Registry, name: String, req: Request) -> SResult<Response> {
        self.runtime.block_on(async {
            route!(reg, self.state.get().unwrap().clone(), name, req,
                "api::get_all" => api::get_all,
                "api::delete_completed" => api::delete_completed,
                "api::get_output" => api::get_output,
                "api::stop" => api::stop,
            )
        })
    }

    async fn on_translate(&self, _: &Registry, str: String, lang: String) -> String {
        self.state.get().unwrap().locale.translate(lang, str)
    }

    async fn on_unload(&self, _: &Registry, _status: PluginStatus) {}
}
