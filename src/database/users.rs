use async_trait::async_trait;
use uuid::Uuid;

use crate::{db::DBClient, models::User};

#[async_trait]
pub trait UserExt {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        username: Option<&str>,
        email: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error>;

    async fn save_user<T: Into<String> + Send>(
        &self,
        username: T,
        email: T,
        password_hash: T,
    ) -> Result<User, sqlx::Error>;

    async fn update_username<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        username: T
    ) -> Result<User, sqlx::Error>;

    async fn update_user_password_hash(
        &self,
        user_id: Uuid,
        new_password_hash: String
    ) -> Result<User, sqlx::Error>;
}

#[async_trait]
impl UserExt for DBClient {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        username: Option<&str>,
        email: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error> {
        
        let query = r#"
            SELECT 
                id, 
                username, 
                email, 
                password_hash,  
                created_at, 
                updated_at 
            FROM users 
            WHERE 
                ($1::uuid IS NULL OR id = $1) AND
                ($2::text IS NULL OR username = $2) AND
                ($3::text IS NULL OR email = $3)
        "#;

    
        let user = sqlx::query_as::<_, User>(query)
            .bind(user_id)
            .bind(username)
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
    
        Ok(user)
    }
    

    async fn save_user<T: Into<String> + Send>(
        &self,
        username: T,
        email: T,
        password_hash: T,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash) 
            VALUES ($1, $2, $3) 
            RETURNING id, username, email, password_hash, created_at, updated_at
            "#,
            username.into(),
            email.into(),
            password_hash.into(),
        ).fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_username<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        username: T
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET username = $1, updated_at = Now()
            WHERE id = $2
            RETURNING id, username, email, password_hash, created_at, updated_at
            "#,
            username.into(),
            user_id
        ).fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user_password_hash(
        &self,
        user_id: Uuid,
        new_password_hash: String
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = Now()
            WHERE id = $2
            RETURNING id, username, email, password_hash, created_at, updated_at
            "#,
            new_password_hash,
            user_id
        ).fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}