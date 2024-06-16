use linera_sdk::views::{MapView, RegisterView, ViewStorageContext};
use linera_views::views::{GraphQLView, RootView};
use nft::AccountOwner;

#[derive(RootView, GraphQLView)]
#[view(context = "ViewStorageContext")]
pub struct BlockbloomGenToken {
    token_counter: RegisterView<u64>,
    token_owner: MapView<u64, AccountOwner>,
    token_uri: MapView<u64, String>
}

#[allow(dead_code)]

impl BlockbloomGenToken {
    pub async fn get_token_owner(&mut self, token: u64) -> AccountOwner {
        self.token_owner
            .get(&token)
            .await
            .expect("Couldn't retrieve")
            .unwrap()
    }

    pub async fn mint_nft(&mut self, token: u64, minter: AccountOwner, token_uri: String) {
        let a = self.token_counter.get();
        self.token_counter.set(*a + 1);

        self.token_owner
            .insert(&token, minter)
            .expect("Couldn't insert in Token Owner");

        self.token_uri
            .insert(&token, token_uri)
            .expect("Couldn't insert in Token URI")
    }

    pub async fn transfer_nft(&mut self, token: u64, new_owner: AccountOwner) {
        self.token_owner
            .insert(&token, new_owner)
            .expect("Couldn't transfer the NFT")
    }

    pub async fn get_all_minted_tokens(&self) -> Vec<u64> {
        let mut tokens = Vec::new();
        let counter = self.token_counter.get();
        for token_id in 0..*counter {
            tokens.push(token_id);
        }
        tokens
    }
}