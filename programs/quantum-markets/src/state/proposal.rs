use anchor_lang::prelude::*;
use crate::constants::*;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq)]
pub enum PoolSide { Yes, No }

#[account]
pub struct ProposalConfig {
    pub id:          u64,
    pub market_id:   u64,
    pub created_at:  i64,
    pub creator:     Pubkey,

    pub vusd_mint:   Pubkey,
    pub yes_mint:    Pubkey,
    pub no_mint:     Pubkey,

    // pool keys will be written later when pools are created
    pub yes_pool:    Pubkey,
    pub no_pool:     Pubkey,

    pub data:        Vec<u8>,
    pub bump:        u8,
}

impl ProposalConfig {
    pub const SIZE: usize =
          DISCRIMINATOR
        + U64_L * 2         // id + market_id
        + U64_L             // created_at
        + PUBKEY_L * 4      // creator + 3 mint addresses
        + PUBKEY_L * 2      // pool keys
        + STRING_PREFIX + 256   // data blob (adjust as you like)
        + U8_L;             // bump
}
