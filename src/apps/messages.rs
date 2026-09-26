use crate::app::{AppContext, AppInputEvent, AppMetadata, AppOutputAction, AppPermission, PhoneApp};
use crate::error::{NpwdError, Result};
use crate::events::{GameEvent, Notification, PhoneEvent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use uuid::Uuid;

/// Single SMS or MMS text message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub is_read: bool,
    pub attachments: Vec<String>,
}

/// Conversation thread grouping messages between participants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub participant_numbers: Vec<String>,
    pub messages: Vec<ChatMessage>,
    pub last_message_preview: String,
    pub last_activity: DateTime<Utc>,
    pub unread_count: u32,
}

impl Conversation {
    pub fn new(participants: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            participant_numbers: participants,
            messages: Vec::new(),
            last_message_preview: String::new(),
            last_activity: Utc::now(),
            unread_count: 0,
        }
    }
}

/// Messages / SMS application managing conversation threads, read receipts, and badges.
#[derive(Debug, Clone)]
pub struct MessagesApp {
    metadata: AppMetadata,
    conversations: HashMap<Uuid, Conversation>,
    active_conversation: Option<Uuid>,
}

impl Default for MessagesApp {
    fn default() -> Self {
        let metadata = AppMetadata::new("messages", "Messages", "messages-icon")
            .with_permission(AppPermission::Notifications)
            .system();

        Self {
            metadata,
            conversations: HashMap::new(),
            active_conversation: None,
        }
    }
}

impl MessagesApp {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve or create a conversation thread for a set of participants.
    pub fn get_or_create_conversation(&mut self, mut participants: Vec<String>) -> Uuid {
        participants.sort();
        for conv in self.conversations.values() {
            let mut existing = conv.participant_numbers.clone();
            existing.sort();
            if existing == participants {
                return conv.id;
            }
        }

        let conv = Conversation::new(participants);
        let id = conv.id;
        self.conversations.insert(id, conv);
        id
    }

    /// Find an existing conversation with a given phone number.
    pub fn find_by_participant(&self, number: &str) -> Option<Uuid> {
        self.conversations
            .values()
            .find(|c| c.participant_numbers.iter().any(|p| p == number))
            .map(|c| c.id)
    }

    /// Send an outgoing SMS message in a conversation.
    pub fn send_message(
        &mut self,
        conversation_id: Uuid,
        sender: &str,
        recipient: &str,
        text: &str,
    ) -> Result<ChatMessage> {
        let conv = self
            .conversations
            .get_mut(&conversation_id)
            .ok_or_else(|| NpwdError::ConversationNotFound(conversation_id.to_string()))?;

        let message = ChatMessage {
            id: Uuid::new_v4(),
            conversation_id,
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            text: text.to_string(),
            timestamp: Utc::now(),
            is_read: true,
            attachments: Vec::new(),
        };

        conv.last_message_preview = text.to_string();
        conv.last_activity = message.timestamp;
        conv.messages.push(message.clone());

        Ok(message)
    }

    /// Process an inbound SMS message from another phone or game event.
    pub fn receive_message(
        &mut self,
        sender: &str,
        recipient: &str,
        text: &str,
    ) -> Result<ChatMessage> {
        let conv_id = self.get_or_create_conversation(vec![sender.to_string(), recipient.to_string()]);
        let is_currently_viewing = self.active_conversation == Some(conv_id);

        let conv = self
            .conversations
            .get_mut(&conv_id)
            .ok_or_else(|| NpwdError::ConversationNotFound(conv_id.to_string()))?;

        let message = ChatMessage {
            id: Uuid::new_v4(),
            conversation_id: conv_id,
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            text: text.to_string(),
            timestamp: Utc::now(),
            is_read: is_currently_viewing,
            attachments: Vec::new(),
        };

        conv.last_message_preview = text.to_string();
        conv.last_activity = message.timestamp;
        if !is_currently_viewing {
            conv.unread_count += 1;
        }
        conv.messages.push(message.clone());

        self.sync_badge_count();
        Ok(message)
    }

    /// Mark all messages in a conversation as read.
    pub fn mark_as_read(&mut self, conversation_id: Uuid) -> Result<()> {
        let conv = self
            .conversations
            .get_mut(&conversation_id)
            .ok_or_else(|| NpwdError::ConversationNotFound(conversation_id.to_string()))?;

        for msg in &mut conv.messages {
            msg.is_read = true;
        }
        conv.unread_count = 0;
        self.sync_badge_count();
        Ok(())
    }

    /// Open a specific conversation thread.
    pub fn open_conversation(&mut self, conversation_id: Uuid) -> Result<()> {
        if !self.conversations.contains_key(&conversation_id) {
            return Err(NpwdError::ConversationNotFound(conversation_id.to_string()));
        }
        self.active_conversation = Some(conversation_id);
        self.mark_as_read(conversation_id)?;
        Ok(())
    }

    /// Close active conversation view back to thread list.
    pub fn close_conversation(&mut self) {
        self.active_conversation = None;
    }

    /// Get conversation details by ID.
    pub fn get_conversation(&self, id: Uuid) -> Option<&Conversation> {
        self.conversations.get(&id)
    }

    /// List all conversation threads ordered by most recent activity.
    pub fn list_conversations(&self) -> Vec<&Conversation> {
        let mut list: Vec<&Conversation> = self.conversations.values().collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.last_activity));
        list
    }

    /// Total count of unread messages across all threads.
    pub fn total_unread_count(&self) -> u32 {
        self.conversations.values().map(|c| c.unread_count).sum()
    }

    /// Delete a conversation thread.
    pub fn delete_conversation(&mut self, id: Uuid) -> Result<()> {
        self.conversations
            .remove(&id)
            .ok_or_else(|| NpwdError::ConversationNotFound(id.to_string()))?;
        if self.active_conversation == Some(id) {
            self.active_conversation = None;
        }
        self.sync_badge_count();
        Ok(())
    }

    fn sync_badge_count(&mut self) {
        self.metadata.badge_count = self.total_unread_count();
    }
}

impl PhoneApp for MessagesApp {
    fn metadata(&self) -> &AppMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut AppMetadata {
        &mut self.metadata
    }

    fn on_mount(&mut self, _ctx: &mut AppContext) -> Result<()> {
        self.sync_badge_count();
        Ok(())
    }

    fn on_pause(&mut self, _ctx: &mut AppContext) -> Result<()> {
        self.active_conversation = None;
        Ok(())
    }

    fn on_resume(&mut self, _ctx: &mut AppContext) -> Result<()> {
        self.sync_badge_count();
        Ok(())
    }

    fn on_close(&mut self, _ctx: &mut AppContext) -> Result<()> {
        self.active_conversation = None;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &AppInputEvent,
        ctx: &mut AppContext,
    ) -> Result<Option<AppOutputAction>> {
        match event {
            AppInputEvent::GameBridge(GameEvent::ReceiveSms { from, text }) => {
                let _msg = self.receive_message(from, ctx.owner_number, text)?;
                ctx.notify(Notification::new(
                    "messages",
                    format!("New SMS from {}", from),
                    text.clone(),
                ));
                Ok(Some(AppOutputAction::SetBadge(self.metadata.badge_count)))
            }
            AppInputEvent::UiAction { action, payload } if action == "send_sms" => {
                let to = payload["to"].as_str().unwrap_or_default();
                let text = payload["text"].as_str().unwrap_or_default();
                let conv_id = self.get_or_create_conversation(vec![ctx.owner_number.to_string(), to.to_string()]);
                self.send_message(conv_id, ctx.owner_number, to, text)?;

                ctx.emit_phone_event(PhoneEvent::SendSms {
                    to: to.to_string(),
                    text: text.to_string(),
                });
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
