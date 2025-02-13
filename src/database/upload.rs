use async_trait::async_trait;
use uuid::Uuid;
use sqlx::{postgres::types::PgInterval, query, query_as};

use crate::{db::DBClient, dtos::IncompleteTrackInfo, models::AudioFile};

#[async_trait]
pub trait UploadExt {
    async fn upload_file(
        &self,
        user_id: Uuid,
        file_name: &String,
    ) -> Result<Uuid, sqlx::Error>;

    async fn upload_chuck(
        &self,
        track_id: Uuid,
        total_chunks: i32,
        uploaded_chunks: i32,
        current_chunk: i32,
        chunk_path: &String,
    ) -> Result<(), sqlx::Error>;
    
    async fn get_audio_file(&self, track_id: Uuid) -> Result<Option<AudioFile>, sqlx::Error>;

    async fn upload_thumbnail(
        &self,
        track_id: Uuid,
        thumbnail_name: &String,
        title: &String,
        artist: &String,
    ) -> Result<(), sqlx::Error>;

    async fn update_status(
        &self,
        track_id: Uuid,
        duration: i64,
    ) -> Result<(), sqlx::Error>;

    async fn get_incomplete_uploads(
        &self,
        user_id: Uuid
    ) -> Result<Vec<IncompleteTrackInfo>, sqlx::Error>;
}

#[async_trait]
impl UploadExt for DBClient {
    async fn upload_file(
        &self,
        user_id: Uuid,
        file_name: &String,
    ) -> Result<Uuid, sqlx::Error> {
        let query = sqlx::query!(
            r#"
            INSERT INTO tracks (
                user_id, file_name
            ) VALUES (
              $1, $2 
            )
            RETURNING id
            "#,
            user_id,
            file_name,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(query.id)
    }

    async fn upload_chuck(
        &self,
        track_id: Uuid,
        total_chunks: i32,
        uploaded_chunks: i32,
        current_chunk: i32,
        chunk_path: &String,
    ) -> Result<(), sqlx::Error> {
        let existing_file = query_as!(
            AudioFile,
            r#"
            SELECT * FROM audio_files 
            WHERE track_id = $1
            "#,
            track_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(mut audio_file) = existing_file {
            audio_file.uploaded_chunks = uploaded_chunks as i32;
            audio_file.current_chunk = current_chunk as i32;

            query!(
                r#"
                UPDATE audio_files
                SET uploaded_chunks = $1,
                    current_chunk = $2,
                    chunk_path = $3,
                    upload_status = CASE
                        WHEN $2::INTEGER = $4::INTEGER THEN 'complete'
                        ELSE 'incomplete'
                    END,
                    updated_at = Now()
                WHERE id = $5
                "#,
                audio_file.uploaded_chunks,
                audio_file.current_chunk,
                chunk_path,
                total_chunks as i32,
                audio_file.id,
            )
            .execute(&self.pool)
            .await?;

        } else {
            query!(
                r#"
                INSERT INTO audio_files (track_id, total_chunks, uploaded_chunks, current_chunk, chunk_path, upload_status)
                VALUES ($1, $2, $3, $4, $5, 'incomplete')
                "#,
                track_id,
                total_chunks,
                uploaded_chunks,
                current_chunk,
                chunk_path
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn get_audio_file(&self, track_id: Uuid) 
    -> Result<Option<AudioFile>, sqlx::Error> {
        let audio_file = query_as!(
            AudioFile,
            r#"
            SELECT * FROM audio_files 
            WHERE track_id = $1
            "#,
            track_id
        ).fetch_optional(&self.pool)
        .await?;

        Ok(audio_file)
    } 

    async fn upload_thumbnail(
        &self,
        track_id: Uuid,
        thumbnail_name: &String,
        title: &String,
        artist: &String,
    ) -> Result<(), sqlx::Error> {

        query!(
            r#"
            UPDATE tracks
            SET title = $1,
                artist = $2,
                thumbnail_name = $3,
                updated_at = Now()
            WHERE id = $4
            "#,
            title,
            artist,
            thumbnail_name,
            track_id
        ).execute(&self.pool)
        .await?;

        Ok(())

    }

    async fn update_status(
        &self,
        track_id: Uuid,
        duration: i64,
    ) -> Result<(), sqlx::Error> {
        let pg_duration = PgInterval {
            days: 0,
            months: 0,
            microseconds: duration * 1_000_000
        };
        query!(
            r#"
            UPDATE tracks
            SET upload_status = 'complete',
                duration = $2,
                updated_at = Now()
            WHERE id = $1
            "#,
            track_id,
            pg_duration
        ).execute(&self.pool)
        .await?;

        query!(
            r#"
                DELETE FROM audio_files WHERE track_id = $1;
            "#,
            track_id
        ).execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_incomplete_uploads(
        &self,
        user_id: Uuid
    ) -> Result<Vec<IncompleteTrackInfo>, sqlx::Error> {
        let uploads = sqlx::query_as!(
            IncompleteTrackInfo,
            r#"
                SELECT 
                    t.title, 
                    t.artist, 
                    t.thumbnail_name, 
                    t.file_name, 
                    af.track_id, 
                    af.total_chunks, 
                    af.uploaded_chunks, 
                    af.current_chunk
                FROM 
                    tracks t
                JOIN 
                    audio_files af ON t.id = af.track_id
                WHERE 
                    t.user_id = $1 
                    AND t.upload_status = 'incomplete'
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(uploads)
    }
}