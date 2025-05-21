use anchor_lang::prelude::*;

#[account]
pub struct DepositRecord {
    pub amount: u64,   // total deposited into this market by this user
}

impl DepositRecord {
    pub const SIZE: usize = 8 /*disc*/ + 8; // just one u64
}

#[account]
pub struct ClaimRecord {
    pub claimed: u64,  // how much of that user’s deposit has been claimed into vUSD
}

impl ClaimRecord {
    pub const SIZE: usize = 8 /*disc*/ + 8;
}
