use crate::app::{AppContext, AppInputEvent, AppRegistry};
use crate::error::{NpwdError, Result};
use crate::events::{EventBus, GameEvent, Notification, NotificationPriority, PhoneEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Security mechanism configured for the smartphone lock screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockSecurity {
    None,
    Pin(String),
    Biometrics { enrolled: bool },
}

/// Simulated hardware battery state and power simulation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Battery {
    pub level: f32,
    pub is_charging: bool,
    /// Percentage drained per second while active / display is on.
    pub discharge_rate_active: f32,
    /// Percentage drained per second while sleeping / display is off.
    pub discharge_rate_sleep: f32,
    /// Percentage gained per second while connected to power.
    pub charge_rate: f32,
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            level: 85.0,
            is_charging: false,
            discharge_rate_active: 0.05, // ~3% per minute
            discharge_rate_sleep: 0.005, // ~0.3% per minute
            charge_rate: 0.3,            // ~18% per minute
        }
    }
}

impl Battery {
    pub fn new(initial_level: f32) -> Self {
        Self {
            level: initial_level.clamp(0.0, 100.0),
            ..Default::default()
        }
    }

    /// Advance battery simulation by delta time in seconds.
    pub fn tick(&mut self, delta_seconds: f32, is_display_on: bool) {
        if self.is_charging {
            self.level = (self.level + self.charge_rate * delta_seconds).min(100.0);
        } else {
            let rate = if is_display_on {
                self.discharge_rate_active
            } else {
                self.discharge_rate_sleep
            };
            self.level = (self.level - rate * delta_seconds).max(0.0);
        }
    }

    /// Rounded battery percentage (0 to 100).
    pub fn percentage(&self) -> u8 {
        self.level.round() as u8
    }

    /// Check if battery is fully depleted.
    pub fn is_depleted(&self) -> bool {
        self.level <= 0.0
    }
}

/// Dynamic status bar state displayed on top of the phone screen.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusBarState {
    pub time_formatted: String,
    pub battery_percentage: u8,
    pub cellular_signal_bars: u8,
    pub wifi_connected: bool,
    pub wifi_ssid: Option<String>,
    pub do_not_disturb: bool,
    pub airplane_mode: bool,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            time_formatted: "12:00".to_string(),
            battery_percentage: 85,
            cellular_signal_bars: 4,
            wifi_connected: true,
            wifi_ssid: Some("LosSantos-5G".to_string()),
            do_not_disturb: false,
            airplane_mode: false,
        }
    }
}

/// The core operating system state machine governing the smartphone screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PhoneState {
    PoweredOff,
    Booting {
        progress_ticks: u32,
        total_ticks: u32,
    },
    LockScreen {
        security: LockSecurity,
        is_unlocked: bool,
    },
    HomeScreen,
    AppActive {
        app_id: String,
    },
    MultitaskingDrawer,
    IncomingCall {
        call_id: Uuid,
        caller_number: String,
        caller_name: Option<String>,
    },
    ActiveCall {
        call_id: Uuid,
        contact_number: String,
        contact_name: Option<String>,
        duration_seconds: u32,
        is_muted: bool,
        speaker_on: bool,
    },
}

/// Pure Rust Simulated Smartphone Operating System.
pub struct PhoneOS {
    pub state: PhoneState,
    previous_state: Option<PhoneState>,
    pub status_bar: StatusBarState,
    pub battery: Battery,
    pub app_registry: AppRegistry,
    pub event_bus: EventBus,
    pub owner_phone_number: String,
    pub player_id: String,
    pub lock_security: LockSecurity,
}

impl PhoneOS {
    pub fn new(owner_phone_number: impl Into<String>, player_id: impl Into<String>) -> Self {
        Self {
            state: PhoneState::HomeScreen,
            previous_state: None,
            status_bar: StatusBarState::default(),
            battery: Battery::default(),
            app_registry: AppRegistry::new(),
            event_bus: EventBus::new(),
            owner_phone_number: owner_phone_number.into(),
            player_id: player_id.into(),
            lock_security: LockSecurity::None,
        }
    }

    /// Construct a phone starting in powered off state.
    pub fn powered_off(owner_phone_number: impl Into<String>, player_id: impl Into<String>) -> Self {
        let mut phone = Self::new(owner_phone_number, player_id);
        phone.state = PhoneState::PoweredOff;
        phone
    }

    /// Set lock screen security mode (PIN, Biometrics, or None).
    pub fn set_lock_security(&mut self, security: LockSecurity) {
        self.lock_security = security;
    }

    /// Press hardware power button.
    pub fn power_button_press(&mut self) -> Result<()> {
        match &self.state {
            PhoneState::PoweredOff => {
                self.power_on()?;
            }
            PhoneState::Booting { .. } => {
                // Ignore button press while booting
            }
            PhoneState::LockScreen { .. } => {
                // Put display to sleep or wake up
            }
            _ => {
                self.lock_screen()?;
            }
        }
        Ok(())
    }

    /// Power on the smartphone and begin boot sequence.
    pub fn power_on(&mut self) -> Result<()> {
        if self.battery.is_depleted() {
            return Err(NpwdError::BatteryDepleted);
        }
        self.state = PhoneState::Booting {
            progress_ticks: 0,
            total_ticks: 3,
        };
        self.event_bus
            .push_outgoing(PhoneEvent::PowerChanged { powered_on: true });
        Ok(())
    }

    /// Force power off the smartphone.
    pub fn power_off(&mut self) -> Result<()> {
        self.state = PhoneState::PoweredOff;
        self.event_bus
            .push_outgoing(PhoneEvent::PowerChanged { powered_on: false });
        Ok(())
    }

    /// Advance boot sequence by 1 tick; automatically transitions to lock/home screen when ready.
    pub fn boot_tick(&mut self) -> Result<bool> {
        if let PhoneState::Booting {
            progress_ticks,
            total_ticks,
        } = &mut self.state
        {
            *progress_ticks += 1;
            if *progress_ticks >= *total_ticks {
                if self.lock_security != LockSecurity::None {
                    self.state = PhoneState::LockScreen {
                        security: self.lock_security.clone(),
                        is_unlocked: false,
                    };
                } else {
                    self.state = PhoneState::HomeScreen;
                }
                return Ok(true);
            }
            return Ok(false);
        }
        Ok(false)
    }

    /// Lock the phone screen.
    pub fn lock_screen(&mut self) -> Result<()> {
        if self.state == PhoneState::PoweredOff {
            return Err(NpwdError::DevicePoweredOff);
        }
        self.state = PhoneState::LockScreen {
            security: self.lock_security.clone(),
            is_unlocked: false,
        };
        self.event_bus.push_outgoing(PhoneEvent::ScreenLocked);
        Ok(())
    }

    /// Attempt to unlock lock screen with PIN code.
    pub fn unlock_with_pin(&mut self, pin: &str) -> Result<()> {
        match &self.state {
            PhoneState::LockScreen { security, .. } => match security {
                LockSecurity::Pin(expected) => {
                    if pin == expected {
                        self.state = PhoneState::HomeScreen;
                        self.event_bus.push_outgoing(PhoneEvent::ScreenUnlocked);
                        Ok(())
                    } else {
                        Err(NpwdError::AuthenticationFailed("Invalid PIN".to_string()))
                    }
                }
                LockSecurity::None => {
                    self.state = PhoneState::HomeScreen;
                    self.event_bus.push_outgoing(PhoneEvent::ScreenUnlocked);
                    Ok(())
                }
                _ => Err(NpwdError::AuthenticationFailed(
                    "PIN security not configured".to_string(),
                )),
            },
            _ => Err(NpwdError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "HomeScreen".to_string(),
                reason: "Device is not currently on lock screen".to_string(),
            }),
        }
    }

    /// Attempt to unlock lock screen using fingerprint or biometrics.
    pub fn unlock_with_biometrics(&mut self) -> Result<()> {
        match &self.state {
            PhoneState::LockScreen { security, .. } => match security {
                LockSecurity::Biometrics { enrolled } if *enrolled => {
                    self.state = PhoneState::HomeScreen;
                    self.event_bus.push_outgoing(PhoneEvent::ScreenUnlocked);
                    Ok(())
                }
                LockSecurity::None => {
                    self.state = PhoneState::HomeScreen;
                    self.event_bus.push_outgoing(PhoneEvent::ScreenUnlocked);
                    Ok(())
                }
                _ => Err(NpwdError::AuthenticationFailed(
                    "Biometric authentication failed or not enrolled".to_string(),
                )),
            },
            _ => Err(NpwdError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "HomeScreen".to_string(),
                reason: "Device is not currently on lock screen".to_string(),
            }),
        }
    }

    /// Press hardware or virtual home button.
    pub fn press_home_button(&mut self) -> Result<()> {
        if self.state == PhoneState::PoweredOff {
            return Err(NpwdError::DevicePoweredOff);
        }

        if let PhoneState::AppActive { ref app_id } = self.state {
            let app_id_clone = app_id.clone();
            let mut ctx = AppContext {
                owner_number: &self.owner_phone_number,
                player_id: &self.player_id,
                battery_percentage: self.status_bar.battery_percentage,
                event_bus: &mut self.event_bus,
            };
            self.app_registry.pause(&app_id_clone, &mut ctx)?;
            self.event_bus
                .push_outgoing(PhoneEvent::AppClosed { app_id: app_id_clone });
        }

        self.state = PhoneState::HomeScreen;
        Ok(())
    }

    /// Open multitasking drawer showing recent active apps.
    pub fn open_multitasking(&mut self) -> Result<()> {
        if self.state == PhoneState::PoweredOff {
            return Err(NpwdError::DevicePoweredOff);
        }
        self.state = PhoneState::MultitaskingDrawer;
        Ok(())
    }

    /// Launch or switch to an application by app_id.
    pub fn launch_app(&mut self, app_id: &str) -> Result<()> {
        if self.state == PhoneState::PoweredOff {
            return Err(NpwdError::DevicePoweredOff);
        }

        if let PhoneState::LockScreen { is_unlocked, .. } = self.state
            && !is_unlocked
        {
            return Err(NpwdError::DeviceLocked);
        }

        if let PhoneState::AppActive { app_id: ref current } = self.state
            && current != app_id
        {
            let current_clone = current.clone();
            let mut ctx = AppContext {
                owner_number: &self.owner_phone_number,
                player_id: &self.player_id,
                battery_percentage: self.status_bar.battery_percentage,
                event_bus: &mut self.event_bus,
            };
            self.app_registry.pause(&current_clone, &mut ctx)?;
        }

        let mut ctx = AppContext {
            owner_number: &self.owner_phone_number,
            player_id: &self.player_id,
            battery_percentage: self.status_bar.battery_percentage,
            event_bus: &mut self.event_bus,
        };
        self.app_registry.launch(app_id, &mut ctx)?;
        self.state = PhoneState::AppActive {
            app_id: app_id.to_string(),
        };

        self.event_bus
            .push_outgoing(PhoneEvent::AppOpened { app_id: app_id.to_string() });
        Ok(())
    }

    /// Close currently active foreground app and return to home screen.
    pub fn close_active_app(&mut self) -> Result<()> {
        if let PhoneState::AppActive { ref app_id } = self.state {
            let app_id_clone = app_id.clone();
            let mut ctx = AppContext {
                owner_number: &self.owner_phone_number,
                player_id: &self.player_id,
                battery_percentage: self.status_bar.battery_percentage,
                event_bus: &mut self.event_bus,
            };
            self.app_registry.close(&app_id_clone, &mut ctx)?;
            self.event_bus
                .push_outgoing(PhoneEvent::AppClosed { app_id: app_id_clone });
            self.state = PhoneState::HomeScreen;
        }
        Ok(())
    }

    /// Trigger an incoming cellular voice call.
    pub fn trigger_incoming_call(
        &mut self,
        call_id: Uuid,
        caller_number: impl Into<String>,
        caller_name: Option<String>,
    ) {
        let caller_num = caller_number.into();
        self.previous_state = Some(self.state.clone());
        self.state = PhoneState::IncomingCall {
            call_id,
            caller_number: caller_num.clone(),
            caller_name: caller_name.clone(),
        };

        self.event_bus.post_notification(
            Notification::new(
                "phone",
                "Incoming Call",
                format!("Call from {}", caller_name.unwrap_or(caller_num)),
            )
            .with_priority(NotificationPriority::High)
            .with_sound("ringtone_default"),
        );
    }

    /// Answer the current incoming call.
    pub fn answer_call(&mut self) -> Result<()> {
        if let PhoneState::IncomingCall {
            call_id,
            caller_number,
            caller_name,
        } = self.state.clone()
        {
            self.state = PhoneState::ActiveCall {
                call_id,
                contact_number: caller_number,
                contact_name: caller_name,
                duration_seconds: 0,
                is_muted: false,
                speaker_on: false,
            };
            self.event_bus.push_outgoing(PhoneEvent::AnswerCall { call_id });
            Ok(())
        } else {
            Err(NpwdError::Internal("No incoming call to answer".to_string()))
        }
    }

    /// Decline incoming call or terminate active ongoing call.
    pub fn reject_or_end_call(&mut self) -> Result<()> {
        match &self.state {
            PhoneState::IncomingCall { call_id, .. } => {
                let id = *call_id;
                self.event_bus.push_outgoing(PhoneEvent::EndCall { call_id: id });
                self.restore_previous_state();
                Ok(())
            }
            PhoneState::ActiveCall { call_id, .. } => {
                let id = *call_id;
                self.event_bus.push_outgoing(PhoneEvent::EndCall { call_id: id });
                self.restore_previous_state();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Toggle microphone mute during active call.
    pub fn toggle_call_mute(&mut self) -> Result<bool> {
        if let PhoneState::ActiveCall { ref mut is_muted, .. } = self.state {
            *is_muted = !*is_muted;
            Ok(*is_muted)
        } else {
            Err(NpwdError::Internal("Not in an active call".to_string()))
        }
    }

    /// Toggle speakerphone during active call.
    pub fn toggle_call_speaker(&mut self) -> Result<bool> {
        if let PhoneState::ActiveCall { ref mut speaker_on, .. } = self.state {
            *speaker_on = !*speaker_on;
            Ok(*speaker_on)
        } else {
            Err(NpwdError::Internal("Not in an active call".to_string()))
        }
    }

    fn restore_previous_state(&mut self) {
        if let Some(prev) = self.previous_state.take() {
            self.state = prev;
        } else {
            self.state = PhoneState::HomeScreen;
        }
    }

    /// Handle inbound game world event from Bevy or Game ECS.
    pub fn handle_game_event(&mut self, event: GameEvent) -> Result<()> {
        match &event {
            GameEvent::IncomingCall {
                call_id,
                caller_number,
                caller_name,
            } => {
                self.trigger_incoming_call(*call_id, caller_number, caller_name.clone());
            }
            GameEvent::CallEnded { call_id, .. } => {
                if let PhoneState::ActiveCall { call_id: active, .. } = self.state {
                    if active == *call_id {
                        self.restore_previous_state();
                    }
                } else if let PhoneState::IncomingCall { call_id: incoming, .. } = self.state
                    && incoming == *call_id
                {
                    self.restore_previous_state();
                }
            }
            GameEvent::CellularSignalChanged { bars } => {
                self.status_bar.cellular_signal_bars = *bars;
            }
            GameEvent::BatteryPowerConnected { is_charging } => {
                self.battery.is_charging = *is_charging;
            }
            GameEvent::TimeUpdated { formatted_time } => {
                self.status_bar.time_formatted = formatted_time.clone();
            }
            _ => {}
        }

        // Route to installed apps if relevant
        let app_ids: Vec<String> = self.app_registry.list_installed().into_iter().map(|m| m.id.clone()).collect();
        for id in app_ids {
            if let Some(app) = self.app_registry.get_mut(&id) {
                let mut ctx = AppContext {
                    owner_number: &self.owner_phone_number,
                    player_id: &self.player_id,
                    battery_percentage: self.status_bar.battery_percentage,
                    event_bus: &mut self.event_bus,
                };
                let _ = app.handle_event(&AppInputEvent::GameBridge(event.clone()), &mut ctx);
            }
        }

        Ok(())
    }

    /// Advance phone clock, battery, delivery timers, and active calls.
    pub fn tick(&mut self, delta_seconds: f32) {
        if self.state == PhoneState::PoweredOff {
            return;
        }

        let is_display_on = !matches!(self.state, PhoneState::PoweredOff);
        self.battery.tick(delta_seconds, is_display_on);
        self.status_bar.battery_percentage = self.battery.percentage();

        if self.battery.is_depleted() {
            let _ = self.power_off();
            return;
        }

        // Advance active call duration
        if let PhoneState::ActiveCall {
            ref mut duration_seconds,
            ..
        } = self.state
        {
            *duration_seconds += delta_seconds.round() as u32;
        }

        // Advance event bus banners
        self.event_bus.tick(delta_seconds);

        // Process any queued incoming game events
        while let Some(event) = self.event_bus.pop_incoming() {
            let _ = self.handle_game_event(event);
        }
    }

    /// Helper to create AppContext borrowing self.
    pub fn create_app_context(&mut self) -> AppContext<'_> {
        AppContext {
            owner_number: &self.owner_phone_number,
            player_id: &self.player_id,
            battery_percentage: self.status_bar.battery_percentage,
            event_bus: &mut self.event_bus,
        }
    }
}
