use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, Transfer};

use crate::state::*;

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
      seeds = [
        b"market".as_ref(),                // &[u8]
        &market_id.to_le_bytes()[..],      // &[u8]
    ],
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

/// Accounts for create_proposal(...)
#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CreateProposal<'info> {
    /// the market this proposal belongs to
    #[account(mut, seeds = [
        b"market".as_ref(),
        &market.id.to_le_bytes()[..]
      ], bump)]
    pub market: Account<'info, Market>,

    /// the signer who is creating
    #[account(mut)]
    pub creator: Signer<'info>,

    /// your new proposal account PDA
    #[account(
      init,
      payer = creator,
      space = 8  // discriminator
        + 8    // id: u64
        + 32   // market: Pubkey
        + 8    // created_at: i64
        + 32   // creator: Pubkey
        + 32   // vusd_mint: Pubkey
        + 32   // yes_mint: Pubkey
        + 32   // no_mint: Pubkey
        + (4 + 1000) // data: Vec<u8> (max 1000 bytes for example)
      ,
      seeds = [
        b"market".as_ref(),                // &[u8]
        &proposal_id.to_le_bytes()[..],      // &[u8]
      ],
      bump
    )]
    pub proposal: Account<'info, Proposal>,

    /// the three mints you’ll CPI-init…
    #[account(
      init,
      payer = creator,
      mint::decimals = 6,
      mint::authority = program_authority,     // you’ll define this PDA below
      seeds = [
        b"vusd_mint".as_ref(),
        &proposal_id.to_le_bytes()[..]
      ],
      bump
    )]
    pub vusd_mint: Account<'info, Mint>,

    #[account(
      init,
      payer = creator,
      mint::decimals = 0,
      mint::authority = program_authority,
      seeds = [
        b"yes_mint".as_ref(),
        &proposal_id.to_le_bytes()[..]
      ],
      bump
    )]
    pub yes_mint: Account<'info, Mint>,

    #[account(
      init,
      payer = creator,
      mint::decimals = 0,
      mint::authority = program_authority,
      seeds = [
        b"no_mint".as_ref(),
        &proposal_id.to_le_bytes()[..]
      ],
      bump
    )]
    pub no_mint: Account<'info, Mint>,

    /// a PDA that signs your CPI calls
    /// CHECK: will be derived as (b"authority", &[bump])
    pub program_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program:  Program<'info, anchor_spl::token::Token>,
    pub rent:           Sysvar<'info, Rent>,
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

impl<'info> Deposit<'info> {
    pub fn into_transfer_to_vault_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from:      self.depositor_ata.to_account_info(),
            to:        self.vault_ata.to_account_info(),
            authority: self.depositor.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}