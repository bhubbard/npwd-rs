use npwd_rs::prelude::*;

#[test]
fn test_messages_threads_and_unread_counts() {
    let mut messages_app = MessagesApp::new();

    let conv_id = messages_app.get_or_create_conversation(vec![
        "555-0100".to_string(),
        "555-0155".to_string(),
    ]);

    // Send outgoing message
    let out = messages_app
        .send_message(conv_id, "555-0100", "555-0155", "Hey, are you ready for the job?")
        .unwrap();
    assert_eq!(out.sender, "555-0100");
    assert!(out.is_read);
    assert_eq!(messages_app.total_unread_count(), 0);

    // Receive incoming message
    let inc = messages_app
        .receive_message("555-0155", "555-0100", "Yeah, waiting by the docks.")
        .unwrap();
    assert_eq!(inc.sender, "555-0155");
    assert!(!inc.is_read);
    assert_eq!(messages_app.total_unread_count(), 1);
    assert_eq!(messages_app.metadata().badge_count, 1);

    // Receive another message from a third party
    messages_app
        .receive_message("555-0888", "555-0100", "Check your bank balance.")
        .unwrap();
    assert_eq!(messages_app.total_unread_count(), 2);
    assert_eq!(messages_app.metadata().badge_count, 2);

    // Open first conversation
    messages_app.open_conversation(conv_id).unwrap();
    assert_eq!(messages_app.total_unread_count(), 1); // Only 1 unread remaining
    assert_eq!(messages_app.metadata().badge_count, 1);
}

#[test]
fn test_game_event_receive_sms_routing() {
    let mut phone = npwd_rs::create_default_phone("555-0100", "player_1").unwrap();

    let event = GameEvent::ReceiveSms {
        from: "555-0999".to_string(),
        text: "The package has arrived.".to_string(),
    };

    phone.handle_game_event(event).unwrap();

    // Check Messages badge
    let msg_app = phone
        .app_registry
        .get_concrete::<MessagesApp>("messages")
        .unwrap();
    assert_eq!(msg_app.total_unread_count(), 1);
    assert_eq!(msg_app.metadata().badge_count, 1);

    // Check notification banner
    let banners = phone.event_bus.active_banners();
    assert_eq!(banners.len(), 1);
    assert_eq!(banners[0].title, "New SMS from 555-0999");
    assert_eq!(banners[0].message, "The package has arrived.");
}
