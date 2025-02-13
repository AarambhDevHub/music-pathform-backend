use async_trait::async_trait;
use sqlx::postgres::types::PgInterval;

use crate::{db::DBClient, dtos::TrackDto};

#[async_trait]
pub trait HistoryExt {
    async fn update_or_insert_playback_hisotry(
        &self,
        track_id: uuid::Uuid,
        user_id: uuid::Uuid,
        duration_played: i64,
    ) -> Result<(), sqlx::Error>;

    async fn get_user_playback_history(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<TrackDto>, sqlx::Error>;
}

#[async_trait]
impl HistoryExt for DBClient {
    async fn update_or_insert_playback_hisotry(
        &self,
        track_id: uuid::Uuid,
        user_id: uuid::Uuid,
        duration_played: i64,
    ) -> Result<(), sqlx::Error> {
        
        let existing_entry = sqlx::query!(
            r#"
            SELECT id, duration_played, played_at
            FROM playback_history
            WHERE track_id = $1 AND user_id = $2
            "#,
            track_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let duration_pg_interval = PgInterval {
            months: 0,
            days: 0,
            microseconds: duration_played * 1_000_000,
        };
    

        if let Some(entry) = existing_entry {
            sqlx::query!(
                r#"
                UPDATE playback_history
                SET duration_played = $1,
                    played_at = CURRENT_TIMESTAMP
                WHERE id = $2
                "#,
                duration_pg_interval as PgInterval,
                entry.id,
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                r#"
                INSERT INTO playback_history (user_id, track_id, duration_played, played_at)
                VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
                "#,
                user_id,
                track_id,
                duration_pg_interval as PgInterval
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn get_user_playback_history(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<TrackDto>, sqlx::Error> {
        let tracks = sqlx::query_as!(
            TrackDto,  // This specifies the target struct for mapping
            r#"
            SELECT
                t.id,
                t.title,
                t.artist,
                t.duration,
                ph.duration_played,
                ph.played_at,
                t.file_name,
                t.upload_status,
                t.thumbnail_name,
                CASE WHEN uf.id IS NOT NULL THEN TRUE ELSE FALSE END AS is_favorite,
                CASE WHEN t.user_id = $1 THEN true ELSE false END as is_created_by_user
            FROM
                playback_history ph
            JOIN
                tracks t ON ph.track_id = t.id
            LEFT JOIN
                user_favorites uf ON uf.track_id = t.id AND uf.user_id = $1
            WHERE
                ph.user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
    
        Ok(tracks) 
    }
}