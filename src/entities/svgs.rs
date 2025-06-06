//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.11

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "svgs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub icon_id: i32,
    #[sea_orm(column_type = "Text")]
    pub weight: String,
    #[sea_orm(column_type = "Text")]
    pub src: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::icons::Entity",
        from = "Column::IconId",
        to = "super::icons::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Icons,
}

impl Related<super::icons::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Icons.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
