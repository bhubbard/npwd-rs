use crate::app::{AppContext, AppInputEvent, AppMetadata, AppOutputAction, AppPermission, PhoneApp};
use crate::error::{NpwdError, Result};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use uuid::Uuid;

/// In-game contact stored in the phone address book.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    pub id: Uuid,
    pub name: String,
    pub phone_number: String,
    pub avatar_url: Option<String>,
    pub is_favorite: bool,
    pub is_blocked: bool,
    pub note: Option<String>,
    pub tags: Vec<String>,
}

impl Contact {
    pub fn new(name: impl Into<String>, phone_number: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            phone_number: phone_number.into(),
            avatar_url: None,
            is_favorite: false,
            is_blocked: false,
            note: None,
            tags: Vec::new(),
        }
    }

    pub fn with_avatar(mut self, url: impl Into<String>) -> Self {
        self.avatar_url = Some(url.into());
        self
    }

    pub fn favorite(mut self) -> Self {
        self.is_favorite = true;
        self
    }

    pub fn blocked(mut self) -> Self {
        self.is_blocked = true;
        self
    }
}

/// Contacts & Address Book simulated smartphone application.
#[derive(Debug, Clone)]
pub struct ContactsApp {
    metadata: AppMetadata,
    contacts: HashMap<Uuid, Contact>,
}

impl Default for ContactsApp {
    fn default() -> Self {
        let metadata = AppMetadata::new("contacts", "Contacts", "contacts-icon")
            .with_permission(AppPermission::Contacts)
            .system();

        Self {
            metadata,
            contacts: HashMap::new(),
        }
    }
}

impl ContactsApp {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new contact to the address book.
    pub fn add_contact(&mut self, contact: Contact) -> Result<Uuid> {
        let id = contact.id;
        self.contacts.insert(id, contact);
        Ok(id)
    }

    /// Update an existing contact.
    pub fn update_contact(&mut self, contact: Contact) -> Result<()> {
        if !self.contacts.contains_key(&contact.id) {
            return Err(NpwdError::ContactNotFound(contact.name));
        }
        self.contacts.insert(contact.id, contact);
        Ok(())
    }

    /// Delete a contact by ID.
    pub fn delete_contact(&mut self, id: Uuid) -> Result<Contact> {
        self.contacts
            .remove(&id)
            .ok_or_else(|| NpwdError::ContactNotFound(id.to_string()))
    }

    /// Get a contact by ID.
    pub fn get(&self, id: Uuid) -> Option<&Contact> {
        self.contacts.get(&id)
    }

    /// Find a contact by matching phone number.
    pub fn find_by_number(&self, number: &str) -> Option<&Contact> {
        self.contacts.values().find(|c| c.phone_number == number)
    }

    /// Search contacts by name, number, or tag.
    pub fn search(&self, query: &str) -> Vec<&Contact> {
        let q = query.to_lowercase();
        let mut results: Vec<&Contact> = self
            .contacts
            .values()
            .filter(|c| {
                c.name.to_lowercase().contains(&q)
                    || c.phone_number.contains(&q)
                    || c.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect();
        results.sort_by(|a, b| a.name.cmp(&b.name));
        results
    }

    /// Toggle favorite status of a contact.
    pub fn toggle_favorite(&mut self, id: Uuid) -> Result<bool> {
        let contact = self
            .contacts
            .get_mut(&id)
            .ok_or_else(|| NpwdError::ContactNotFound(id.to_string()))?;
        contact.is_favorite = !contact.is_favorite;
        Ok(contact.is_favorite)
    }

    /// Toggle blocked status of a contact.
    pub fn toggle_blocked(&mut self, id: Uuid) -> Result<bool> {
        let contact = self
            .contacts
            .get_mut(&id)
            .ok_or_else(|| NpwdError::ContactNotFound(id.to_string()))?;
        contact.is_blocked = !contact.is_blocked;
        Ok(contact.is_blocked)
    }

    /// Return all favorites.
    pub fn favorites(&self) -> Vec<&Contact> {
        let mut favs: Vec<&Contact> = self.contacts.values().filter(|c| c.is_favorite).collect();
        favs.sort_by(|a, b| a.name.cmp(&b.name));
        favs
    }

    /// Return all blocked contacts.
    pub fn blocked(&self) -> Vec<&Contact> {
        self.contacts.values().filter(|c| c.is_blocked).collect()
    }

    /// List all contacts sorted alphabetically.
    pub fn list_all(&self) -> Vec<&Contact> {
        let mut list: Vec<&Contact> = self.contacts.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    /// Total count of saved contacts.
    pub fn count(&self) -> usize {
        self.contacts.len()
    }
}

impl PhoneApp for ContactsApp {
    fn metadata(&self) -> &AppMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut AppMetadata {
        &mut self.metadata
    }

    fn on_mount(&mut self, _ctx: &mut AppContext) -> Result<()> {
        Ok(())
    }

    fn on_pause(&mut self, _ctx: &mut AppContext) -> Result<()> {
        Ok(())
    }

    fn on_resume(&mut self, _ctx: &mut AppContext) -> Result<()> {
        Ok(())
    }

    fn on_close(&mut self, _ctx: &mut AppContext) -> Result<()> {
        Ok(())
    }

    fn handle_event(
        &mut self,
        _event: &AppInputEvent,
        _ctx: &mut AppContext,
    ) -> Result<Option<AppOutputAction>> {
        Ok(None)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
