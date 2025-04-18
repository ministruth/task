use sea_orm_migration::prelude::*;

use crate::migration::*;
use skynet_api_task::ID;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_table::Migration),
            Box::new(m20250301_000001_create_table::Migration),
        ]
    }

    fn migration_table_name() -> DynIden {
        Alias::new(format!("seaql_migrations_{ID}")).into_iden()
    }
}

pub fn table_prefix(table: &impl types::Iden) -> Alias {
    Alias::new(format!("{}_{}", ID, table.to_string()))
}
