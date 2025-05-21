use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, MintTo, mint_to};
use crate::state::proposal::ProposalConfig;
use crate::state::deposit::{DepositRecord, ClaimRecord};

#[derive(Accounts)]
pub struct ClaimForProposal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [b"proposal", &proposal.id.to_le_bytes()],
        bump,
    )]
    pub proposal: Account<'info, ProposalConfig>,

    #[account(
        seeds = [b"market", &proposal.market_id.to_le_bytes()],
        bump,
    )]
    pub market: Account<'info, crate::state::config::MarketConfig>,

    /// User’s cumulative deposit in that market
    #[account(
        seeds = [b"deposit", market.key().as_ref(), payer.key().as_ref()],
        bump,
    )]
    pub deposit_record: Account<'info, DepositRecord>,

    /// How much already claimed into this proposal
    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"claim", proposal.key().as_ref(), payer.key().as_ref()],
        bump,
        space = ClaimRecord::SIZE
    )]
    pub claim_record: Account<'info, ClaimRecord>,

    /// vUSD mint of this proposal
    #[account(mut, address = proposal.vusd_mint)]
    pub vusd_mint: Account<'info, Mint>,

    /// User’s vUSD ATA
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = vusd_mint,
        associated_token::authority = payer
    )]
    pub user_vusd: Account<'info, TokenAccount>,

    /// Authority PDA allowed to mint vUSD
    /// CHECK: signer via seeds
    #[account(seeds = [b"proposal_auth"], bump)]
    pub proposal_auth: UncheckedAccount<'info>,

    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimForProposal<'info> {
    pub fn handler(&mut self, bumps: ClaimForProposalBumps) -> Result<()> {
        let total = self.deposit_record.amount;
        let claimed = self.claim_record.claimed;
        let claimable = total
            .checked_sub(claimed)
            .ok_or(crate::errors::QuantumError::Overflow)?;
        require!(claimable > 0, crate::errors::QuantumError::NothingToClaim);

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint:      self.vusd_mint.to_account_info(),
                    to:        self.user_vusd.to_account_info(),
                    authority: self.proposal_auth.to_account_info(),
                },
                &[&[b"proposal_auth", &[bumps.proposal_auth]]],
            ),
            claimable,
        )?;

        self.claim_record.claimed = total; // now fully claimed
        Ok(())
    }
}
