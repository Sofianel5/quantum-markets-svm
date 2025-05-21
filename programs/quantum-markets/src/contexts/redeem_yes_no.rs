use anchor_lang::prelude::*;
use anchor_spl::token::{Burn, burn, Transfer, TokenAccount, Mint, Token};
use crate::state::proposal::ProposalConfig;

#[derive(Accounts)]
pub struct RedeemYesNo<'info> {
    #[account(mut)] pub payer: Signer<'info>,

    #[account(seeds = [b"proposal", &proposal.id.to_le_bytes()], bump)]
    pub proposal: Account<'info, ProposalConfig>,

    /// CHECK:
    #[account(seeds = [b"proposal_auth"], bump)]
    pub proposal_auth: UncheckedAccount<'info>,

    // mints
    #[account(mut, address = proposal.yes_mint)] pub yes_mint: Account<'info, Mint>,
    #[account(mut, address = proposal.no_mint)]  pub no_mint:  Account<'info, Mint>,
    #[account(address = proposal.vusd_mint)] pub vusd_mint: Account<'info, Mint>,

    // user token accounts
    #[account(mut)] pub user_yes: Account<'info, TokenAccount>,
    #[account(mut)] pub user_no:  Account<'info, TokenAccount>,
    #[account(mut)] pub user_vusd: Account<'info, TokenAccount>,

    // vault that holds vUSD
    #[account(
        mut,
        associated_token::mint = vusd_mint,
        associated_token::authority = proposal_auth
    )]
    pub vault_vusd: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> RedeemYesNo<'info> {
    pub fn handler(&mut self, bumps: RedeemYesNoBumps, amount: u64) -> Result<()> {
        // burn YES + NO from caller
        for (mint, from) in [
            (&self.yes_mint, &self.user_yes),
            (&self.no_mint,  &self.user_no),
        ] {
            burn(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    Burn {
                        mint:      mint.to_account_info(),
                        from:      from.to_account_info(),
                        authority: self.payer.to_account_info(),
                    }),
                amount,
            )?;
        }
        
        let auth_seeds: &[&[u8]] = &[b"proposal_auth", &[bumps.proposal_auth]];

        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from:      self.vault_vusd.to_account_info(),
                    to:        self.user_vusd.to_account_info(),
                    authority: self.proposal_auth.to_account_info(),
                },
                &[auth_seeds],
            ),
            amount,
        )?;
        Ok(())
    }
}
