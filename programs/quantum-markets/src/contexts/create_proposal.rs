use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token, MintTo, mint_to};
use crate::state::config::MarketConfig;
use crate::state::global::GlobalState;
use crate::state::proposal::ProposalConfig;
use crate::state::deposit::DepositRecord;
use crate::errors::QuantumError;

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    // ============== input side ==============
    #[account(mut)]
    pub payer: Signer<'info>,

    /// market we’re proposing inside
    #[account(
        mut,
        seeds = [b"market", &market.id.to_le_bytes()],
        bump
    )]
    pub market: Account<'info, MarketConfig>,

    /// User’s deposit tracker (ATA of reward token)
    #[account(
        mut,
        seeds = [b"deposit", market.key().as_ref(), payer.key().as_ref()],
        bump,
    )]
    pub user_deposit: Account<'info, DepositRecord>,

    /// reward token mint (same as market.market_token)
    pub reward_mint: Account<'info, Mint>,

    // ============== global counter ==============
    #[account(
      mut,
      seeds = [b"global"],
      bump,
    )]
    pub global: Account<'info, GlobalState>,

    // ============== mints created for this proposal ==============
    #[account(
        init,
        payer = payer,
        seeds = [b"vusd".as_ref(), &global.next_id.to_le_bytes()],
        bump,
        mint::decimals = 6,
        mint::authority = proposal_auth
    )]
    pub vusd_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        seeds = [b"yes_mint".as_ref(), &global.next_id.to_le_bytes()],
        bump,
        mint::decimals = 0,
        mint::authority = proposal_auth
    )]
    pub yes_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        seeds = [b"no_mint".as_ref(), &global.next_id.to_le_bytes()],
        bump,
        mint::decimals = 0,
        mint::authority = proposal_auth
    )]
    pub no_mint: Account<'info, Mint>,

    // ============== vault ATAs (owned by proposal_auth) ==============
    #[account(
        init,
        payer = payer,
        associated_token::mint = vusd_mint,
        associated_token::authority = proposal_auth
    )]
    pub vusd_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = yes_mint,
        associated_token::authority = proposal_auth
    )]
    pub yes_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = no_mint,
        associated_token::authority = proposal_auth
    )]
    pub no_vault: Box<Account<'info, TokenAccount>>,

    // ============== user ATAs to receive inventory ==============
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = yes_mint,
        associated_token::authority = payer
    )]
    pub user_yes: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = no_mint,
        associated_token::authority = payer
    )]
    pub user_no: Box<Account<'info, TokenAccount>>,

    /// PDA that will sign `mint_to` CPIs
    /// CHECK: only used as mint_authority
    #[account(
        seeds = [b"proposal_auth"],
        bump
    )]
    pub proposal_auth: UncheckedAccount<'info>,

    // ============== new ProposalConfig PDA ==============
    #[account(
        init,
        payer = payer,
        seeds = [b"proposal", &global.next_id.to_le_bytes()],
        bump,
        space = 8 + ProposalConfig::SIZE
    )]
    pub proposal: Account<'info, ProposalConfig>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateProposal<'info> {
    pub fn handler(
        &mut self,
        bumps: CreateProposalBumps,     // { proposal_auth, proposal, vusd_mint, yes_mint, … }
        data: Vec<u8>,                 // opaque proposal blob
    ) -> Result<()> {
        // 1) ensure caller has enough un-claimed deposit
        let min_d = self.market.min_deposit;
        let claimable = self.user_deposit.amount;     // simplistic: all tokens in this ATA
        require!(claimable >= min_d, QuantumError::MinDeposit);

        // burn (lock) the minDeposit from user_deposit
        self.user_deposit.amount = claimable
            .checked_sub(min_d)
            .ok_or(QuantumError::Underflow)?;

        // 2) split D (min_d) exactly like Solidity logic
        let burn_total = min_d * 2 / 3;          // D * 2/3  as YES+NO
        let token_per_pool = burn_total / 2;     // D/3
        let vusd_to_mint = min_d - burn_total;   // D/3
        let vusd_per_pool = vusd_to_mint / 2;    // D/6   (we just mint D/3 total)

        // 3) mint vUSD into vault
        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.vusd_mint.to_account_info(),
                    to: self.vusd_vault.to_account_info(),
                    authority: self.proposal_auth.to_account_info(),
                },
                &[&[b"proposal_auth", &[bumps.proposal_auth]]],
            ),
            vusd_to_mint,
        )?;


        // 4) mint YES+NO into vault AND to caller
        let mint_yes = |to: &Account<'info, TokenAccount>, amount| -> Result<()> {
            mint_to(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    MintTo {
                        mint: self.yes_mint.to_account_info(),
                        to: to.to_account_info(),
                        authority: self.proposal_auth.to_account_info(),
                    },
                    &[&[b"proposal_auth", &[bumps.proposal_auth]]],
                ),
                amount,
            )
        };
        let mint_no = |to: &Account<'info, TokenAccount>, amount| -> Result<()> {
            mint_to(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    MintTo {
                        mint: self.no_mint.to_account_info(),
                        to: to.to_account_info(),
                        authority: self.proposal_auth.to_account_info(),
                    },
                    &[&[b"proposal_auth", &[bumps.proposal_auth]]],
                ),
                amount,
            )
        };

        mint_yes(&self.yes_vault, token_per_pool)?;
        mint_no(&self.no_vault, token_per_pool)?;
        mint_yes(&self.user_yes, token_per_pool)?;
        mint_no(&self.user_no, token_per_pool)?;

        // 5) record ProposalConfig
        let id = self.global.next_id;
        let now = Clock::get()?.unix_timestamp;
        self.proposal.set_inner(ProposalConfig {
            id,
            market_id: self.market.id,
            created_at: now,
            creator: self.payer.key(),
            vusd_mint: self.vusd_mint.key(),
            yes_mint:  self.yes_mint.key(),
            no_mint:   self.no_mint.key(),
            yes_pool:  Pubkey::default(), // will be filled after pool bootstrap
            no_pool:   Pubkey::default(),
            data,
            bump: bumps.proposal,
        });

        // 6) bump the global counter
        self.global.next_id = id
            .checked_add(1)
            .ok_or(QuantumError::Overflow)?;

        Ok(())
    }
}
