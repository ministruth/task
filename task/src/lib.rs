use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use actix_cloud::{
    actix_web::web::Data,
    i18n::{Locale, i18n},
    memorydb,
    router::CSRFType,
    state::{GlobalState, ServerHandle},
    tokio,
};
use dashmap::DashMap;
use migration::migrator::Migrator;
use sea_orm_migration::MigratorTrait;
use skynet_api::{
    HyUuid, MenuItem, Skynet,
    ffi_rpc::{
        self,
        abi_stable::prefix_type::PrefixTypeTrait,
        async_ffi, async_trait,
        ffi_rpc_macro::{
            plugin_impl_call, plugin_impl_instance, plugin_impl_root, plugin_impl_trait,
        },
        registry::Registry,
        rmp_serde,
    },
    permission::{PERM_READ, PERM_WRITE, PermChecker},
    plugin::{PluginStatus, Request, Response},
    request::{Method, Router, RouterType},
    route,
    sea_orm::{DatabaseConnection, TransactionTrait},
    service::{SKYNET_SERVICE, SResult, Service},
    uuid,
    viewer::permissions::PermissionViewer,
};
use skynet_api_task::{ID, viewer::tasks::TaskViewer};

mod api;
mod migration;
mod service;

include!(concat!(env!("OUT_DIR"), "/response.rs"));

#[plugin_impl_instance(|| Plugin{
    cb: Default::default(),
    db: Default::default(),
    state: Default::default(),
    view_id: Default::default(),
    manage_id: Default::default(),
    script_handle: Default::default(),
})]
#[plugin_impl_root]
#[plugin_impl_call(skynet_api::plugin::api::PluginApi, skynet_api_task::Service)]
struct Plugin {
    cb: DashMap<HyUuid, String>,
    db: OnceLock<DatabaseConnection>,
    state: OnceLock<Data<GlobalState>>,
    view_id: OnceLock<HyUuid>,
    manage_id: OnceLock<HyUuid>,
    script_handle: DashMap<HyUuid, bool>,
}

#[plugin_impl_trait]
impl skynet_api::plugin::api::PluginApi for Plugin {
    async fn on_load(
        &self,
        reg: &Registry,
        mut skynet: Skynet,
        _runtime_path: PathBuf,
    ) -> SResult<Skynet> {
        let server: Service = reg.get(SKYNET_SERVICE).unwrap().into();
        skynet.logger.plugin_start(server);

        let db = skynet.get_db().await?;
        Migrator::up(&db, None).await?;
        let _ = self.db.set(db);

        let tx = self.db.get().unwrap().begin().await?;
        let _ = self.view_id.set(
            PermissionViewer::find_or_init(&tx, &format!("view.{ID}"), "plugin task viewer")
                .await?
                .id,
        );
        let _ = self.manage_id.set(
            PermissionViewer::find_or_init(&tx, &format!("manage.{ID}"), "plugin task manager")
                .await?
                .id,
        );
        tx.commit().await?;

        TaskViewer::clean_running(self.db.get().unwrap()).await?;

        let _ = skynet.insert_menu(
            MenuItem {
                id: HyUuid(uuid!("ee689b2e-beaa-43ac-837d-466cad5ff999")),
                plugin: Some(ID),
                name: String::from("menu.task"),
                path: format!("/plugin/{ID}/task"),
                checker: PermChecker::new_entry(*self.view_id.get().unwrap(), PERM_READ),
                ..Default::default()
            },
            0,
            Some(HyUuid(uuid!("d00d36d0-6068-4447-ab04-f82ce893c04e"))),
        );
        let _ = skynet.insert_menu(
            MenuItem {
                id: HyUuid(uuid!("f046860e-ebf7-48e8-b4ff-151fc4e19b6e")),
                plugin: Some(ID),
                name: String::from("menu.task"),
                path: format!("/plugin/{ID}/script"),
                checker: PermChecker::new_entry(*self.manage_id.get().unwrap(), PERM_READ),
                ..Default::default()
            },
            1,
            Some(HyUuid(uuid!("cca5b3b0-40a3-465c-8b08-91f3e8d3b14d"))),
        );
        let locale = Locale::new(skynet.config.lang.clone()).add_locale(i18n!("locales"));
        let state = GlobalState {
            memorydb: Arc::new(memorydb::default::DefaultBackend::new()),
            config: Default::default(),
            logger: None,
            locale,
            server: ServerHandle::default(),
        }
        .build();
        let _ = self.state.set(state);
        Ok(skynet)
    }

    async fn on_register(&self, _: &Registry, _skynet: Skynet, mut r: Vec<Router>) -> Vec<Router> {
        let view_id = *self.view_id.get().unwrap();
        let manage_id = *self.manage_id.get().unwrap();
        r.extend(vec![
            Router {
                path: format!("/plugins/{ID}/tasks"),
                method: Method::Get,
                route: RouterType::Http(ID, String::from("api::get_tasks")),
                checker: PermChecker::new_entry(view_id, PERM_READ),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/tasks"),
                method: Method::Delete,
                route: RouterType::Http(ID, String::from("api::delete_completed")),
                checker: PermChecker::new_entry(view_id, PERM_WRITE),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/tasks/{{tid}}/output"),
                method: Method::Get,
                route: RouterType::Http(ID, String::from("api::get_output")),
                checker: PermChecker::new_entry(view_id, PERM_READ),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/tasks/{{tid}}/stop"),
                method: Method::Post,
                route: RouterType::Http(ID, String::from("api::stop")),
                checker: PermChecker::new_entry(view_id, PERM_WRITE),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/scripts"),
                method: Method::Get,
                route: RouterType::Http(ID, String::from("api::get_scripts")),
                checker: PermChecker::new_entry(manage_id, PERM_READ),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/scripts/{{sid}}"),
                method: Method::Get,
                route: RouterType::Http(ID, String::from("api::get_script")),
                checker: PermChecker::new_entry(manage_id, PERM_READ),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/scripts"),
                method: Method::Post,
                route: RouterType::Http(ID, String::from("api::add_script")),
                checker: PermChecker::new_entry(manage_id, PERM_WRITE),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/scripts/{{sid}}"),
                method: Method::Put,
                route: RouterType::Http(ID, String::from("api::put_script")),
                checker: PermChecker::new_entry(manage_id, PERM_WRITE),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/scripts"),
                method: Method::Delete,
                route: RouterType::Http(ID, String::from("api::delete_script_batch")),
                checker: PermChecker::new_entry(manage_id, PERM_WRITE),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/scripts/{{sid}}"),
                method: Method::Delete,
                route: RouterType::Http(ID, String::from("api::delete_script")),
                checker: PermChecker::new_entry(manage_id, PERM_WRITE),
                csrf: CSRFType::Header,
            },
            Router {
                path: format!("/plugins/{ID}/scripts/{{sid}}/run"),
                method: Method::Post,
                route: RouterType::Http(ID, String::from("api::run_script")),
                checker: PermChecker::new_entry(manage_id, PERM_WRITE),
                csrf: CSRFType::Header,
            },
        ]);
        r
    }

    async fn on_route(&self, reg: &Registry, name: String, req: Request) -> SResult<Response> {
        route!(reg, self.state.get().unwrap(), name, req,
            "api::get_tasks" => api::get_tasks,
            "api::delete_completed" => api::delete_completed,
            "api::get_output" => api::get_output,
            "api::stop" => api::stop,
            "api::get_scripts" => api::get_scripts,
            "api::get_script" => api::get_script,
            "api::add_script" => api::add_script,
            "api::put_script" => api::put_script,
            "api::delete_script_batch" => api::delete_script_batch,
            "api::delete_script" => api::delete_script,
            "api::run_script" => api::run_script,
        )
    }

    async fn on_translate(&self, _: &Registry, str: String, lang: String) -> String {
        self.state.get().unwrap().locale.translate(lang, str)
    }

    async fn on_unload(&self, _: &Registry, _status: PluginStatus) {}
}
