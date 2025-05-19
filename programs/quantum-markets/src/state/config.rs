use anchor_lang::prelude::*;
use crate::constants::*;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq)]
pub enum MarketStatus {
  Open,
  ProposalAccepted,
  Timeout,
  ResolvedYes,
  ResolvedNo,
}

#[account]
pub struct MarketConfig {
  pub id:        u64,
  pub created_at: i64,
  pub min_deposit: u64,
  pub strike_price: u64,
  pub creator:   Pubkey,
  pub market_token: Pubkey,
  pub resolver:  Pubkey,
  pub status:    MarketStatus,  // derive AnchorEnum
  pub title:     String,
  pub bump:      u8,
}

impl MarketConfig {
    pub const SIZE: usize = 
          DISCRIMINATOR
        + U64_L          // id: u64
        + U64_L          // created_at: i64 (8 bytes)
        + U64_L          // min_deposit: u64
        + U64_L          // strike_price: u64
        + PUBKEY_L       // creator: Pubkey
        + PUBKEY_L       // market_token: Pubkey
        + PUBKEY_L       // resolver: Pubkey
        + U8_L           // status: MarketStatus as a u8
        + STRING_PREFIX + STR_MAX_LEN // title: String
        + U8_L;          // bump: u8
    
    pub fn init(
        &mut self,
        id: u64,
        created_at: i64,
        min_deposit: u64,
        strike_price: u64,
        creator: Pubkey,
        market_token: Pubkey,
        resolver: Pubkey,
        status: MarketStatus,
        title: String,
        bump: u8
    ) {
        self.id = id;
        self.created_at = created_at;
        self.min_deposit = min_deposit;
        self.strike_price = strike_price;
        self.creator = creator;
        self.market_token = market_token;
        self.resolver = resolver;
        self.status = status;
        self.title = title;
        self.bump = bump;
    }
}
