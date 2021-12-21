use std::collections::VecDeque;

use dashmap::DashMap;
use twilight_model::{
    channel::{embed::Embed, Message},
    gateway::payload::incoming::{MessageDelete, MessageUpdate},
    id::{ChannelId, MessageId, WebhookId},
};

pub struct Cache {
    messages: DashMap<ChannelId, VecDeque<CachedMessage>>,
    webhooks: DashMap<ChannelId, CachedWebhook>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            messages: DashMap::new(),
            webhooks: DashMap::new(),
        }
    }

    pub fn add_message(&self, message: Message) {
        let channel_id = message.channel_id;

        let mut messages = self.messages.get_mut(&channel_id).unwrap_or_else(|| {
            self.messages
                .insert(channel_id, VecDeque::with_capacity(20));
            self.messages.get_mut(&channel_id).unwrap()
        });

        if messages.len() == 20 {
            messages.pop_front();
        }
        messages.push_back(message.into());
    }

    pub fn update_message(&self, message: MessageUpdate) {
        self._update_message(message);
    }

    fn _update_message(&self, message: MessageUpdate) -> Option<()> {
        let mut messages = self.messages.get_mut(&message.channel_id)?;

        for cached_message in messages.value_mut() {
            if cached_message.id == message.id {
                if !message.attachments.map_or(true, |v| v.is_empty()) {
                    cached_message.content = MessageContent::AttachmentsOrComponents;
                } else if let MessageContent::Valid { content, embeds } =
                    &mut cached_message.content
                {
                    if let Some(updated_content) = message.content {
                        *content = updated_content;
                    }
                    if let Some(updated_embeds) = message.embeds {
                        *embeds = updated_embeds;
                    }
                }
                return Some(());
            }
        }

        None
    }

    pub fn delete_message(&self, message: MessageDelete) {
        self._delete_message(message);
    }

    fn _delete_message(&self, message: MessageDelete) -> Option<()> {
        let mut messages = self.messages.get_mut(&message.channel_id)?;
        let message_position = messages
            .iter_mut()
            .position(|cached_message| cached_message.id == message.id)?;

        messages.remove(message_position);

        Some(())
    }
}

#[derive(Debug)]
struct CachedMessage {
    id: MessageId,
    content: MessageContent,
}

#[derive(Debug)]
enum MessageContent {
    Valid { content: String, embeds: Vec<Embed> },
    AttachmentsOrComponents,
}

impl From<Message> for CachedMessage {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            content: if message.attachments.is_empty() && message.components.is_empty() {
                MessageContent::Valid {
                    content: message.content,
                    embeds: message.embeds,
                }
            } else {
                MessageContent::AttachmentsOrComponents
            },
        }
    }
}

struct CachedWebhook {
    id: WebhookId,
    token: String,
}
