use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Priority level for system and application notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum NotificationPriority {
    Low,
    #[default]
    Normal,
    High,
    Emergency,
}

/// Simulated in-game banner notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub app_id: String,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub banner_duration_remaining: f32,
    pub banner_duration_total: f32,
    pub sound_cue: Option<String>,
    pub is_read: bool,
    pub priority: NotificationPriority,
}

impl Notification {
    pub fn new(app_id: impl Into<String>, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            app_id: app_id.into(),
            title: title.into(),
            message: message.into(),
            timestamp: Utc::now(),
            banner_duration_remaining: 5.0,
            banner_duration_total: 5.0,
            sound_cue: Some("notification_default".to_string()),
            is_read: false,
            priority: NotificationPriority::Normal,
        }
    }

    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_sound(mut self, sound: impl Into<String>) -> Self {
        self.sound_cue = Some(sound.into());
        self
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.banner_duration_remaining = duration;
        self.banner_duration_total = duration;
        self
    }
}

/// Events originating from the outer Game ECS (Bevy / FiveM engine) routed into Phone OS.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameEvent {
    /// Inbound text message from another player or NPC.
    ReceiveSms {
        from: String,
        text: String,
    },
    /// Direct salary or paycheck deposited into player bank account.
    SalaryPaid {
        amount: i64,
        employer: String,
    },
    /// Emergency 911/police/medic dispatch alert.
    DispatchAlert {
        code: String,
        message: String,
        location: Option<[f32; 3]>,
    },
    /// Darknet/hitman bounty posted in marketplace.
    BountyPlaced {
        target_name: String,
        reward: i64,
        description: String,
    },
    /// Inbound cellular voice call.
    IncomingCall {
        call_id: Uuid,
        caller_number: String,
        caller_name: Option<String>,
    },
    /// Call disconnected or rejected by remote party.
    CallEnded {
        call_id: Uuid,
        reason: String,
    },
    /// Cellular network signal strength update (0 to 5 bars).
    CellularSignalChanged {
        bars: u8,
    },
    /// Phone plugged into or unplugged from a charger (car, wall, power bank).
    BatteryPowerConnected {
        is_charging: bool,
    },
    /// In-game clock synchronisation update.
    TimeUpdated {
        formatted_time: String,
    },
    /// Extensible custom game event payload.
    Custom {
        topic: String,
        payload: serde_json::Value,
    },
}

/// Events emitted by the Phone OS back into the outer Game ECS.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PhoneEvent {
    /// Request to initiate a voice call.
    PlaceCall {
        target_number: String,
    },
    /// Answer an incoming voice call.
    AnswerCall {
        call_id: Uuid,
    },
    /// Hang up or decline a voice call.
    EndCall {
        call_id: Uuid,
    },
    /// Outbound SMS sent.
    SendSms {
        to: String,
        text: String,
    },
    /// Direct wire transfer between player accounts.
    SendMoney {
        to_account: String,
        amount: i64,
        memo: String,
    },
    /// Emergency call to 911 dispatch with coordinates.
    Dial911 {
        emergency_type: String,
        message: String,
        coordinates: Option<[f32; 3]>,
    },
    /// Item purchased on the marketplace / darknet.
    PurchaseItem {
        listing_id: Uuid,
        price: i64,
    },
    /// Photo captured via camera app.
    CapturePhoto {
        position: [f32; 3],
        heading: f32,
        filter: String,
    },
    /// App launched into active foreground.
    AppOpened {
        app_id: String,
    },
    /// App closed or minimized to background.
    AppClosed {
        app_id: String,
    },
    /// Phone screen locked.
    ScreenLocked,
    /// Phone screen successfully unlocked.
    ScreenUnlocked,
    /// Power button toggled state.
    PowerChanged {
        powered_on: bool,
    },
    /// Notification banner was dismissed by the user.
    NotificationDismissed {
        notification_id: Uuid,
    },
}

/// Central Event Bus bridging Game ECS and Phone OS message channels.
#[derive(Debug, Clone, Default)]
pub struct EventBus {
    incoming_queue: VecDeque<GameEvent>,
    outgoing_queue: VecDeque<PhoneEvent>,
    notifications: Vec<Notification>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push an event from the Game ECS into the phone's inbound queue.
    pub fn push_incoming(&mut self, event: GameEvent) {
        self.incoming_queue.push_back(event);
    }

    /// Retrieve the next inbound game event.
    pub fn pop_incoming(&mut self) -> Option<GameEvent> {
        self.incoming_queue.pop_front()
    }

    /// Queue an outbound event from the phone to the Game ECS.
    pub fn push_outgoing(&mut self, event: PhoneEvent) {
        self.outgoing_queue.push_back(event);
    }

    /// Drain all pending outbound events to forward to Game ECS (e.g. Bevy systems).
    pub fn drain_outgoing(&mut self) -> Vec<PhoneEvent> {
        self.outgoing_queue.drain(..).collect()
    }

    /// Post a new notification to the phone's notification drawer & active banner queue.
    pub fn post_notification(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    /// Read all notifications.
    pub fn notifications(&self) -> &[Notification] {
        &self.notifications
    }

    /// Read all active banner notifications (where countdown > 0).
    pub fn active_banners(&self) -> Vec<&Notification> {
        self.notifications
            .iter()
            .filter(|n| n.banner_duration_remaining > 0.0)
            .collect()
    }

    /// Advance notification banner countdowns.
    pub fn tick(&mut self, delta_seconds: f32) {
        for n in &mut self.notifications {
            if n.banner_duration_remaining > 0.0 {
                n.banner_duration_remaining = (n.banner_duration_remaining - delta_seconds).max(0.0);
            }
        }
    }

    /// Mark a notification as read and clear its banner timer.
    pub fn dismiss_notification(&mut self, id: Uuid) {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.is_read = true;
            n.banner_duration_remaining = 0.0;
            self.push_outgoing(PhoneEvent::NotificationDismissed { notification_id: id });
        }
    }

    /// Clear all read notifications.
    pub fn clear_read_notifications(&mut self) {
        self.notifications.retain(|n| !n.is_read);
    }
}
