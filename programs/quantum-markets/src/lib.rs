use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

pub mod state;
pub mod instructions;

use crate::state::*;
use crate::instructions::*;

declare_id!("ASnYjL8hE148BWM35vQ85ppjc7rRK5YDLENZhPyW2D7w");

#[program]
pub mod quantum_markets {
    use super::*;

    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_id: u64,
        min_deposit: u64,
        strike_price: u64,
        title: String,
    ) -> Result<()> {
        let m = &mut ctx.accounts.market;
        m.id = market_id;
        m.created_at = Clock::get()?.unix_timestamp;
        m.min_deposit = min_deposit;
        m.strike_price = strike_price;
        m.creator = *ctx.accounts.creator.key;
        m.market_token = ctx.accounts.market_token_mint.key();
        m.resolver = ctx.accounts.resolver.key();
        m.status = MarketStatus::Open;
        m.title = title;
        Ok(())
    }

    pub fn deposit_to_market(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Transfer `amount` of SPL `market_token` from user → program vault
        token::transfer(
            ctx.accounts
                .into_transfer_to_vault_context(),
            amount,
        )?;
        // Track deposit in a PDA (e.g. `DepositRecord`)
        Ok(())
    }

    pub fn create_proposal(ctx: Context<CreateProposal>, proposal_id: u64, data: Vec<u8>) -> Result<()> {
        // 1. Init the three mints (VUSD, YES, NO) via CPI as shown above  
        // 2. Mint initial liquidity according to min_deposit  
        // 3. Write Proposal { id, market, creator, vusd_mint, yes_mint, no_mint, data }  
        let p = &mut ctx.accounts.proposal;
        p.id = proposal_id;
        p.market = ctx.accounts.market.key();
        p.created_at = Clock::get()?.unix_timestamp;
        p.creator = *ctx.accounts.creator.key;
        p.vusd_mint = ctx.accounts.vusd_mint.key();
        p.yes_mint = ctx.accounts.yes_mint.key();
        p.no_mint = ctx.accounts.no_mint.key();
        p.data = data;
        Ok(())
    }

}

