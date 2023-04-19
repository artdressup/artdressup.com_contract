use crate::*;
use near_sdk::{AccountId, env};
// use near_sdk::PromiseOrValue::Promise;
use near_sdk::Promise;
use crate::{Contract, TokenId, TokenMetadata};
use near_sdk::serde::{Serialize};

#[derive(Serialize, BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct ReservationNft {
    pub token_id: TokenId,
    pub reservation_time: u64,
}

pub trait ArtDressUpNFT {
    fn create_reservation(&mut self, token_id: TokenId);
    fn complete_reservation(&mut self, account_id: AccountId, token_id: TokenId, metadata: TokenMetadata);
    // 예약한 NFT 토큰들 확인!
    fn get_reservations(&self, account_id: AccountId) -> Option<Vec<ReservationNft>>;
    // 토큰 소각하고 10 near 돌려주기
    fn del_nft(&mut self, token_id: TokenId);
    // 예약 삭제 -- 테스트
    fn del_reservations(&mut self, account_id: AccountId);
}

#[near_bindgen]
impl ArtDressUpNFT for Contract {
    #[payable]
    fn create_reservation(&mut self, token_id: TokenId) {
        // 금액이 부족한지 확인!
        let deposit = env::attached_deposit();
        assert!(deposit >= 20, "Deposit is not sufficient.");

        if self.tokens_by_id.contains_key(&token_id) {
            panic!("Token already exists.");
        }

        // env::predecessor_account_id(),

        let account_id = env::predecessor_account_id();
        let mut reservation_nft_vec = self.reservation_nft.get(&account_id).unwrap_or_else(|| {
            Vector::new(
                StorageKey::ReservationNftInner {
                    account_id_hash: hash_account_id(&account_id)
                }.try_to_vec().unwrap()
            )
        });

        let len = reservation_nft_vec.len();
        for i in 0..len {
            let data = reservation_nft_vec.get(i).unwrap();
            // 이미 존재하는 예약!!
            assert_ne!(data.token_id, token_id, "Reservations that exist.");
        }

        let reservation = ReservationNft {
            token_id,
            reservation_time: env::block_timestamp(),
        };

        reservation_nft_vec.push(&reservation);
        self.reservation_nft.insert(&account_id, &reservation_nft_vec);


        // dev 계정으로 near 이체
        let amount_yocto = to_yocto(9); // <---- 최소 가격 보장 금액
        let dev_account = "dev.artdressup.testnet".to_string();
        let dev_account_id_op = AccountId::try_from(dev_account);
        let dev_account_id = dev_account_id_op.unwrap();

        Promise::new(dev_account_id).transfer(amount_yocto);
    }

    fn get_reservations(&self, account_id: AccountId) -> Option<Vec<ReservationNft>> {
        // let account_id = env::predecessor_account_id();
        let value = self.reservation_nft.get(&account_id);
        if value.is_none() {
            None
        } else {
            let result = value.map(|vector| {
                let vec_reservations: Vec<ReservationNft> = vector.iter().collect();
                vec_reservations
            });
            result
        }
    }

    // #[warn(unused_mut)]
    #[payable]
    fn complete_reservation(&mut self, account_id: AccountId, token_id: TokenId, metadata: TokenMetadata) {
        // 프로젝트 소유자가 호출하는 함수
        // 서버에서 이미지 조합후 webp 로 만들고, arweave에 업로드하여 url 링크를 받으면 이 함수를 호출한다.
        // 사용자가 예약한 NFT를 발행해 준다.

        // 소유자가 함수를 호출 하였는지 확인!
        assert_eq!(env::predecessor_account_id(), self.owner_id, "A function that can only be called by the owner.");

        let reservation_op = self.reservation_nft.get(&account_id);
        if reservation_op.is_none() {
            panic!("Reservation does not exist.");
        }

        let mut reservations = reservation_op.unwrap();
        let len = reservations.len();
        let mut remove_idx = 0;
        let mut is_exist = false;
        // 예약 내용이 조재하는지 확인
        for i in 0..len {
            let data = reservations.get(i).unwrap();
            if data.token_id == token_id {
                remove_idx = i;
                is_exist = true;
                break;
            }
        }

        if is_exist == false {
            panic!("Reservation does not exist.");
        }

        let receiver_id = account_id.clone();

        let mut metadata = metadata;
        if self.nft_seq_num.is_some() {
            let title = metadata.title.unwrap();
            let mut num = self.nft_seq_num.get().unwrap();
            num += 1;
            self.nft_seq_num.set(&num);
            let num_str = num.to_string();
            let new_title = format!("{} #{}", title, num_str);
            metadata.title = Some(new_title);
        }

        // 로열티 5%로 지정
        let mut royaltie: HashMap<AccountId, u32> = HashMap::new();
        royaltie.insert(self.owner_id.clone(), 5);

        self.nft_mint(token_id.clone(), metadata, receiver_id, Some(royaltie));
        // self.reservation_nft.remove(&token_id);

        // let mut metadata = metadata.title;
        // metadata.title = Some("aa".to_string());

        reservations.swap_remove(remove_idx);
        if len - 1 == 0 {
            self.reservation_nft.remove(&account_id);
        } else {
            self.reservation_nft.insert(&account_id, &reservations);
        }
    }

    // nft 소각
    fn del_nft(&mut self, token_id: TokenId) {
        let token_meta_data_op = self.token_metadata_by_id.get(&token_id);
        if token_meta_data_op.is_none() {
            panic!("NFT does not exist.");
        }

        let account_id = env::predecessor_account_id();

        let tokens_by_id_op = self.tokens_by_id.get(&token_id);
        if tokens_by_id_op.unwrap().owner_id != account_id {
            panic!("Not your NFT.")
        }

        // token_per_owner 에는
        let tokens_per_owner_op = self.tokens_per_owner.get(&account_id);
        if tokens_per_owner_op.is_none() {
            panic!("NFT does not exist.");
        }

        let mut tokens_per_owner = tokens_per_owner_op.unwrap();

        let mut exist: bool = false;
        for _token_id in tokens_per_owner.iter() {
            if _token_id == token_id {
                exist = true;
                break;
            }
        }

        if exist == false {
            panic!("NFT does not exist.");
        }

        // NFT 토큰 삭제
        self.token_metadata_by_id.remove(&token_id);
        self.tokens_by_id.remove(&token_id);
        tokens_per_owner.remove(&token_id);
        // tokens_per_owner.remove(&token_id);
        self.tokens_per_owner.insert(&account_id, &tokens_per_owner);

        let mut num = self.nft_del_num.get().unwrap();
        num += 1;
        self.nft_del_num.set(&num);

        let amount_yocto = to_yocto(10); // <---- 최소 가격 보장 금액
        Promise::new(account_id).transfer(amount_yocto);
    }

    fn del_reservations(&mut self, account_id: AccountId) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "A function that can only be called by the owner.");

        self.reservation_nft.remove(&account_id);
    }
}

fn to_yocto(near_amount: u128) -> u128 {
    near_amount * 10u128.pow(24)
}