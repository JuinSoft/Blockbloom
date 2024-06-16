#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::BlockbloomToken;
use async_graphql::{EmptySubscription, Request, Response, Schema};
use async_trait::async_trait;
use linera_sdk::{
    base::WithServiceAbi, graphql::GraphQLMutationRoot, QueryContext, Service, ViewStateStorage,
};
use nft::Operation;
use std::sync::Arc;
use thiserror::Error;

linera_sdk::service!(BlockbloomToken);

impl WithServiceAbi for BlockbloomToken {
    type Abi = nft::BlockbloomABI;
}

#[async_trait]
impl Service for BlockbloomToken {
    type Error = ServiceError;
    type Storage = ViewStateStorage<Self>;

    async fn query_application(
        self: Arc<Self>,
        _context: &QueryContext,
        request: Request,
    ) -> Result<Response, Self::Error> {
        let schema =
            Schema::build(self.clone(), Operation::mutation_root(), EmptySubscription).finish();
        let response = schema.execute(request).await;
        Ok(response)
    }
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Queries not supported by application")]
    QueriesNotSupported,

    #[error("Invalid query argument; could not deserialize request")]
    InvalidQuery(#[from] serde_json::Error),
}
