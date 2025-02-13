use async_trait::async_trait;

use crate::{db::DBClient, dtos::TrackDto};


#[async_trait]
pub trait TrackExt {
    async fn get_random_tracks(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<TrackDto>, sqlx::Error>;
}

#[async_trait]
impl TrackExt for DBClient {
    async fn get_random_tracks(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<TrackDto>, sqlx::Error> {

        let tracks = sqlx::query_as!(
            TrackDto,
            r#"
            SELECT 
                t.id,
                t.title,
                t.artist,
                t.upload_status,
                t.duration,
                t.file_name,
                t.thumbnail_name,
                COALESCE(ph.played_at, NULL) AS played_at,
                CASE WHEN uf.id IS NOT NULL THEN true ELSE false END as is_favorite,
                COALESCE(ph.duration_played, INTERVAL '0 seconds') AS duration_played,  -- Default to 0 if no playback history
                CASE WHEN t.user_id = $1 THEN true ELSE false END as is_created_by_user
            FROM tracks t
            LEFT JOIN user_favorites uf 
                ON t.id = uf.track_id AND uf.user_id = $1
            LEFT JOIN playback_history ph 
                ON t.id = ph.track_id AND ph.user_id = $1 -- Join to get duration_played
            WHERE t.upload_status = 'complete'
            ORDER BY RANDOM()
            LIMIT 20
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tracks)
    }
}