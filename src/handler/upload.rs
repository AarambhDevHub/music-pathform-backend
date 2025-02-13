use std::{
    fs::{self, File, OpenOptions}, 
    io::Write,
    sync::Arc
};

use axum::{extract::Multipart, response::IntoResponse, routing::post, Extension, Json, Router};
use chrono::Duration;
use symphonia::core::{formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint};

use crate::{auth::JWTAuthMiddleware, database::upload::UploadExt, dtos::{Response, UploadResponse}, error::HttpError, AppState};

fn sanitize_filename(filename: &str) -> String {
    filename.replace(&['/', '\\'][..], "").replace("..", "")
}

fn is_upload_complete(temp_dir: &str, total_chunks: usize) -> bool {
    match fs::read_dir(temp_dir) {
        Ok(entries) => entries.count() == total_chunks,
        Err(_) => false,
    }
}

fn get_audio_duration(file_path: &str) -> Result<Duration, Box<dyn std::error::Error>> {
      // Open the audio file
      let file = File::open(file_path)?;
      let mss = MediaSourceStream::new(Box::new(file), Default::default());
  
      // Create a hint to help the format registry guess the format
      let mut hint = Hint::new();
      
      // Provide the file extension as a format hint
      if let Some(extension) = file_path.split('.').last() {
          hint.with_extension(extension);
      }
  
      // Probe the media file
      let probed = symphonia::default::get_probe().format(
          &hint,
          mss,
          &FormatOptions::default(),
          &MetadataOptions::default(),
      )?;
      
      // Get the default track
      let track = probed.format.default_track().ok_or("No track found")?;
      
      // Get the time base for the track
      let time_base = track.codec_params.time_base.ok_or("No time base")?;
      
      // Get the frame count, handling the `Result<u64, &str>`
      let n_frames = track.codec_params.n_frames.ok_or("No frame count")?;
      
      // Calculate the duration safely after handling the result
      let duration_seconds = (n_frames as f64 * time_base.numer as f64) / time_base.denom as f64;
  
      // Return the duration as a `Duration` object
      Ok(Duration::seconds(duration_seconds as i64))
}

async fn assemble_file(
    temp_dir: &str, 
    file_name: &str, 
    total_chunks: usize, 
    track_id: uuid::Uuid,
    app_state: Arc<AppState>,
) -> std::io::Result<()> {
    let output_path = format!("uploads/{}", file_name);
    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&output_path)?;

    for chunk_number in 0..total_chunks {
        let chunk_path = format!("{}/chunk_{}", temp_dir, chunk_number);
        let chunk_data = fs::read(&chunk_path)?;
        output_file.write_all(&chunk_data)?;
    }

    // Clean up the temporary chunks
    fs::remove_dir_all(temp_dir)?;

    let duration = match get_audio_duration(&output_path) {
        Ok(duration) => duration,
        Err(e) => {
            // Convert the error to std::io::Error and return
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        }
    };

    let duration_seconds = duration.num_seconds(); // Assuming `duration` is of type `Duration`

    app_state.db_client
        .update_status(track_id.clone(), duration_seconds)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}

pub fn upload_handler() -> Router {
    Router::new()
        .route("/", post(upload_chunks))
        .route("/thumbnail", post(upload_thumbnail))
}

pub async fn upload_chunks(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HttpError> {

    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();
    let mut file_name = String::new();
    let mut chunk_number =0;
    let mut total_chunks = 0;
    let mut track_id: Option<uuid::Uuid> = None;
    let mut chunk_data = Vec::new();

    let mut uploaded_chunks = 0;

    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or_default().to_string();
        match field_name.as_str() {
            "fileName" => {
                file_name = field.text().await.unwrap_or_default();
                file_name = sanitize_filename(&file_name);
            }
            "chunkNumber" => {
                chunk_number = field.text().await.unwrap_or_default().parse().unwrap_or(0);
            }
            "totalChunks" => {
                total_chunks = field.text().await.unwrap_or_default().parse().unwrap_or(0);
            }
            "trackId" => {
                let id = field.text().await.unwrap_or_default();
                track_id = Some(uuid::Uuid::parse_str(&id).unwrap());
            }
            "chunk" => {
                match field.bytes().await {
                    Ok(bytes) => chunk_data = bytes.to_vec(),
                    Err(err) => {
                        eprintln!("Error reading chunk data: {:?}", err);
                        return Err(HttpError::bad_request("File Upload failed"));
                    }
                }
            }
            _ => {
                return Err(HttpError::bad_request("File Upload failed 2"));
            }
        }
    }
    
    // Error handling for `next_field` outside the loop
    if let Err(err) = multipart.next_field().await {
        eprintln!("Error processing field: {:?}", err);
        return Err(HttpError::bad_request("Failed to read multipart field"));
    }
    

    if file_name.is_empty() || chunk_data.is_empty() {
        return Err(HttpError::bad_request("File name and chunk data are missing"));
    }

    if let Some(track_id) = track_id {
        if let Some(existing_file) = app_state.db_client.get_audio_file(track_id).await.map_err(|e| HttpError::server_error(e.to_string()))? {
            uploaded_chunks = existing_file.uploaded_chunks; // Get the uploaded_chunks value
        }
    }

    uploaded_chunks += 1;


    if chunk_number == 0 {
        track_id = Some(
            app_state.db_client.upload_file(user_id.clone(), &file_name)
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?,
        );
    }
    let track_id = track_id.ok_or(HttpError::bad_request("track id missing"))?;

    let temp_dir = format!("uploads/temp/{}", &file_name);
    if let Err(_err) = fs::create_dir_all(&temp_dir) {
        return Err(HttpError::server_error("Failed to create temp directory".to_string()));
    }

    let chuck_path = format!("{}/chunk_{}",temp_dir,chunk_number);
    let mut file = match File::create(&chuck_path) {
        Ok(f) => f,
        Err(_err) => {
            return Err(HttpError::server_error("failed to create chuck file"));
        }
    };

    app_state.db_client.upload_chuck(
        track_id.clone(), 
        total_chunks, 
        uploaded_chunks, 
        chunk_number as i32, 
        &chuck_path
    )
    .await
    .map_err(|e| HttpError::server_error(e.to_string()))?;

    if let Err(_err) = file.write_all(&chunk_data) {
        return Err(HttpError::server_error("failed to create chuck file"));
    }

    if is_upload_complete(&temp_dir, total_chunks as usize) {
        if let Err(_err) = assemble_file(&temp_dir, &file_name, total_chunks as usize, track_id.clone(), app_state).await {
            return Err(HttpError::server_error("Failed to complite file".to_string()));
        }
    }

    Ok(Json(UploadResponse{ track_id }))
}


pub async fn upload_thumbnail(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(_user): Extension<JWTAuthMiddleware>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HttpError> {

    // let user = &user.user;
    // let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let mut track_id: Option<uuid::Uuid> = None;
    let mut title = String::new();
    let mut artist = String::new();
    let mut thumbnail_name = String::new();
    let mut thumbnail_data = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or_default().to_string();
        match field_name.as_str() {
            "track_id" => {
                track_id = Some(uuid::Uuid::parse_str(&field.text().await.unwrap()).map_err(|_| {
                    HttpError::bad_request("Invalid track ID format")
                })?);
            }
            "title" => {
                title = field.text().await.unwrap();
            }
            "artist" => {
                artist = field.text().await.unwrap();
            }
            "thumbnail" => {
                thumbnail_name = field.file_name().unwrap_or_default().to_string();
                match field.bytes().await {
                    Ok(bytes) => thumbnail_data = bytes.to_vec(),
                    Err(err) => {
                        eprintln!("Error reading chunk data: {:?}", err);
                        return Err(HttpError::bad_request("File Upload failed"))
                    }
                }
            }
            _ => {
                return Err(HttpError::bad_request("File Upload failed"))
            }
        }
    }

    let track_id = track_id.ok_or(HttpError::bad_request("track id missing"))?;

    if thumbnail_name.is_empty() || thumbnail_data.is_empty() {
        return Err(HttpError::bad_request("Thumbnail is missing"));
    }    

    let thumbnail_dir = format!("assets/images/");
    if let Err(_err) = fs::create_dir_all(&thumbnail_dir) {
        return Err(HttpError::server_error("Createing failed"));
    }

    let thumbnail_file_path = format!("{}/{}", &thumbnail_dir, &thumbnail_name);
    let mut file = match File::create(&thumbnail_file_path) {
        Ok(f) => f,
        Err(err) => {
            return Err(HttpError::server_error(err.to_string()));
        }
    };

    if let Err(_err) = file.write_all(&thumbnail_data) {
        return Err(HttpError::server_error("Createing failed"));
    }

    app_state.db_client
        .upload_thumbnail(track_id.clone(), &thumbnail_name, &title, &artist)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;    

    let response = Response {
        status: "success",
        message: "Thumbnail updated successfull!".to_string(),
    };

    Ok(Json(response))
}