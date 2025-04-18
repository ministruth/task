use sea_orm_migration::prelude::*;

use super::migrator::table_prefix;

#[derive(Iden)]
pub(crate) enum Scripts {
    Table,
    ID,
    Name,
    Code,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(table_prefix(&Scripts::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Scripts::ID)
                            .char_len(36)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Scripts::Name).string_len(32).not_null())
                    .col(ColumnDef::new(Scripts::Code).string().not_null())
                    .col(ColumnDef::new(Scripts::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Scripts::UpdatedAt).big_integer().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(table_prefix(&Scripts::Table))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
