use async_graphql::{Enum, SimpleObject};
use bson::datetime::DateTime;
use bson::Uuid;
use serde::{Deserialize, Serialize};

use crate::{
    product_variant::ProductVariant, user::User
};

/// The Review of a user.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, SimpleObject)]
pub struct Review {
    /// Review UUID.
    pub _id: Uuid,
    /// User.
    pub user: User,
    /// Product variant that review is about.
    pub product_variant: ProductVariant,
    /// Body of Review.
    pub body: String,
    /// Rating of Review in 1-5 stars.
    pub rating: Rating,
    /// Timestamp when Review was created.
    pub created_at: DateTime,
    /// Flag if review is visible,
    pub is_visible: bool,
    
}

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Rating {
    OneStars = 1,
    TwoStars = 2,
    ThreeStars = 3,
    FourStars = 4,
    FiveStars = 5,
}