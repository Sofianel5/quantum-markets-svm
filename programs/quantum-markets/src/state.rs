use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MarketStatus {
    Open,
    ProposalAccepted,
    Timeout,
    ResolvedYes,
    ResolvedNo,
}

#[account]
pub struct Market {
    pub id:           u64,
    pub created_at:   i64,         // Unix timestamp
    pub min_deposit:  u64,
    pub strike_price: u64,
    pub creator:      Pubkey,
    pub market_token: Pubkey,      // SPL-Token mint for the “stake” token
    pub resolver:     Pubkey,      // off-chain resolver program or oracle
    pub status:       MarketStatus,
    pub title:        String,      // up to 64 bytes, adjust space
    pub accepted_proposal: Option<u64>,
}

#[account]
pub struct Proposal {
    pub id:           u64,
    pub market:       Pubkey,      // PDA of the Market this belongs to
    pub created_at:   i64,
    pub creator:      Pubkey,
    pub vusd_mint:    Pubkey,      // mint address for VUSD
    pub yes_mint:     Pubkey,      // mint address for YES token
    pub no_mint:      Pubkey,      // mint address for NO token
    pub data:         Vec<u8>,     // your “bytes data” payload
}
