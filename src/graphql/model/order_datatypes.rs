use async_graphql::{Enum, InputObject, SimpleObject};

/// GraphQL order direction.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum OrderDirection {
    /// Ascending order direction.
    Asc,
    /// Descending order direction.
    Desc,
}

impl Default for OrderDirection {
    fn default() -> Self {
        Self::Asc
    }
}

/// Implements conversion to `i32`` for MongoDB document sorting.
impl From<OrderDirection> for i32 {
    fn from(value: OrderDirection) -> Self {
        match value {
            OrderDirection::Asc => 1,
            OrderDirection::Desc => -1,
        }
    }
}

/// Describes the fields that a review can be ordered by.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReviewOrderField {
    /// Orders by "id".
    Id,
    /// Orders by "user_id".
    UserId,
    /// Orders by "product_variant".
    ProductVariant,
    /// Orders by "rating".
    Rating,
    /// Orders by "created_at".
    CreatedAt,
}

impl ReviewOrderField {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewOrderField::Id => "_id",
            ReviewOrderField::UserId => "user",
            ReviewOrderField::ProductVariant => "product_variant",
            ReviewOrderField::Rating => "rating",
            ReviewOrderField::CreatedAt => "last_updated_at",
        }
    }
}

impl Default for ReviewOrderField {
    fn default() -> Self {
        Self::Id
    }
}

/// Specifies the order of reviews.
#[derive(SimpleObject, InputObject)]
pub struct ReviewOrderInput {
    /// Order direction of reviews.
    pub direction: Option<OrderDirection>,
    /// Field that reviews should be ordered by.
    pub field: Option<ReviewOrderField>,
}

impl Default for ReviewOrderInput {
    fn default() -> Self {
        Self {
            direction: Some(Default::default()),
            field: Some(Default::default()),
        }
    }
}

/// Describes the fields that a foreign types can be ordered by.
///
/// Only the id valid at the moment.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum CommonOrderField {
    /// Orders by "id".
    Id,
}

impl CommonOrderField {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommonOrderField::Id => "_id",
        }
    }
}

impl Default for CommonOrderField {
    fn default() -> Self {
        Self::Id
    }
}

/// Specifies the order of foreign types.
#[derive(SimpleObject, InputObject)]
pub struct CommonOrderInput {
    /// Order direction of reviews.
    pub direction: Option<OrderDirection>,
    /// Field that reviews should be ordered by.
    pub field: Option<CommonOrderField>,
}

impl Default for CommonOrderInput {
    fn default() -> Self {
        Self {
            direction: Some(Default::default()),
            field: Some(Default::default()),
        }
    }
}
