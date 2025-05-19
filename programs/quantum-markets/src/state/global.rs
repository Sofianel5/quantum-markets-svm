use anchor_lang::prelude::*;

#[account]
pub struct GlobalState {
    pub next_id: u64,
}
