use crate::state::global::GlobalState;
use anchor_lang::{prelude::*};

#[derive(Accounts)]
pub struct InitializeGlobal<'info> {
  #[account(
    init,
    seeds = [b"global"],
    bump,
    payer = payer,
    space = 8 + 8,      // discriminator + u64
  )]
  pub global: Account<'info, GlobalState>,

  #[account(mut)] pub payer: Signer<'info>,
  pub system_program: Program<'info, System>,
}

impl<'info> InitializeGlobal<'info> {
  pub fn handler(&mut self) -> Result<()> {
    self.global.next_id = 0;
    Ok(())
  }
}
