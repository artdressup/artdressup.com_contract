use std::collections::HashMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, PromiseOrValue,
};

use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::approval::*;
pub use crate::royalty::*;
pub use crate::events::*;
use crate::artdressup_nft::{ReservationNft};

mod internal;
mod approval;
mod enumeration;
mod metadata;
mod mint;
mod nft_core;
mod royalty;
mod events;
mod artdressup_nft;

/// This spec can be treated like a version of the standard.
pub const NFT_METADATA_SPEC: &str = "1.0.0";
/// This is the name of the NFT standard we're using
pub const NFT_STANDARD_NAME: &str = "nep171";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    //contract owner
    pub owner_id: AccountId,

    //keeps track of all the token IDs for a given account
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    //keeps track of the token struct for a given token ID
    pub tokens_by_id: LookupMap<TokenId, Token>,

    //keeps track of the token metadata for a given token ID
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

    //keeps track of the metadata for the contract
    pub metadata: LazyOption<NFTContractMetadata>,

    // nft mint 예약
    // pub reservation_mints: LookupMap<TokenId, ReservationMint>,

    pub reservation_nft: LookupMap<AccountId, Vector<ReservationNft>>,

    // nft 발행 시 번호 부여, mint 될때 마다 1씩 증가
    pub nft_seq_num: LazyOption<u32>,
    // nft 소각 시 1씩 증가
    pub nft_del_num: LazyOption<u32>,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
    // ReservationMints,
    ReservationNft,
    ReservationNftInner { account_id_hash: CryptoHash },
    NftSeqNum,
    NftDelNum,
}

#[near_bindgen]
impl Contract {
    /*
        initialization function (can only be called once).
        this initializes the contract with default metadata so the
        user doesn't have to manually type metadata.
    */
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        //calls the other function "new: with some default metadata and the owner_id passed in
        Self::new(
            // force_init,
            owner_id,
            NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "Art Dress Up".to_string(),
                symbol: "ADU".to_string(),
                icon: Some("https://cdn.artdressup.com/icon.png".to_string()),
                base_uri: Some("https://cdn.artdressup.com/nft/".to_string()),
                reference: None,
                reference_hash: None,
            },
        )
    }

    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub fn set_owner_id(&mut self, new_owner_id: AccountId) -> AccountId {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "no permissions");

        self.owner_id = new_owner_id;
        self.owner_id.clone()
    }

    /*
        initialization function (can only be called once).
        this initializes the contract with metadata that was passed in and
        the owner_id. 
    */
    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        // 이미 초기화된 경우에 대한 처리
        // if env::state_exists() && force_init.unwrap_or(false) == false {
        //     panic!("The contract has already been initialized");
        // }

        let zero_u32: u32 = 0;
        //create a variable of type Self with all the fields initialized.
        let this = Self {
            //Storage keys are simply the prefixes used for the collections. This helps avoid data collision
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(
                StorageKey::TokenMetadataById.try_to_vec().unwrap(),
            ),
            //set the owner_id field equal to the passed in owner_id. 
            owner_id,
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
            // reservation_mints: LookupMap::new(StorageKey::ReservationMints.try_to_vec().unwrap()),
            reservation_nft: LookupMap::new(StorageKey::ReservationNft.try_to_vec().unwrap()),
            nft_seq_num: LazyOption::new(
                StorageKey::NftSeqNum.try_to_vec().unwrap(),
                Some(&zero_u32)
            ),
            nft_del_num: LazyOption::new(
                StorageKey::NftDelNum.try_to_vec().unwrap(),
                Some(&zero_u32)
            ),
        };

        //return the Contract object
        this
    }
}

#[cfg(test)]
mod tests;