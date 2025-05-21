use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, Mint, transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::config::MarketConfig;
use crate::state::deposit::DepositRecord;

#[derive(Accounts)]
pub struct DepositToMarket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = market.market_token)]
    pub reward_mint: Account<'info, Mint>,

    /// User’s ATA holding the market token
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,

    /// Program vault that holds all deposits for this market
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint     = reward_mint,
        associated_token::authority = market,
    )]
    pub market_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"market", &market.id.to_le_bytes()],
        bump,
    )]
    pub market: Account<'info, MarketConfig>,

    /// PDA that tracks this user’s total deposits in this market
    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"deposit", market.key().as_ref(), payer.key().as_ref()],
        bump,
        space = DepositRecord::SIZE
    )]
    pub deposit_record: Account<'info, DepositRecord>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> DepositToMarket<'info> {
    pub fn handler(&mut self, amount: u64) -> Result<()> {
        // reject if market closed
        require!(
            matches!(self.market.status, crate::state::config::MarketStatus::Open | crate::state::config::MarketStatus::ProposalAccepted),
            crate::errors::QuantumError::MarketClosed
        );

        // token transfer
        let cpi = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from:      self.user_token.to_account_info(),
                to:        self.market_vault.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        );
        transfer(cpi, amount)?;

        // bump deposit total
        self.deposit_record.amount = self
            .deposit_record
            .amount
            .checked_add(amount)
            .ok_or(crate::errors::QuantumError::Overflow)?;

        Ok(())
    }
}
