use sea_orm::*;
use uuid::Uuid;

use crate::storage::entities::user;

pub async fn find_by_email(db: &DatabaseConnection, email: &str) -> Result<Option<user::Model>, DbErr> {
    user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
}

pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<user::Model>, DbErr> {
    user::Entity::find_by_id(id)
        .one(db)
        .await
}

pub async fn create(
    db: &DatabaseConnection,
    id: Uuid,
    name: &str,
    email: &str,
    password_hash: &str,
) -> Result<user::Model, DbErr> {
    let now = chrono::Utc::now();
    
    let user = user::ActiveModel {
        id: Set(id),
        name: Set(name.to_string()),
        email: Set(email.to_string()),
        password: Set(password_hash.to_string()),
        created_at: Set(now),
    };

    user.insert(db).await
}

pub async fn email_exists(db: &DatabaseConnection, email: &str) -> Result<bool, DbErr> {
    let count = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .count(db)
        .await?;

    Ok(count > 0)
}
