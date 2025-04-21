use sea_orm_migration::prelude::*;

use super::migrator::table_prefix;

#[derive(Iden)]
enum Tasks {
    Table,
    ID,
    Name,
    Detail,
    Output,
    Result,
    Percent,
    Sid,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Scripts {
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
        manager
            .create_table(
                Table::create()
                    .table(table_prefix(&Tasks::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tasks::ID)
                            .char_len(36)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tasks::Name).string_len(32).not_null())
                    .col(ColumnDef::new(Tasks::Detail).string_len(1024))
                    .col(ColumnDef::new(Tasks::Output).string())
                    .col(ColumnDef::new(Tasks::Result).integer())
                    .col(
                        ColumnDef::new(Tasks::Percent)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Tasks::Sid).char_len(36))
                    .col(ColumnDef::new(Tasks::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Tasks::UpdatedAt).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .to(table_prefix(&Scripts::Table), Scripts::ID)
                            .from_col(Tasks::Sid)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
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
        manager
            .drop_table(Table::drop().table(table_prefix(&Tasks::Table)).to_owned())
            .await?;
        Ok(())
    }
}
