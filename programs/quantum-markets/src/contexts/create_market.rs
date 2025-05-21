use anchor_lang::{prelude::*};
use crate::errors::QuantumError;
use crate::state::config::{MarketStatus, MarketConfig};
use crate::state::global::GlobalState;
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub reward_mint: Account<'info, Mint>,
    /// CHECK: this is just a Pubkey to store the on-chain resolver address;
    pub resolver: UncheckedAccount<'info>,
    #[account(
        init,
        seeds = [b"market".as_ref(), &global.next_id.to_le_bytes()],
        bump,
        payer = payer,
        space = 8 + MarketConfig::SIZE,
    )]
    pub market: Account<'info, MarketConfig>,
    #[account(
      mut,
      seeds = [b"global"],
      bump,
    )]
    pub global: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>
}

impl<'info> CreateMarket<'info> {
    pub fn create_market(
        &mut self,
        bumps: CreateMarketBumps,
        min_deposit: u64,
        strike_price: u64,
        title: String,
    ) -> Result<()> {
        let market_id = self.global.next_id;
        self.global.next_id = market_id
            .checked_add(1)
            .ok_or(QuantumError::Overflow)?;
        let bump = bumps.market;
        let now = Clock::get()?.unix_timestamp;
        self.market.created_at = now;
        self.market.min_deposit = min_deposit;
        self.market.strike_price = strike_price;
        self.market.creator = self.payer.key();
        self.market.market_token = self.reward_mint.key();
        self.market.resolver = self.resolver.key();
        self.market.status = MarketStatus::Open;
        self.market.title = title;
        self.market.bump = bump;
        Ok(())
    }
}