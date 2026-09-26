use crate::app::{AppContext, AppInputEvent, AppMetadata, AppOutputAction, PhoneApp};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Audio, visual, and operational preferences of the smartphone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhoneSettings {
    pub wallpaper_id: String,
    pub ringtone_id: String,
    pub vibration_enabled: bool,
    pub dark_mode: bool,
    pub mute: bool,
    pub volume_media: u8,
    pub volume_ringtone: u8,
    pub volume_notifications: u8,
    pub auto_lock_seconds: u32,
    pub airplane_mode: bool,
    pub do_not_disturb: bool,
    pub bluetooth_enabled: bool,
}

impl Default for PhoneSettings {
    fn default() -> Self {
        Self {
            wallpaper_id: "default_gradient".to_string(),
            ringtone_id: "synth_wave".to_string(),
            vibration_enabled: true,
            dark_mode: true,
            mute: false,
            volume_media: 80,
            volume_ringtone: 85,
            volume_notifications: 75,
            auto_lock_seconds: 60,
            airplane_mode: false,
            do_not_disturb: false,
            bluetooth_enabled: true,
        }
    }
}

/// Settings configuration application.
#[derive(Debug, Clone)]
pub struct SettingsApp {
    metadata: AppMetadata,
    pub settings: PhoneSettings,
}

impl Default for SettingsApp {
    fn default() -> Self {
        let metadata = AppMetadata::new("settings", "Settings", "settings-icon").system();
        Self {
            metadata,
            settings: PhoneSettings::default(),
        }
    }
}

impl SettingsApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_wallpaper(&mut self, wallpaper: impl Into<String>) {
        self.settings.wallpaper_id = wallpaper.into();
    }

    pub fn set_ringtone(&mut self, ringtone: impl Into<String>) {
        self.settings.ringtone_id = ringtone.into();
    }

    pub fn toggle_dark_mode(&mut self) -> bool {
        self.settings.dark_mode = !self.settings.dark_mode;
        self.settings.dark_mode
    }

    pub fn toggle_vibration(&mut self) -> bool {
        self.settings.vibration_enabled = !self.settings.vibration_enabled;
        self.settings.vibration_enabled
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.settings.mute = !self.settings.mute;
        self.settings.mute
    }

    pub fn set_volume_levels(&mut self, media: u8, ringtone: u8, notifications: u8) {
        self.settings.volume_media = media.min(100);
        self.settings.volume_ringtone = ringtone.min(100);
        self.settings.volume_notifications = notifications.min(100);
    }
}

impl PhoneApp for SettingsApp {
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
        _event: &AppInputEvent,
        _ctx: &mut AppContext,
    ) -> Result<Option<AppOutputAction>> {
        Ok(None)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
