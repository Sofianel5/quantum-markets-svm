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

    pub fn create_market(
        ctx: Context<CreateMarket>,
        min_deposit: u64,
        strike_price: u64,
        title: String,
    ) -> Result<()> {
        ctx.accounts.create_market(&ctx.bumps, min_deposit, strike_price, title)
    }
}


