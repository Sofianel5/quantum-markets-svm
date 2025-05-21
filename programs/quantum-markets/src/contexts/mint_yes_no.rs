use anchor_lang::prelude::*;
use anchor_spl::token::{Transfer, transfer, MintTo, mint_to, TokenAccount, Mint, Token};
use crate::state::proposal::ProposalConfig;

#[derive(Accounts)]
pub struct MintYesNo<'info> {
    #[account(mut)] pub payer: Signer<'info>,

    #[account(
        seeds = [b"proposal", &proposal.id.to_le_bytes()],
        bump
    )]
    pub proposal: Account<'info, ProposalConfig>,

    // vUSD mint and authority
    #[account(mut, address = proposal.vusd_mint)] pub vusd_mint: Account<'info, Mint>,
    /// CHECK:
    #[account(seeds = [b"proposal_auth"], bump)]
    pub proposal_auth: UncheckedAccount<'info>,

    // user gives vUSD
    #[account(mut)]
    pub user_vusd: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = vusd_mint,
        associated_token::authority = proposal_auth
    )]
    pub vault_vusd: Account<'info, TokenAccount>,

    // YES mint, NO mint
    #[account(mut, address = proposal.yes_mint)] pub yes_mint: Account<'info, Mint>,
    #[account(mut, address = proposal.no_mint)]  pub no_mint:  Account<'info, Mint>,

    // user ATAs to receive inventory
    #[account(init_if_needed, payer = payer, associated_token::mint = yes_mint, associated_token::authority = payer)]
    pub user_yes: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = payer, associated_token::mint = no_mint,  associated_token::authority = payer)]
    pub user_no:  Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintYesNo<'info> {
    pub fn handler(&mut self, bumps: MintYesNoBumps, amount: u64) -> Result<()> {
        // move vUSD from user → vault
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    from:      self.user_vusd.to_account_info(),
                    to:        self.vault_vusd.to_account_info(),
                    authority: self.payer.to_account_info(),
                }),
            amount,
        )?;

        // mint YES
        for (mint, dest) in [
            (&self.yes_mint, &self.user_yes),
            (&self.no_mint,  &self.user_no),
        ] {
            mint_to(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    MintTo {
                        mint:      mint.to_account_info(),
                        to:        dest.to_account_info(),
                        authority: self.proposal_auth.to_account_info(),
                    },
                    &[&[b"proposal_auth", &[bumps.proposal_auth]]],
                ),
                amount,
            )?;
        }
        Ok(())
    }
}
