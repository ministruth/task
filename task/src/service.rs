use std::collections::BTreeMap;

use actix_cloud::tokio::runtime;
use rhai::{Engine, EvalAltResult, Position};
use skynet_api::{
    HyUuid, Result, anyhow, bail,
    ffi_rpc::{self, async_trait, ffi_rpc_macro::plugin_impl_trait, registry::Registry, rmp_serde},
    sea_orm::TransactionTrait,
    service::SResult,
};
use skynet_api_task::{
    TaskCallback, TaskScript, Value,
    semver::Version,
    viewer::{scripts::ScriptViewer, tasks::TaskViewer},
};

use crate::{PLUGIN_INSTANCE, Plugin};

impl Plugin {
    fn is_script_aborted(&self, id: &HyUuid) -> bool {
        self.script_handle.get(id).is_some_and(|x| *x)
    }

    fn script_abort(&self, id: &HyUuid) -> bool {
        if let Some(mut x) = self.script_handle.get_mut(id) {
            *x = true;
            true
        } else {
            false
        }
    }

    fn param_script(p: BTreeMap<String, Value>) -> rhai::Map {
        let mut ret = rhai::Map::new();
        for (k, v) in p {
            ret.insert(
                k.into(),
                match v {
                    Value::String(x) => x.into(),
                    Value::Integer(x) => x.into(),
                    Value::Float(x) => x.into(),
                    Value::Bool(x) => x.into(),
                },
            );
        }
        ret
    }

    fn param_plugin(p: &rhai::Map) -> Result<BTreeMap<String, Value>> {
        let mut ret = BTreeMap::new();
        for (k, v) in p {
            let v = if v.is::<i64>() {
                Value::Integer(v.as_int().unwrap())
            } else if v.is::<String>() {
                Value::String(v.to_owned().into_string().unwrap())
            } else if v.is::<f64>() {
                Value::Float(v.as_float().unwrap())
            } else if v.is::<bool>() {
                Value::Bool(v.as_bool().unwrap())
            } else {
                bail!("Invalid param type {}", v.type_name());
            };
            ret.insert(k.to_string(), v);
        }
        Ok(ret)
    }
}

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
        let m = TaskViewer::create(PLUGIN_INSTANCE.db.get().unwrap(), &name, &detail).await?;
        self.cb.insert(m.id, cb);
        Ok(m.id)
    }

    async fn stop(&self, r: &Registry, id: HyUuid) -> bool {
        let _ = TaskViewer::finish_out(
            PLUGIN_INSTANCE.db.get().unwrap(),
            &id,
            9,
            "Task aborted by the user",
        )
        .await;
        let x = self.cb.get(&id).map(|x| x.to_owned());
        match x {
            Some(x) => {
                if x == "self" {
                    self.script_abort(&id)
                } else {
                    match r.get(&x) {
                        Some(x) => {
                            let x: TaskCallback = x.into();
                            x.stop(r, &id).await
                        }
                        None => false,
                    }
                }
            }
            None => false,
        }
    }

    async fn create_script(
        &self,
        r: &Registry,
        name: String,
        detail: Option<String>,
        sid: HyUuid,
    ) -> SResult<Option<HyUuid>> {
        let s = ScriptViewer::find_by_id(PLUGIN_INSTANCE.db.get().unwrap(), &sid).await?;
        match s {
            Some(s) => Ok(Some(self.create_code(r, name, detail, s.code).await?)),
            None => Ok(None),
        }
    }

    async fn create_code(
        &self,
        r: &Registry,
        name: String,
        detail: Option<String>,
        code: String,
    ) -> SResult<HyUuid> {
        let r = r.clone();
        let id = self.create(&r, name, detail, String::from("self")).await?;
        self.script_handle.insert(id, false);
        runtime::Handle::current().spawn_blocking(move || {
            let mut engine = Engine::new();
            let _id = id.clone();
            engine.register_fn(
                "task_update",
                move |output: &str, percent: i64| -> Result<(), Box<EvalAltResult>> {
                    if PLUGIN_INSTANCE.is_script_aborted(&_id) {
                        return Err(EvalAltResult::ErrorTerminated(
                            "Aborted".into(),
                            Position::NONE,
                        )
                        .into());
                    }
                    let output = output.to_owned();
                    runtime::Handle::current()
                        .block_on(async {
                            let tx = PLUGIN_INSTANCE.db.get().unwrap().begin().await?;
                            TaskViewer::update(&tx, &_id, &output, percent as u32).await?;
                            tx.commit().await?;
                            Ok(())
                        })
                        .map_err(|x: anyhow::Error| x.to_string().into())
                },
            );
            let _id = id.clone();
            let _r = r.clone();
            engine.register_fn(
                "api_call",
                move |pid: &str,
                      name: &str,
                      param: rhai::Map|
                      -> Result<rhai::Map, Box<EvalAltResult>> {
                    if PLUGIN_INSTANCE.is_script_aborted(&_id) {
                        return Err(EvalAltResult::ErrorTerminated(
                            "Aborted".into(),
                            Position::NONE,
                        )
                        .into());
                    }
                    if let Some(x) = _r.get(pid) {
                        runtime::Handle::current()
                            .block_on(async {
                                let ret = TaskScript::from(x)
                                    .call(&_r, name, &Self::param_plugin(&param)?)
                                    .await?;
                                Ok(Self::param_script(ret))
                            })
                            .map_err(|x: anyhow::Error| x.to_string().into())
                    } else {
                        return Err("Plugin ID not exist".into());
                    }
                },
            );
            let ret = engine.eval::<i64>(&code);
            if !PLUGIN_INSTANCE.is_script_aborted(&id) {
                runtime::Handle::current().block_on(async {
                    match ret {
                        Ok(ret) => {
                            let _ = TaskViewer::finish(
                                PLUGIN_INSTANCE.db.get().unwrap(),
                                &id,
                                ret as i32,
                            )
                            .await;
                        }
                        Err(e) => {
                            let _ = TaskViewer::finish_out(
                                PLUGIN_INSTANCE.db.get().unwrap(),
                                &id,
                                1,
                                &e.to_string(),
                            )
                            .await;
                        }
                    }
                });
            }
            PLUGIN_INSTANCE.script_handle.remove(&id);
        });
        Ok(id)
    }
}
