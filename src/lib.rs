use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use actix_cloud::{
    self, actix_web::http::Method, async_trait, i18n::i18n, router::CSRFType, state::GlobalState,
    tokio::runtime::Runtime,
};
use migration::migrator::Migrator;
use parking_lot::lock_api::RwLock;
use skynet_api::{
    create_plugin,
    permission::{IDTypes::PermManagePluginID, PermEntry, PermType, PERM_READ, PERM_WRITE},
    plugin::{self, Plugin},
    request::{box_json_router, Router},
    sea_orm::{DatabaseConnection, TransactionTrait},
    uuid, HyUuid, MenuItem, Result, Skynet,
};
use skynet_api_task::ID;

mod api;
mod migration;
mod service;

include!(concat!(env!("OUT_DIR"), "/response.rs"));
static SERVICE: OnceLock<Arc<service::Service>> = OnceLock::new();
static RUNTIME: OnceLock<Runtime> = OnceLock::new();
static DB: OnceLock<DatabaseConnection> = OnceLock::new();

#[derive(Debug, Default)]
struct Task;

#[async_trait]
impl Plugin for Task {
    fn on_load(
        &self,
        _: PathBuf,
        mut skynet: Box<Skynet>,
        mut state: Box<GlobalState>,
    ) -> (Box<Skynet>, Box<GlobalState>, Result<()>) {
        RUNTIME.set(Runtime::new().unwrap()).unwrap();
        let srv = service::Service {
            killer_tx: RwLock::new(HashMap::new()),
        };
        if let Err(e) = RUNTIME.get().unwrap().block_on(async {
            let db = plugin::init_db(&skynet.config.database.dsn, Migrator {}).await?;
            let _ = DB.set(db);

            let tx = DB.get().unwrap().begin().await?;
            srv.clean_running(&tx).await?;
            tx.commit().await?;
            Ok(())
        }) {
            return (skynet, state, Err(e));
        }

        let _ = skynet.insert_menu(
            MenuItem {
                id: HyUuid(uuid!("ee689b2e-beaa-43ac-837d-466cad5ff999")),
                name: format!("{ID}.menu.task"),
                path: format!("/plugin/{ID}/"),
                perm: Some(PermEntry {
                    pid: skynet.default_id[PermManagePluginID],
                    perm: PERM_READ,
                }),
                ..Default::default()
            },
            1,
            Some(HyUuid(uuid!("d00d36d0-6068-4447-ab04-f82ce893c04e"))),
        );
        state.locale = state.locale.add_locale(i18n!("locales"));
        let _ = SERVICE.set(Arc::new(srv));
        (skynet, state, Ok(()))
    }

    fn on_route(&self, skynet: &Skynet, mut r: Vec<Router>) -> Vec<Router> {
        let csrf = CSRFType::Header;
        r.extend(vec![
            Router {
                path: format!("/plugins/{ID}/tasks"),
                method: Method::GET,
                route: box_json_router(api::get_all),
                checker: PermType::Entry(PermEntry {
                    pid: skynet.default_id[PermManagePluginID],
                    perm: PERM_READ,
                }),
                csrf,
            },
            Router {
                path: format!("/plugins/{ID}/tasks"),
                method: Method::DELETE,
                route: box_json_router(api::delete_completed),
                checker: PermType::Entry(PermEntry {
                    pid: skynet.default_id[PermManagePluginID],
                    perm: PERM_WRITE,
                }),
                csrf,
            },
            Router {
                path: format!("/plugins/{ID}/tasks/{{tid}}/output"),
                method: Method::GET,
                route: box_json_router(api::get_output),
                checker: PermType::Entry(PermEntry {
                    pid: skynet.default_id[PermManagePluginID],
                    perm: PERM_READ,
                }),
                csrf,
            },
            Router {
                path: format!("/plugins/{ID}/tasks/{{tid}}/stop"),
                method: Method::POST,
                route: box_json_router(api::stop),
                checker: PermType::Entry(PermEntry {
                    pid: skynet.default_id[PermManagePluginID],
                    perm: PERM_WRITE,
                }),
                csrf,
            },
        ]);
        r
    }
}

create_plugin!(Task, Task::default);
