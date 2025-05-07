use anchor_lang::prelude::*;

declare_id!("ASnYjL8hE148BWM35vQ85ppjc7rRK5YDLENZhPyW2D7w");

#[program]
pub mod quantum_markets {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
