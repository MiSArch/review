use async_graphql::SimpleObject;

use crate::{base_connection::BaseConnection, review::Review};

/// A connection of Reviews.
#[derive(Debug, SimpleObject, Clone)]
#[graphql(shareable)]
pub struct ReviewConnection {
    /// The resulting entities.
    pub nodes: Vec<Review>,
    /// Whether this connection has a next page.
    pub has_next_page: bool,
    /// The total amount of items in this connection.
    pub total_count: u64,
}

/// Implementation of conversion from BaseConnection<Review> to ReviewConnection.
///
/// Prevents GraphQL naming conflicts.
impl From<BaseConnection<Review>> for ReviewConnection {
    fn from(value: BaseConnection<Review>) -> Self {
        Self {
            nodes: value.nodes,
            has_next_page: value.has_next_page,
            total_count: value.total_count,
        }
    }
}
