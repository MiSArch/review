use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject};
use bson::{doc, Document, Uuid};
use mongodb::{options::FindOptions, Collection, Database};
use mongodb_cursor_pagination::{error::CursorError, FindResult, PaginatedCursor};
use serde::{Deserialize, Serialize};

use super::{
    connection::{
        base_connection::{BaseConnection, FindResultWrapper},
        review_connection::ReviewConnection,
    },
    order_datatypes::ReviewOrderInput,
    review::Review,
};

/// Type of a user owning reviews.
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone, SimpleObject)]
#[graphql(complex)]
pub struct User {
    /// UUID of the user.
    pub _id: Uuid,
}

#[ComplexObject]
impl User {
    /// Retrieves reviews of user.
    async fn reviews<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "Describes that the `first` N reviews should be retrieved.")]
        first: Option<u32>,
        #[graphql(desc = "Describes how many reviews should be skipped at the beginning.")]
        skip: Option<u64>,
        #[graphql(desc = "Specifies the order in which reviews are retrieved.")] order_by: Option<
            ReviewOrderInput,
        >,
    ) -> Result<ReviewConnection> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Review> = db_client.collection::<Review>("reviews");
        let review_order = order_by.unwrap_or_default();
        let sorting_doc = doc! {review_order.field.unwrap_or_default().as_str(): i32::from(review_order.direction.unwrap_or_default())};
        let find_options = FindOptions::builder()
            .skip(skip)
            .limit(first.map(|definitely_first| i64::from(definitely_first)))
            .sort(sorting_doc)
            .build();
        let document_collection = collection.clone_with_type::<Document>();
        let filter = doc! {"user._id": self._id};
        let maybe_find_results: Result<FindResult<Review>, CursorError> =
            PaginatedCursor::new(Some(find_options.clone()), None, None)
                .find(&document_collection, Some(&filter))
                .await;
        match maybe_find_results {
            Ok(find_results) => {
                let find_result_wrapper = FindResultWrapper(find_results);
                let connection = Into::<BaseConnection<Review>>::into(find_result_wrapper);
                Ok(Into::<ReviewConnection>::into(connection))
            }
            Err(_) => return Err(Error::new("Retrieving reviews failed in MongoDB.")),
        }
    }
}

impl From<Uuid> for User {
    fn from(value: Uuid) -> Self {
        User { _id: value }
    }
}
