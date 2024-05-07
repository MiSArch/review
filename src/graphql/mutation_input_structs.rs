use async_graphql::{InputObject, SimpleObject};
use bson::Uuid;

use super::model::review::Rating;

#[derive(SimpleObject, InputObject)]
pub struct CreateReviewInput {
    /// UUID of user owning the review.
    pub user_id: Uuid,
    /// UUID of product variant in review.
    pub product_variant_id: Uuid,
    /// Body of review.
    pub body: String,
    /// Rating of review in 1-5 stars.
    pub rating: Rating,
    /// Flag if review is visible, by default set to true.
    pub is_visible: Option<bool>,
}

#[derive(SimpleObject, InputObject)]
pub struct UpdateReviewInput {
    /// UUID of review to update.
    pub id: Uuid,
    /// Body of review to update.
    pub body: Option<String>,
    /// Rating of review in 1-5 stars to update.
    pub rating: Option<Rating>,
    /// Flag if review is visible.
    pub is_visible: Option<bool>,
}
