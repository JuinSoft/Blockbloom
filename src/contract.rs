#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::Blockbloom;
use async_trait::async_trait;
use linera_sdk::{
    base::{ApplicationId, Owner, SessionId, WithContractAbi},
    contract::system_api,
    ApplicationCallResult, CalleeContext, Contract, ExecutionResult, MessageContext,
    OperationContext, SessionCallResult, ViewStateStorage,
};
use log::info;
use nft::{Account, AccountOwner, ApplicationCall, Message, Operation};
use thiserror::Error;

linera_sdk::contract!(Blockbloom);

impl WithContractAbi for Blockbloom {
    type Abi = nft::NFTabi;
}

#[async_trait]
impl Contract for Blockbloom {
    type Error = CustomError;
    type Storage = ViewStateStorage<Self>;

    async fn initialize(
        &mut self,
        _ctx: &OperationContext,
        _args: Self::InitializationArgument,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        Ok(ExecutionResult::default())
    }
    async fn execute_operation(
        &mut self,
        ctx: &OperationContext,
        op: Self::Operation,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        match op {
            Operation::Mint {
                owner,
                token_id,
                token_uri,
            } => {
                self.create_nft(token_id, owner, token_uri).await;
                Ok(ExecutionResult::default())
            }

            Operation::Transfer {
                token_id,
                new_owner,
            } => {
                Self::verify_account_auth(
                    &mut self,
                    None,
                    ctx.authenticated_signer,
                    token_id,
                )
                .await?;

                self.relocate_nft_account(new_owner, token_id).await;
                Ok(ExecutionResult::default())
            }
        }
    }

    async fn execute_message(
        &mut self,
        _ctx: &MessageContext,
        msg: Self::Message,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        match msg {
            Message::Transfer {
                token_id,
                target_account,
            } => {
                self.process_message(token_id, target_account.owner).await;
                Ok(ExecutionResult::default())
            }
            Message::Recieve {
                token_id,
                target_account,
            } => {
                self.relocate_nft(token_id, target_account).await;
                Ok(ExecutionResult::default())
            }
        }
    }
}

impl Blockbloom {
    async fn relocate_nft_account(
        &mut self,
        recipient: Account,
        token_id: u64,
    ) -> ExecutionResult<Message> {

        if recipient.chain_id == system_api::current_chain_id() {
            self.relocate_nft(token_id, recipient.owner).await;
            return ExecutionResult::default();
        }

        let cross_chain_msg = Message::Recieve {
            token_id: token_id,
            target_account: recipient.owner,
        };

        self.relocate_nft(token_id, recipient.owner).await;
        ExecutionResult::default()
            .with_authenticated_message(recipient.chain_id, cross_chain_msg)
    }

    async fn verify_account_auth(
        &mut self,
        auth_app_id: Option<ApplicationId>,
        auth_signer: Option<Owner>,
        token_id: u64,
    ) -> Result<(), CustomError> {
        let previous_owner = self.fetch_token_owner(token_id).await;
        let approval: AccountOwner = self.fetch_approvals(token_id).await;

        if let AccountOwner::User(addr) = previous_owner {
            if auth_signer == Some(addr) {
                Ok(())
            } else {
                if let AccountOwner::User(addr) = approval {
                    if auth_signer == Some(addr) {
                        Ok(())
                    } else {
                        Err(CustomError::InvalidAuth)
                    }
                } else {
                    Err(CustomError::InvalidAuth)
                }
            }
        } else if let AccountOwner::Application(id) = previous_owner {
            if auth_app_id == Some(id) {
                Ok(())
            } else {
                if let AccountOwner::Application(id) = approval {
                    if auth_app_id == Some(id) {
                        return Ok(());
                    } else {
                        return Err(CustomError::InvalidAuth);
                    }
                }
                Err(CustomError::InvalidAuth)
            }
        } else {
            Err(CustomError::InvalidAuth)
        }
    }
}

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("Failed to deserialize BCS bytes")]
    BcsDeserializationError(#[from] bcs::Error),

    #[error("Failed to deserialize JSON string")]
    JsonDeserializationError(#[from] serde_json::Error),
    
    #[error("The requested transfer is not correctly authenticated.")]
    InvalidAuth,
}
