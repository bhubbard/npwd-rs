use npwd_rs::prelude::*;

#[test]
fn test_event_bus_banners_and_drain() {
    let mut bus = EventBus::new();

    let notif = Notification::new("system", "Warning", "Severe weather alert")
        .with_priority(NotificationPriority::High)
        .with_duration(4.0);

    let notif_id = notif.id;
    bus.post_notification(notif);

    assert_eq!(bus.active_banners().len(), 1);

    // Tick 2 seconds -> banner still active (2s left)
    bus.tick(2.0);
    assert_eq!(bus.active_banners().len(), 1);
    assert!((bus.active_banners()[0].banner_duration_remaining - 2.0).abs() < 0.01);

    // Dismiss manually
    bus.dismiss_notification(notif_id);
    assert!(bus.active_banners().is_empty());

    let outgoing = bus.drain_outgoing();
    assert_eq!(outgoing.len(), 1);
    assert_eq!(
        outgoing[0],
        PhoneEvent::NotificationDismissed {
            notification_id: notif_id
        }
    );
}

#[test]
fn test_bevy_ecs_message_loop_pattern() {
    let mut phone = npwd_rs::create_default_phone("555-0100", "player_1").unwrap();

    // Simulated frame tick loop
    let incoming_frame_events = vec![
        GameEvent::CellularSignalChanged { bars: 5 },
        GameEvent::TimeUpdated {
            formatted_time: "14:45".to_string(),
        },
        GameEvent::SalaryPaid {
            amount: 75_000,
            employer: "Dynasty8 Real Estate".to_string(),
        },
    ];

    // Push frame events
    for ev in incoming_frame_events {
        phone.event_bus.push_incoming(ev);
    }

    // Tick 1 frame (e.g. 1/60s = 0.0166s)
    phone.tick(0.0166);

    // Validate states updated
    assert_eq!(phone.status_bar.cellular_signal_bars, 5);
    assert_eq!(phone.status_bar.time_formatted, "14:45");

    let bank = phone.app_registry.get_concrete::<BankApp>("bank").unwrap();
    assert_eq!(bank.balance(), 500_000 + 75_000);
}
