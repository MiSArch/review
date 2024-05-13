use axum::{debug_handler, extract::State, http::StatusCode, Json};
use bson::Uuid;
use log::info;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use crate::graphql::model::{product::Product, product_variant::ProductVariant, user::User};

/// Data to send to Dapr in order to describe a subscription.
#[derive(Serialize)]
pub struct Pubsub {
    #[serde(rename(serialize = "pubsubName"))]
    pub pubsubname: String,
    pub topic: String,
    pub route: String,
}

/// Reponse data to send to Dapr when receiving an event.
#[derive(Serialize)]
pub struct TopicEventResponse {
    pub status: u8,
}

/// Default status is `0` -> Ok, according to Dapr specs.
impl Default for TopicEventResponse {
    fn default() -> Self {
        Self { status: 0 }
    }
}

/// Relevant part of Dapr event wrapped in a cloud envelope.
#[derive(Deserialize, Debug)]
pub struct Event<T> {
    pub topic: String,
    pub data: T,
}

/// Relevant part of Dapr event data.
#[derive(Deserialize, Debug)]
pub struct EventData {
    pub id: Uuid,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// Relevant part of product variant creation event data.
pub struct ProductVariantEventData {
    /// Product variant UUID.
    pub id: Uuid,
    /// Associated product UUID.
    pub product_id: Uuid,
}

/// Service state containing database connections.
#[derive(Clone)]
pub struct HttpEventServiceState {
    pub product_collection: Collection<Product>,
    pub product_variant_collection: Collection<ProductVariant>,
    pub user_collection: Collection<User>,
}

/// HTTP endpoint to list topic subsciptions.
pub async fn list_topic_subscriptions() -> Result<Json<Vec<Pubsub>>, StatusCode> {
    let pubsub_user = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "user/user/created".to_string(),
        route: "/on-topic-event".to_string(),
    };
    let pubsub_product = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "catalog/product/created".to_string(),
        route: "/on-topic-event".to_string(),
    };
    let pubsub_product_variant = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "catalog/product-variant/created".to_string(),
        route: "/on-topic-event".to_string(),
    };
    Ok(Json(vec![
        pubsub_user,
        pubsub_product,
        pubsub_product_variant,
    ]))
}

/// HTTP endpoint to receive events.
///
/// * `state` - Service state containing database connections.
/// * `event` - Event handled by endpoint.
#[debug_handler(state = HttpEventServiceState)]
pub async fn on_topic_event(
    State(state): State<HttpEventServiceState>,
    Json(event): Json<Event<EventData>>,
) -> Result<Json<TopicEventResponse>, StatusCode> {
    info!("{:?}", event);

    match event.topic.as_str() {
        "user/user/created" => create_in_mongodb(&state.user_collection, event.data.id).await?,
        "catalog/product/created" => {
            create_in_mongodb(&state.product_collection, event.data.id).await?
        }
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
    Ok(Json(TopicEventResponse::default()))
}

/// HTTP endpoint to product variant creation events.
///
/// * `state` - Service state containing database connections.
/// * `event` - Event handled by endpoint.
#[debug_handler(state = HttpEventServiceState)]
pub async fn on_product_variant_creation_event(
    State(state): State<HttpEventServiceState>,
    Json(event): Json<Event<ProductVariantEventData>>,
) -> Result<Json<TopicEventResponse>, StatusCode> {
    info!("{:?}", event);

    match event.topic.as_str() {
        "catalog/product-variant/created" => {
            let product_variant = ProductVariant::from(event.data);
            add_product_variant_to_mongodb(&state.product_variant_collection, product_variant)
                .await?
        }
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
    Ok(Json(TopicEventResponse::default()))
}

/// Add a newly created product variant to MongoDB.
///
/// * `collection` - MongoDB collection to add newly created product variant to.
/// * `product_variant` - Newly created product variant.
pub async fn add_product_variant_to_mongodb(
    collection: &Collection<ProductVariant>,
    product_variant: ProductVariant,
) -> Result<(), StatusCode> {
    match collection.insert_one(product_variant, None).await {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Create a new object: `T` in MongoDB.
///
/// * `collection` - MongoDB collection to add newly created object to.
/// * `id` - UUID of newly created object.
pub async fn create_in_mongodb<T: Serialize + From<Uuid>>(
    collection: &Collection<T>,
    id: Uuid,
) -> Result<(), StatusCode> {
    let object = T::from(id);
    match collection.insert_one(object, None).await {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
