use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::state::Market;

/// Accounts for create_market(...)
#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct CreateMarket<'info> {
    #[account(
      init,
      payer = creator,
      space = 8  // discriminator
        + 8   // id: u64
        + 8   // created_at: i64
        + 8   // min_deposit
        + 8   // strike_price
        + 32  // creator: Pubkey
        + 32  // market_token: Pubkey
        + 32  // resolver: Pubkey
        + 1   // status: enum (u8)
        + 4+64// title: String (up to 64 bytes)
        + 9   // accepted_proposal: Option<u64>
      ,
      seeds = [b"market", &market_id.to_le_bytes()],
      bump
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub creator: Signer<'info>,

    /// The SPL mint your market uses for deposits:
    #[account()]
    pub market_token_mint: Account<'info, Mint>,

    /// A resolver program or oracle (just a Pubkey):
    /// CHECK: we trust this pubkey off‐chain
    pub resolver: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub rent:           Sysvar<'info, Rent>,
    pub token_program:  Program<'info, Token>,
}

/// Accounts for deposit_to_market(...)
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds=[b"market", &market.id.to_le_bytes()], bump)]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub depositor: Signer<'info>,

    /// The depositor’s token account for market_token:
    #[account(mut,
      associated_token::mint = market.market_token,
      associated_token::authority = depositor
    )]
    pub depositor_ata: Account<'info, anchor_spl::token::TokenAccount>,

    /// A vault PDA where the program holds deposits:
    #[account(mut,
      associated_token::mint = market.market_token,
      associated_token::authority = market
    )]
    pub vault_ata: Account<'info, anchor_spl::token::TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
