//! # npwd-rs
//! Pure Rust simulated in-game smartphone architecture, OS state machine, apps, and game-world event bridge.
//!
//! Inspired by `project-error/npwd` (FiveM TypeScript/React smartphone framework), re-architected
//! into an idiomatic, deterministic, and modular pure Rust crate suitable for game engines
//! such as Bevy, Fyrox, Macroquad, or standalone server-side roleplay simulations.

pub mod app;
pub mod apps;
pub mod error;
pub mod events;
pub mod os;

pub mod prelude {
    pub use crate::app::{
        AppContext, AppInputEvent, AppMetadata, AppOutputAction, AppPermission, AppRegistry,
        PhoneApp,
    };
    pub use crate::apps::bank::{
        BankApp, Invoice, InvoiceStatus, Transaction, TransactionType,
    };
    pub use crate::apps::camera::{CameraApp, PhotoFilter, PhotoMetadata, WorldPosition};
    pub use crate::apps::contacts::{Contact, ContactsApp};
    pub use crate::apps::marketplace::{
        Delivery, Listing, ListingCategory, ListingStatus, MarketplaceApp,
    };
    pub use crate::apps::messages::{ChatMessage, Conversation, MessagesApp};
    pub use crate::apps::settings::{PhoneSettings, SettingsApp};
    pub use crate::error::{NpwdError, Result};
    pub use crate::events::{
        EventBus, GameEvent, Notification, NotificationPriority, PhoneEvent,
    };
    pub use crate::os::{Battery, LockSecurity, PhoneOS, PhoneState, StatusBarState};
}

use apps::{BankApp, CameraApp, ContactsApp, MarketplaceApp, MessagesApp, SettingsApp};
use error::Result;
use os::PhoneOS;

/// Create a fully provisioned `PhoneOS` instance with all core simulated applications pre-registered.
///
/// Pre-registers:
/// - Contacts (`contacts`)
/// - Messages / SMS (`messages`)
/// - Maze Bank (`bank`)
/// - Camera & Gallery (`camera`)
/// - Marketplace / DarkNet (`marketplace`)
/// - System Settings (`settings`)
pub fn create_default_phone(owner_phone_number: &str, player_id: &str) -> Result<PhoneOS> {
    let mut phone = PhoneOS::new(owner_phone_number, player_id);

    phone.app_registry.register(ContactsApp::new())?;
    phone.app_registry.register(MessagesApp::new())?;
    phone.app_registry.register(BankApp::default())?;
    phone.app_registry.register(CameraApp::new())?;
    phone.app_registry.register(MarketplaceApp::new())?;
    phone.app_registry.register(SettingsApp::new())?;

    Ok(phone)
}
