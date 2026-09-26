use npwd_rs::prelude::*;

#[test]
fn test_battery_drain_and_charging() {
    let mut battery = Battery::new(100.0);
    assert_eq!(battery.percentage(), 100);

    // Drain while active display is on for 100 seconds
    battery.tick(100.0, true);
    // 100 * 0.05 = 5.0 drained
    assert!((battery.level - 95.0).abs() < 0.01);
    assert_eq!(battery.percentage(), 95);

    // Drain while sleeping for 100 seconds
    battery.tick(100.0, false);
    // 100 * 0.005 = 0.5 drained -> 94.5
    assert!((battery.level - 94.5).abs() < 0.01);

    // Plug in charger
    battery.is_charging = true;
    battery.tick(10.0, true);
    // 10 * 0.3 = 3.0 gained -> 97.5
    assert!((battery.level - 97.5).abs() < 0.01);
}

#[test]
fn test_battery_depletion_auto_shutdown() {
    let mut phone = PhoneOS::new("555-0100", "player_1");
    phone.battery.level = 0.05;

    // Tick enough to deplete battery
    phone.tick(2.0);

    // Should shut down automatically
    assert_eq!(phone.state, PhoneState::PoweredOff);
    assert!(phone.battery.is_depleted());

    // Trying to power on with depleted battery fails
    let err = phone.power_on();
    assert!(matches!(err, Err(NpwdError::BatteryDepleted)));
}

#[test]
fn test_battery_charging_via_game_event() {
    let mut phone = PhoneOS::new("555-0100", "player_1");
    assert!(!phone.battery.is_charging);

    phone
        .handle_game_event(GameEvent::BatteryPowerConnected { is_charging: true })
        .unwrap();
    assert!(phone.battery.is_charging);

    phone
        .handle_game_event(GameEvent::BatteryPowerConnected { is_charging: false })
        .unwrap();
    assert!(!phone.battery.is_charging);
}
