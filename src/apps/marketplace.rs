use crate::app::{AppContext, AppInputEvent, AppMetadata, AppOutputAction, AppPermission, PhoneApp};
use crate::apps::camera::WorldPosition;
use crate::error::{NpwdError, Result};
use crate::events::{GameEvent, Notification, PhoneEvent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use uuid::Uuid;

/// Listing category ranging from public goods to DarkNet clandestine contraband.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListingCategory {
    General,
    Vehicles,
    Electronics,
    Services,
    BlackMarket,
}

/// Lifecycle status of a marketplace posting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListingStatus {
    Active,
    Sold,
    InDelivery,
    Completed,
    Cancelled,
}

/// Item or service listing on the simulated marketplace / DarkNet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Listing {
    pub id: Uuid,
    pub seller_name: String,
    pub seller_contact: String,
    pub title: String,
    pub description: String,
    /// Price in cents.
    pub price: i64,
    pub category: ListingCategory,
    pub is_anonymous: bool,
    pub status: ListingStatus,
    pub created_at: DateTime<Utc>,
}

/// Timed courier or dead-drop delivery tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Delivery {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub item_name: String,
    pub buyer_id: String,
    pub drop_location: Option<WorldPosition>,
    pub remaining_seconds: f32,
    pub total_seconds: f32,
    pub is_delivered: bool,
}

/// Marketplace & DarkNet simulated smartphone application.
#[derive(Debug, Clone)]
pub struct MarketplaceApp {
    metadata: AppMetadata,
    listings: HashMap<Uuid, Listing>,
    deliveries: HashMap<Uuid, Delivery>,
}

impl Default for MarketplaceApp {
    fn default() -> Self {
        let metadata = AppMetadata::new("marketplace", "Marketplace", "market-icon")
            .with_permission(AppPermission::Network)
            .with_permission(AppPermission::Location)
            .system();

        Self {
            metadata,
            listings: HashMap::new(),
            deliveries: HashMap::new(),
        }
    }
}

impl MarketplaceApp {
    pub fn new() -> Self {
        Self::default()
    }

    /// Post a new listing for sale.
    #[allow(clippy::too_many_arguments)]
    pub fn create_listing(
        &mut self,
        seller_name: impl Into<String>,
        seller_contact: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        price: i64,
        category: ListingCategory,
        is_anonymous: bool,
    ) -> Result<Listing> {
        let listing = Listing {
            id: Uuid::new_v4(),
            seller_name: seller_name.into(),
            seller_contact: seller_contact.into(),
            title: title.into(),
            description: description.into(),
            price,
            category,
            is_anonymous,
            status: ListingStatus::Active,
            created_at: Utc::now(),
        };

        self.listings.insert(listing.id, listing.clone());
        Ok(listing)
    }

    /// Purchase a listing and dispatch a timed delivery.
    pub fn purchase_listing(
        &mut self,
        listing_id: Uuid,
        buyer_id: &str,
        drop_location: Option<WorldPosition>,
        delivery_duration_secs: f32,
    ) -> Result<Delivery> {
        let listing = self
            .listings
            .get_mut(&listing_id)
            .ok_or_else(|| NpwdError::ListingNotFound(listing_id.to_string()))?;

        if listing.status != ListingStatus::Active {
            return Err(NpwdError::Internal("Listing is not active".to_string()));
        }

        listing.status = ListingStatus::InDelivery;

        let delivery = Delivery {
            id: Uuid::new_v4(),
            listing_id,
            item_name: listing.title.clone(),
            buyer_id: buyer_id.to_string(),
            drop_location,
            remaining_seconds: delivery_duration_secs,
            total_seconds: delivery_duration_secs,
            is_delivered: false,
        };

        self.deliveries.insert(delivery.id, delivery.clone());
        Ok(delivery)
    }

    /// Advance delivery timers; returns any deliveries that reached destination this tick.
    pub fn tick_deliveries(&mut self, delta_seconds: f32) -> Vec<Delivery> {
        let mut completed = Vec::new();

        for delivery in self.deliveries.values_mut() {
            if !delivery.is_delivered {
                delivery.remaining_seconds = (delivery.remaining_seconds - delta_seconds).max(0.0);
                if delivery.remaining_seconds <= 0.0 {
                    delivery.is_delivered = true;
                    completed.push(delivery.clone());
                }
            }
        }

        for c in &completed {
            if let Some(l) = self.listings.get_mut(&c.listing_id) {
                l.status = ListingStatus::Completed;
            }
        }

        completed
    }

    /// Query listings by keyword and optional category filter.
    pub fn search(&self, query: &str, category: Option<ListingCategory>) -> Vec<&Listing> {
        let q = query.to_lowercase();
        let mut results: Vec<&Listing> = self
            .listings
            .values()
            .filter(|l| {
                let matches_cat = category.is_none() || category == Some(l.category);
                let matches_query = q.is_empty()
                    || l.title.to_lowercase().contains(&q)
                    || l.description.to_lowercase().contains(&q);
                matches_cat && matches_query && l.status == ListingStatus::Active
            })
            .collect();

        results.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        results
    }

    /// List all active listings.
    pub fn list_active(&self) -> Vec<&Listing> {
        let mut list: Vec<&Listing> = self
            .listings
            .values()
            .filter(|l| l.status == ListingStatus::Active)
            .collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        list
    }

    /// Retrieve listing by ID.
    pub fn get_listing(&self, id: Uuid) -> Option<&Listing> {
        self.listings.get(&id)
    }

    /// Active pending deliveries.
    pub fn pending_deliveries(&self) -> Vec<&Delivery> {
        self.deliveries
            .values()
            .filter(|d| !d.is_delivered)
            .collect()
    }
}

impl PhoneApp for MarketplaceApp {
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
        event: &AppInputEvent,
        ctx: &mut AppContext,
    ) -> Result<Option<AppOutputAction>> {
        match event {
            AppInputEvent::GameBridge(GameEvent::BountyPlaced {
                target_name,
                reward,
                description,
            }) => {
                let listing = self.create_listing(
                    "Anonymous Contract",
                    "DARKNET-DROP",
                    format!("BOUNTY: {}", target_name),
                    description,
                    *reward,
                    ListingCategory::BlackMarket,
                    true,
                )?;
                ctx.notify(
                    Notification::new(
                        "marketplace",
                        "New DarkNet Contract",
                        format!("Target: {} (Reward: ${:.2})", target_name, (*reward as f64) / 100.0),
                    )
                    .with_sound("darknet_chime"),
                );
                Ok(Some(AppOutputAction::Custom(serde_json::to_value(&listing).unwrap())))
            }
            AppInputEvent::UiAction { action, payload } if action == "buy" => {
                let listing_id = Uuid::parse_str(payload["listing_id"].as_str().unwrap_or_default())
                    .map_err(|e| NpwdError::Internal(e.to_string()))?;
                let duration = payload["duration"].as_f64().unwrap_or(30.0) as f32;

                let delivery = self.purchase_listing(listing_id, ctx.player_id, None, duration)?;
                let listing = self.get_listing(listing_id).unwrap();

                ctx.emit_phone_event(PhoneEvent::PurchaseItem {
                    listing_id,
                    price: listing.price,
                });

                ctx.notify(Notification::new(
                    "marketplace",
                    "Order Confirmed",
                    format!("Courier dispatched for {}", delivery.item_name),
                ));

                Ok(Some(AppOutputAction::Custom(serde_json::to_value(&delivery).unwrap())))
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
