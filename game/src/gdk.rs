use std::{str::FromStr, time::{SystemTime, UNIX_EPOCH}};

use aptos_sdk::{bcs, coin_client::CoinClient, rest_client::{Client, FaucetClient}, types::{account_config::chain_id, transaction::{RawTransaction, SignedTransaction}, LocalAccount}};
use bevy::utils::tracing::instrument::WithSubscriber;
use once_cell::sync::Lazy;
use url::Url;
use tokio;
use anyhow::{Context, Result};
use aptos_sdk::move_types::ident_str;
use aptos_sdk::move_types::identifier::Identifier;
use aptos_sdk::move_types::language_storage::{ModuleId, TypeTag};
use aptos_sdk::rest_client::aptos::AptosCoin;
use aptos_sdk::rest_client::Transaction;
//use aptos_sdk::rest_client::aptos_api_types::TransactionPayload;
use aptos_sdk::transaction_builder::{TransactionBuilder, TransactionFactory};
use aptos_sdk::types::account_address::AccountAddress;
use aptos_sdk::types::chain_id::ChainId;
use aptos_sdk::types::transaction::{EntryFunction, TransactionPayload};
use rand::rngs::OsRng;

use std::time::{ Instant, Duration };

static NODE_URL: Lazy<Url> = Lazy::new(|| {
    Url::from_str(
        std::env::var("APTOS_NODE_URL")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("http://127.0.0.1:8080/"),
    )
    .unwrap()
});
 
static FAUCET_URL: Lazy<Url> = Lazy::new(|| {
    Url::from_str(
        std::env::var("APTOS_FAUCET_URL")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("http://127.0.0.1:8081/"),
    )
    .unwrap()
});

enum State{
    None,    
    Creating,
    Created,
    Joining,
    Started,
}

pub struct GDK{
    rest_client: Client,
    faucet_client: FaucetClient,
    //coin_client: CoinClient,
    player_account: LocalAccount,
    transaction_factory: TransactionFactory,
    module_id: ModuleId,
    pub state: State,
    pub game_address: Option<AccountAddress>,
}

impl GDK {
    pub fn new() -> GDK{
        let rest_client = Client::new(NODE_URL.clone());
        let faucet_client = FaucetClient::new(FAUCET_URL.clone(), NODE_URL.clone());        
        let coin_client = CoinClient::new(&rest_client);
                
        // let mut alice = LocalAccount::generate(&mut OsRng);
        // let bob = LocalAccount::generate(&mut OsRng);
        let player_account = LocalAccount::generate(&mut OsRng);
        let transaction_factory = TransactionFactory::new(ChainId::new(149));
        let contract_account: AccountAddress = AccountAddress::from_hex_literal("0x35bcaf14a08f75b726ff25dbad2063286a4b3ff191753b0f1d57913b2038a687").unwrap();
        let module_id = ModuleId::new(
            contract_account,
            ident_str!("backgammon").to_owned()
        );        

        return GDK {
            rest_client,
            faucet_client,
            // coin_client,
            player_account,
            transaction_factory,
            game_address: None,
            module_id,
            state: State::None
        };


        // faucet_client
        //     .create_account(bob.address())
        //     .await
        //     .context("Failed to fund Bob's account")?;
    
        // println!(
        //     "Alice: {:?}",
        //     coin_client
        //         .get_account_balance(&alice.address())
        //         .await
        //         .context("Failed to get Alice's account balance the second time")?
        // );
        // let coin_client = CoinClient::new(&rest_client);
        // println!(
        //     "Bob: {:?}",
        //     coin_client
        //         .get_account_balance(&bob.address())
        //         .await
        //         .context("Failed to get Bob's account balance the second time")?
        // );    
    
    }

    async fn fund(&self) {
        let result = self.faucet_client
            .fund(self.player_account.address(), 100_000_000)
            .await
            .context("Failed to fund player's account");
    }

    pub fn get_address(&self)->String {
        return self.player_account.address().to_standard_string();
    }

    pub async fn get_latest_transaction_version(&self){
        let info = self.rest_client.get_ledger_information().await;
        info.unwrap().into_inner().version;        
    }

    pub async fn create_game(&mut self){      
        self.state = State::Creating;
          
        println!("{}",self.player_account.address());

        

        // let chain_id = self.rest_client.get_index()
        //     .await
        //     .context("Failed to get chain ID")?
        //     .inner()
        //     .chain_id;
        // println!("chain id: {}",chain_id);

        let function_id:Identifier = ident_str!("create_game").to_owned();
        let type_tags: Vec<TypeTag> = vec![];
        let args : Vec<Vec<u8>> = vec![];
        let entry_function = EntryFunction::new(self.module_id.clone(),function_id,type_tags,args);
        let builder = self.transaction_factory
            .entry_function(entry_function)
            .sender(self.player_account.address())
            .sequence_number(self.player_account.sequence_number())
            .max_gas_amount(5_000)
            .gas_unit_price(100)
            .expiration_timestamp_secs(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 10 // timeout_seconds
            );
        let raw_transaction = builder.build();
        let signed_transaction = self.player_account.sign_transaction(raw_transaction);
        // let simulated_result = self.rest_client.simulate_bcs(&signed_transaction).await;
        // match simulated_result {
        //     Err(error) => println!("Failed in simulation {}",error),
        //     _ => println!("{}","Simulation succeeded.")
        // }
        
        let result: std::result::Result<aptos_sdk::rest_client::Response<Transaction>, aptos_sdk::rest_client::error::RestError> = self.rest_client.submit_and_wait(&signed_transaction)
            .await;        

        self.game_address = Some(self.player_account.address());
                
        match result {
            Err(error) => println!("{}",error),
            Ok(response) => println!("{}","Game started.")
        };

        self.state = State::Created;
    }

    async fn roll_the_dice(&self){
        let function_id:Identifier = ident_str!("roll_the_dice").to_owned();
        let type_tags: Vec<TypeTag> = vec![];
        let args : Vec<Vec<u8>> = vec![];
        let entry_function = EntryFunction::new(self.module_id.clone(),function_id,type_tags,args);
        let builder = self.transaction_factory.entry_function(entry_function);
        let raw_transaction = builder.build();
        let signed_transaction = self.player_account.sign_transaction(raw_transaction);
        let result = self.rest_client.submit_and_wait(&signed_transaction)
            .await;            
        match result {
            Err(error) => println!("{}",error),
            Ok(response) => {
                println!("{}","Dice rolled.");                
            }
        }
    }

    pub async fn join_game(&mut self,game_addr_encoded: String ){
        let game_addr = AccountAddress::from_hex_literal(&game_addr_encoded.as_str()).unwrap();
        self.state = State::Joining;
        let function_id:Identifier = ident_str!("join_game").to_owned();
        let type_tags: Vec<TypeTag> = vec![TypeTag::Address];
        let args : Vec<Vec<u8>> = vec![
            bcs::to_bytes(&game_addr).unwrap()
        ];
        let entry_function = EntryFunction::new(self.module_id.clone(),function_id,type_tags,args);
        let builder = self.transaction_factory.entry_function(entry_function);
        let raw_transaction = builder.build();
        let signed_transaction = self.player_account.sign_transaction(raw_transaction);
        let result = self.rest_client.submit_and_wait(&signed_transaction)
            .await;            
        match result {
            Err(error) => println!("{}",error),
            Ok(response) => {
                println!("{}","Joined game.");
            }
        };
        self.game_address = Some(game_addr);
        self.state = State::Started;
    }
    
}


