use npwd_rs::prelude::*;

#[test]
fn test_camera_photo_capture_and_gallery() {
    let mut camera = CameraApp::new();

    let p1 = camera.capture_photo(
        WorldPosition::new(-1024.5, 345.2, 14.8),
        180.0,
        PhotoFilter::Cyberpunk,
    );

    let p2 = camera.capture_photo(
        WorldPosition::new(12.0, -45.0, 5.5),
        90.0,
        PhotoFilter::Vintage,
    );

    assert_eq!(camera.count(), 2);
    let gallery = camera.gallery();
    assert_eq!(gallery.len(), 2);
    assert_eq!(gallery[0].id, p2.id); // Newest first

    // Toggle favorite
    assert!(camera.toggle_favorite(p1.id).unwrap());
    assert_eq!(camera.favorites().len(), 1);

    // Filter change
    camera.apply_filter(p1.id, PhotoFilter::Noir).unwrap();
    assert_eq!(camera.get_photo(p1.id).unwrap().filter, PhotoFilter::Noir);
}

#[test]
fn test_marketplace_listings_and_deliveries() {
    let mut market = MarketplaceApp::new();

    let listing = market
        .create_listing(
            "Franklin C.",
            "555-0199",
            "Armored Cognoscenti",
            "Lightly used executive sedan.",
            8_500_000,
            ListingCategory::Vehicles,
            false,
        )
        .unwrap();

    assert_eq!(market.list_active().len(), 1);

    // Search
    assert_eq!(market.search("Cognoscenti", None).len(), 1);
    assert_eq!(
        market.search("Sedan", Some(ListingCategory::Vehicles)).len(),
        1
    );
    assert_eq!(
        market.search("Sedan", Some(ListingCategory::Electronics)).len(),
        0
    );

    // Purchase with 5-second delivery
    let delivery = market
        .purchase_listing(listing.id, "player_1", None, 5.0)
        .unwrap();
    assert_eq!(market.pending_deliveries().len(), 1);
    assert_eq!(delivery.item_name, "Armored Cognoscenti");

    // Tick 3 seconds -> still pending
    let completed = market.tick_deliveries(3.0);
    assert!(completed.is_empty());
    assert_eq!(market.pending_deliveries().len(), 1);

    // Tick 3 more seconds -> completed!
    let completed2 = market.tick_deliveries(3.0);
    assert_eq!(completed2.len(), 1);
    assert_eq!(completed2[0].id, delivery.id);
    assert!(market.pending_deliveries().is_empty());
}

#[test]
fn test_settings_configurations() {
    let mut settings_app = SettingsApp::new();
    assert!(settings_app.settings.dark_mode);

    settings_app.toggle_dark_mode();
    assert!(!settings_app.settings.dark_mode);

    settings_app.set_wallpaper("neon_city");
    assert_eq!(settings_app.settings.wallpaper_id, "neon_city");

    settings_app.set_volume_levels(100, 50, 40);
    assert_eq!(settings_app.settings.volume_media, 100);
    assert_eq!(settings_app.settings.volume_ringtone, 50);
}
