use npwd_rs::prelude::*;
use uuid::Uuid;

#[test]
fn test_power_cycle_and_boot_sequence() {
    let mut phone = PhoneOS::powered_off("555-0100", "player_1");
    assert_eq!(phone.state, PhoneState::PoweredOff);

    // Cannot lock or launch when powered off
    assert!(phone.lock_screen().is_err());
    assert!(phone.launch_app("contacts").is_err());

    // Power on starts boot sequence
    phone.power_on().expect("Should initiate power on");
    assert!(matches!(phone.state, PhoneState::Booting { .. }));

    // Advance boot ticks
    assert!(!phone.boot_tick().unwrap());
    assert!(!phone.boot_tick().unwrap());
    assert!(phone.boot_tick().unwrap()); // 3rd tick completes boot

    // With no PIN set, boots straight to HomeScreen
    assert_eq!(phone.state, PhoneState::HomeScreen);

    // Power off restores powered off state
    phone.power_off().expect("Should power off");
    assert_eq!(phone.state, PhoneState::PoweredOff);
}

#[test]
fn test_lock_screen_with_pin() {
    let mut phone = PhoneOS::new("555-0100", "player_1");
    phone.set_lock_security(LockSecurity::Pin("1337".to_string()));

    phone.lock_screen().expect("Should lock screen");
    assert!(matches!(phone.state, PhoneState::LockScreen { .. }));

    // Failed PIN attempt
    let bad_res = phone.unlock_with_pin("0000");
    assert!(bad_res.is_err());
    assert!(matches!(phone.state, PhoneState::LockScreen { .. }));

    // Successful PIN unlock
    phone.unlock_with_pin("1337").expect("Should unlock with valid PIN");
    assert_eq!(phone.state, PhoneState::HomeScreen);

    // Event bus recorded unlock
    let outgoing = phone.event_bus.drain_outgoing();
    assert!(outgoing.contains(&PhoneEvent::ScreenUnlocked));
}

#[test]
fn test_lock_screen_with_biometrics() {
    let mut phone = PhoneOS::new("555-0100", "player_1");
    phone.set_lock_security(LockSecurity::Biometrics { enrolled: true });

    phone.lock_screen().unwrap();
    phone.unlock_with_biometrics().expect("Biometric unlock should succeed");
    assert_eq!(phone.state, PhoneState::HomeScreen);
}

#[test]
fn test_app_lifecycle_and_multitasking() {
    let mut phone = npwd_rs::create_default_phone("555-0100", "player_1").unwrap();

    // Launch Contacts
    phone.launch_app("contacts").expect("Should launch contacts");
    assert_eq!(
        phone.state,
        PhoneState::AppActive {
            app_id: "contacts".to_string()
        }
    );
    assert_eq!(phone.app_registry.running_apps(), &["contacts"]);

    // Launch Messages (should pause Contacts and bring Messages to front)
    phone.launch_app("messages").expect("Should switch to messages");
    assert_eq!(
        phone.state,
        PhoneState::AppActive {
            app_id: "messages".to_string()
        }
    );
    assert_eq!(
        phone.app_registry.running_apps(),
        &["messages".to_string(), "contacts".to_string()]
    );

    // Press home button
    phone.press_home_button().expect("Should return to home");
    assert_eq!(phone.state, PhoneState::HomeScreen);

    // Open multitasking drawer
    phone.open_multitasking().expect("Should open multitasking drawer");
    assert_eq!(phone.state, PhoneState::MultitaskingDrawer);

    // Resume contacts from multitasking
    phone.launch_app("contacts").expect("Should resume contacts");
    assert_eq!(
        phone.state,
        PhoneState::AppActive {
            app_id: "contacts".to_string()
        }
    );
    assert_eq!(
        phone.app_registry.running_apps(),
        &["contacts".to_string(), "messages".to_string()]
    );

    // Close active app completely
    phone.close_active_app().expect("Should close contacts");
    assert_eq!(phone.state, PhoneState::HomeScreen);
    assert_eq!(phone.app_registry.running_apps(), &["messages".to_string()]);
}

#[test]
fn test_call_lifecycle_and_state_restoration() {
    let mut phone = npwd_rs::create_default_phone("555-0100", "player_1").unwrap();
    phone.launch_app("bank").unwrap();

    let call_id = Uuid::new_v4();
    phone.trigger_incoming_call(call_id, "555-9999", Some("Franklin".to_string()));

    assert!(matches!(phone.state, PhoneState::IncomingCall { .. }));

    // Answer call
    phone.answer_call().expect("Should answer call");
    if let PhoneState::ActiveCall {
        contact_name,
        is_muted,
        speaker_on,
        ..
    } = &phone.state
    {
        assert_eq!(contact_name.as_deref(), Some("Franklin"));
        assert!(!is_muted);
        assert!(!speaker_on);
    } else {
        panic!("Expected ActiveCall state");
    }

    // Toggle mute and speaker
    assert!(phone.toggle_call_mute().unwrap());
    assert!(phone.toggle_call_speaker().unwrap());

    // Advance time during call
    phone.tick(10.0);
    if let PhoneState::ActiveCall { duration_seconds, .. } = phone.state {
        assert_eq!(duration_seconds, 10);
    }

    // End call -> should restore previous state (bank app)
    phone.reject_or_end_call().expect("Should end call");
    assert_eq!(
        phone.state,
        PhoneState::AppActive {
            app_id: "bank".to_string()
        }
    );

    let outgoing = phone.event_bus.drain_outgoing();
    assert!(outgoing.contains(&PhoneEvent::EndCall { call_id }));
}
