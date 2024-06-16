use std::str::FromStr;

use async_graphql::{scalar, InputObject, Request, Response};
use linera_sdk::{
    base::{ApplicationId, ChainId, ContractAbi, Owner, ServiceAbi, AccountOwner},
    graphql::GraphQLMutationRoot,
    views::crypto::CryptoHash,
};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

pub struct BlockbloomABI;

impl ContractAbi for BlockbloomABI {
    type Parameters = ();
    type InitializationArgument = ();
    type Operation = Operation;
    type Message = Message;
    type ApplicationCall = ApplicationCall;
    type SessionCall = ();
    type SessionState = ();
    type Response = ();
}

impl ServiceAbi for BlockbloomABI {
    type Parameters = ();
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, InputObject,
)]
pub struct Account {
    pub chain_id: ChainId,
    pub owner: AccountOwner,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Transfer {
        token_id: u64,
        target_account: Account,
    },
    Recieve {
        token_id: u64,
        target_account: AccountOwner,
    },
}

impl Default for Message {
    fn default() -> Self {
        Message::Transfer {
            token_id: 0,
            target_account: Account {
                chain_id: ChainId(CryptoHash::from_str("S").unwrap()),
                owner: AccountOwner::User(Owner(CryptoHash::from_str("S").unwrap())),
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    Mint {
        owner: AccountOwner,
        token_id: u64,
        token_uri: String,
    },

    Transfer {
        token_id: u64,
        new_owner: Account,
    },
}
