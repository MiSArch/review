use async_graphql::{InputObject, SimpleObject};
use bson::Uuid;
use crate::review::Rating;

#[derive(SimpleObject, InputObject)]
pub struct AddReviewInput {
    /// UUID of product variant in review.
    pub product_variant_id: Uuid,
    /// Body of Review.
    pub body: String,
    /// Rating of Review in 1-5 stars.
    pub rating: Rating,
    /// Flag if review is visible, by default set to true.
    pub is_visible: Option<bool>,
}

#[derive(SimpleObject, InputObject)]
pub struct UpdateReviewInput {
    /// UUID of review to update.
    pub id: Uuid,
    /// Body of Review to update.
    pub body: Option<String>,
    /// Rating of Review in 1-5 stars to update.
    pub rating: Option<Rating>,
    /// Flag if review is visible.
    pub is_visible: Option<bool>,
}
