use crate::error::{NpwdError, Result};
use crate::events::{EventBus, GameEvent, Notification, PhoneEvent};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{HashMap, HashSet};

/// Permissions requested or held by smartphone applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppPermission {
    Camera,
    Contacts,
    Location,
    Banking,
    Network,
    Notifications,
    Storage,
}

/// Metadata describing an installed application in the phone's system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    pub id: String,
    pub display_name: String,
    pub icon: String,
    pub badge_count: u32,
    pub permissions: HashSet<AppPermission>,
    pub is_system_app: bool,
}

impl AppMetadata {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            icon: icon.into(),
            badge_count: 0,
            permissions: HashSet::new(),
            is_system_app: false,
        }
    }

    pub fn with_permission(mut self, perm: AppPermission) -> Self {
        self.permissions.insert(perm);
        self
    }

    pub fn system(mut self) -> Self {
        self.is_system_app = true;
        self
    }

    pub fn with_badge(mut self, badge: u32) -> Self {
        self.badge_count = badge;
        self
    }
}

/// Context provided to applications during lifecycle hooks and event handling.
#[derive(Debug)]
pub struct AppContext<'a> {
    pub owner_number: &'a str,
    pub player_id: &'a str,
    pub battery_percentage: u8,
    pub event_bus: &'a mut EventBus,
}

impl<'a> AppContext<'a> {
    pub fn emit_phone_event(&mut self, event: PhoneEvent) {
        self.event_bus.push_outgoing(event);
    }

    pub fn notify(&mut self, notification: Notification) {
        self.event_bus.post_notification(notification);
    }
}

/// Input events dispatched directly to an application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppInputEvent {
    /// Inbound game world event routed to app.
    GameBridge(GameEvent),
    /// User tapped a button or UI element in the app.
    UiAction {
        action: String,
        payload: serde_json::Value,
    },
    /// Deep link or internal route navigation.
    Navigate {
        route: String,
    },
    /// Hardware/system back key.
    Back,
}

/// Action produced by an app back to the Phone OS.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppOutputAction {
    CloseApp,
    LaunchApp(String),
    SendPhoneEvent(PhoneEvent),
    SetBadge(u32),
    Custom(serde_json::Value),
}

/// Trait defining the lifecycle and capabilities of a simulated phone application.
pub trait PhoneApp: std::fmt::Debug + Send + Sync + 'static {
    /// Return immutable application metadata.
    fn metadata(&self) -> &AppMetadata;

    /// Return mutable application metadata (e.g. to update badge count).
    fn metadata_mut(&mut self) -> &mut AppMetadata;

    /// Invoked when the application is launched into memory.
    fn on_mount(&mut self, ctx: &mut AppContext) -> Result<()>;

    /// Invoked when the user navigates away or switches apps.
    fn on_pause(&mut self, ctx: &mut AppContext) -> Result<()>;

    /// Invoked when the app is brought back to the foreground.
    fn on_resume(&mut self, ctx: &mut AppContext) -> Result<()>;

    /// Invoked when the app is completely terminated from multitasking drawer.
    fn on_close(&mut self, ctx: &mut AppContext) -> Result<()>;

    /// Handle incoming UI actions or game bridge events.
    fn handle_event(
        &mut self,
        event: &AppInputEvent,
        ctx: &mut AppContext,
    ) -> Result<Option<AppOutputAction>>;

    /// Any downcasting helper for concrete access.
    fn as_any(&self) -> &dyn Any;

    /// Mutable Any downcasting helper.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Application Registry managing installed and currently running apps.
#[derive(Default, Debug)]
pub struct AppRegistry {
    apps: HashMap<String, Box<dyn PhoneApp>>,
    running_apps: Vec<String>,
}

impl AppRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new app in the registry.
    pub fn register<T: PhoneApp>(&mut self, app: T) -> Result<()> {
        let id = app.metadata().id.clone();
        if self.apps.contains_key(&id) {
            return Err(NpwdError::AppAlreadyExists(id));
        }
        self.apps.insert(id, Box::new(app));
        Ok(())
    }

    /// Unregister an existing app by ID.
    pub fn unregister(&mut self, id: &str) -> Result<Box<dyn PhoneApp>> {
        self.running_apps.retain(|app_id| app_id != id);
        self.apps
            .remove(id)
            .ok_or_else(|| NpwdError::AppNotFound(id.to_string()))
    }

    /// Check if an app is installed.
    pub fn is_installed(&self, id: &str) -> bool {
        self.apps.contains_key(id)
    }

    /// Retrieve reference to an installed app.
    pub fn get(&self, id: &str) -> Option<&dyn PhoneApp> {
        self.apps.get(id).map(|b| b.as_ref())
    }

    /// Retrieve mutable reference to an installed app.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut dyn PhoneApp> {
        self.apps.get_mut(id).map(|b| b.as_mut())
    }

    /// Retrieve concrete app type via downcasting.
    pub fn get_concrete<T: PhoneApp + 'static>(&self, id: &str) -> Option<&T> {
        self.get(id)?.as_any().downcast_ref::<T>()
    }

    /// Retrieve concrete mutable app type via downcasting.
    pub fn get_concrete_mut<T: PhoneApp + 'static>(&mut self, id: &str) -> Option<&mut T> {
        self.get_mut(id)?.as_any_mut().downcast_mut::<T>()
    }

    /// List metadata for all installed applications.
    pub fn list_installed(&self) -> Vec<&AppMetadata> {
        let mut list: Vec<&AppMetadata> = self.apps.values().map(|app| app.metadata()).collect();
        list.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        list
    }

    /// Return the list of running apps in multitasking order (most recent first).
    pub fn running_apps(&self) -> &[String] {
        &self.running_apps
    }

    /// Launch or switch to an app, triggering lifecycle hooks.
    pub fn launch(&mut self, id: &str, ctx: &mut AppContext) -> Result<()> {
        if !self.apps.contains_key(id) {
            return Err(NpwdError::AppNotFound(id.to_string()));
        }

        let is_running = self.running_apps.contains(&id.to_string());
        if is_running {
            self.running_apps.retain(|app_id| app_id != id);
            self.running_apps.insert(0, id.to_string());
            if let Some(app) = self.apps.get_mut(id) {
                app.on_resume(ctx)?;
            }
        } else {
            self.running_apps.insert(0, id.to_string());
            if let Some(app) = self.apps.get_mut(id) {
                app.on_mount(ctx)?;
            }
        }
        Ok(())
    }

    /// Pause an active app when navigating to home screen or multitasking.
    pub fn pause(&mut self, id: &str, ctx: &mut AppContext) -> Result<()> {
        if let Some(app) = self.apps.get_mut(id) {
            app.on_pause(ctx)?;
        }
        Ok(())
    }

    /// Terminate and close an app completely.
    pub fn close(&mut self, id: &str, ctx: &mut AppContext) -> Result<()> {
        self.running_apps.retain(|app_id| app_id != id);
        if let Some(app) = self.apps.get_mut(id) {
            app.on_close(ctx)?;
        }
        Ok(())
    }
}
