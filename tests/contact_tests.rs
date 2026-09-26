use npwd_rs::prelude::*;

#[test]
fn test_contacts_crud_and_search() {
    let mut contacts_app = ContactsApp::new();

    let c1 = Contact::new("Lester Crest", "555-0144")
        .with_avatar("avatar_lester.png")
        .favorite();
    let c2 = Contact::new("Trevor Philips", "555-0199").blocked();
    let mut c3 = Contact::new("Michael De Santa", "555-0188");
    c3.tags.push("Heist".to_string());

    let id1 = contacts_app.add_contact(c1).unwrap();
    let id2 = contacts_app.add_contact(c2).unwrap();
    let id3 = contacts_app.add_contact(c3).unwrap();

    assert_eq!(contacts_app.count(), 3);

    // Get and find
    assert_eq!(contacts_app.get(id1).unwrap().name, "Lester Crest");
    assert_eq!(
        contacts_app.find_by_number("555-0199").unwrap().name,
        "Trevor Philips"
    );

    // Search
    let results = contacts_app.search("Lester");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id1);

    let heist_search = contacts_app.search("heist");
    assert_eq!(heist_search.len(), 1);
    assert_eq!(heist_search[0].id, id3);

    let number_search = contacts_app.search("0199");
    assert_eq!(number_search.len(), 1);
    assert_eq!(number_search[0].id, id2);

    // Favorites & Blocked
    assert_eq!(contacts_app.favorites().len(), 1);
    assert_eq!(contacts_app.blocked().len(), 1);

    // Toggle favorite
    assert!(contacts_app.toggle_favorite(id3).unwrap());
    assert_eq!(contacts_app.favorites().len(), 2);

    // Delete contact
    contacts_app.delete_contact(id2).unwrap();
    assert_eq!(contacts_app.count(), 2);
    assert!(contacts_app.get(id2).is_none());
}
