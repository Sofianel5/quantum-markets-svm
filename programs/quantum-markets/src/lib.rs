use anchor_lang::prelude::*;

mod contexts;
use contexts::*;
mod constants;
mod state;
mod errors;

declare_id!("ASnYjL8hE148BWM35vQ85ppjc7rRK5YDLENZhPyW2D7w");

#[program]
pub mod quantum_markets {
    use super::*;

    pub fn initialize_global(ctx: Context<InitializeGlobal>) -> Result<()> {
        ctx.accounts.handler()
    }

    pub fn create_market(
        ctx: Context<CreateMarket>,
        min_deposit: u64,
        strike_price: u64,
        title: String,
    ) -> Result<()> {
        ctx.accounts.handler(ctx.bumps, min_deposit, strike_price, title)
    }

    pub fn deposit_to_market(
        ctx: Context<DepositToMarket>,
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.handler(amount)
    }

    pub fn mint_yes_no(
        ctx: Context<MintYesNo>,
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.handler(ctx.bumps, amount)
    }

    pub fn redeem_yes_no(
        ctx: Context<RedeemYesNo>,
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.handler(ctx.bumps, amount)
    }

    pub fn claim_for_proposal(
        ctx: Context<ClaimForProposal>
    ) -> Result<()> {
        ctx.accounts.handler(ctx.bumps)
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        data: Vec<u8>,
    ) -> Result<()> {
        ctx.accounts.handler(ctx.bumps, data)
    }
}
