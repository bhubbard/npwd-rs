use crate::app::{AppContext, AppInputEvent, AppMetadata, AppOutputAction, AppPermission, PhoneApp};
use crate::error::{NpwdError, Result};
use crate::events::PhoneEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use uuid::Uuid;

/// 3D Coordinates in the game world.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl WorldPosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// Artistic post-processing filter applied to photo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PhotoFilter {
    #[default]
    Normal,
    Monochrome,
    Vintage,
    Cyberpunk,
    Noir,
    Sunset,
}

/// Metadata and location tracking for a captured in-game photo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhotoMetadata {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub file_name: String,
    pub file_url: String,
    pub position: WorldPosition,
    pub heading: f32,
    pub filter: PhotoFilter,
    pub is_favorite: bool,
    pub caption: Option<String>,
}

/// Camera & Gallery simulated smartphone application.
#[derive(Debug, Clone)]
pub struct CameraApp {
    metadata: AppMetadata,
    photos: HashMap<Uuid, PhotoMetadata>,
}

impl Default for CameraApp {
    fn default() -> Self {
        let metadata = AppMetadata::new("camera", "Camera", "camera-icon")
            .with_permission(AppPermission::Camera)
            .with_permission(AppPermission::Location)
            .with_permission(AppPermission::Storage)
            .system();

        Self {
            metadata,
            photos: HashMap::new(),
        }
    }
}

impl CameraApp {
    pub fn new() -> Self {
        Self::default()
    }

    /// Capture and save a new photo into the phone's gallery.
    pub fn capture_photo(
        &mut self,
        position: WorldPosition,
        heading: f32,
        filter: PhotoFilter,
    ) -> PhotoMetadata {
        let id = Uuid::new_v4();
        let file_name = format!("IMG_{}.jpg", id.simple());
        let file_url = format!("internal://gallery/{}", file_name);

        let photo = PhotoMetadata {
            id,
            timestamp: Utc::now(),
            file_name,
            file_url,
            position,
            heading,
            filter,
            is_favorite: false,
            caption: None,
        };

        self.photos.insert(id, photo.clone());
        photo
    }

    /// Get photo metadata by ID.
    pub fn get_photo(&self, id: Uuid) -> Option<&PhotoMetadata> {
        self.photos.get(&id)
    }

    /// Delete a photo from the album.
    pub fn delete_photo(&mut self, id: Uuid) -> Result<PhotoMetadata> {
        self.photos
            .remove(&id)
            .ok_or_else(|| NpwdError::Internal(format!("Photo {} not found", id)))
    }

    /// Change or re-apply a filter to an existing photo.
    pub fn apply_filter(&mut self, id: Uuid, filter: PhotoFilter) -> Result<()> {
        let photo = self
            .photos
            .get_mut(&id)
            .ok_or_else(|| NpwdError::Internal(format!("Photo {} not found", id)))?;
        photo.filter = filter;
        Ok(())
    }

    /// Toggle favorite status of a photo.
    pub fn toggle_favorite(&mut self, id: Uuid) -> Result<bool> {
        let photo = self
            .photos
            .get_mut(&id)
            .ok_or_else(|| NpwdError::Internal(format!("Photo {} not found", id)))?;
        photo.is_favorite = !photo.is_favorite;
        Ok(photo.is_favorite)
    }

    /// Set or update caption for a photo.
    pub fn set_caption(&mut self, id: Uuid, caption: impl Into<String>) -> Result<()> {
        let photo = self
            .photos
            .get_mut(&id)
            .ok_or_else(|| NpwdError::Internal(format!("Photo {} not found", id)))?;
        photo.caption = Some(caption.into());
        Ok(())
    }

    /// List all photos ordered newest first.
    pub fn gallery(&self) -> Vec<&PhotoMetadata> {
        let mut list: Vec<&PhotoMetadata> = self.photos.values().collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.timestamp));
        list
    }

    /// List favorite photos.
    pub fn favorites(&self) -> Vec<&PhotoMetadata> {
        let mut list: Vec<&PhotoMetadata> = self.photos.values().filter(|p| p.is_favorite).collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.timestamp));
        list
    }

    /// Total photos count.
    pub fn count(&self) -> usize {
        self.photos.len()
    }
}

impl PhoneApp for CameraApp {
    fn metadata(&self) -> &AppMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut AppMetadata {
        &mut self.metadata
    }

    fn on_mount(&mut self, _ctx: &mut AppContext) -> Result<()> {
        Ok(())
    }

    fn on_pause(&mut self, _ctx: &mut AppContext) -> Result<()> {
        Ok(())
    }

    fn on_resume(&mut self, _ctx: &mut AppContext) -> Result<()> {
        Ok(())
    }

    fn on_close(&mut self, _ctx: &mut AppContext) -> Result<()> {
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &AppInputEvent,
        ctx: &mut AppContext,
    ) -> Result<Option<AppOutputAction>> {
        match event {
            AppInputEvent::UiAction { action, payload } if action == "snap" => {
                let x = payload["x"].as_f64().unwrap_or(0.0) as f32;
                let y = payload["y"].as_f64().unwrap_or(0.0) as f32;
                let z = payload["z"].as_f64().unwrap_or(0.0) as f32;
                let heading = payload["heading"].as_f64().unwrap_or(0.0) as f32;
                let filter_str = payload["filter"].as_str().unwrap_or("Normal");

                let filter = match filter_str {
                    "Monochrome" => PhotoFilter::Monochrome,
                    "Vintage" => PhotoFilter::Vintage,
                    "Cyberpunk" => PhotoFilter::Cyberpunk,
                    "Noir" => PhotoFilter::Noir,
                    "Sunset" => PhotoFilter::Sunset,
                    _ => PhotoFilter::Normal,
                };

                let photo = self.capture_photo(WorldPosition::new(x, y, z), heading, filter);
                ctx.emit_phone_event(PhoneEvent::CapturePhoto {
                    position: [x, y, z],
                    heading,
                    filter: filter_str.to_string(),
                });

                Ok(Some(AppOutputAction::Custom(serde_json::to_value(&photo).unwrap())))
            }
            _ => Ok(None),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
