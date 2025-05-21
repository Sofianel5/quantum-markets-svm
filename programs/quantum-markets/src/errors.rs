use anchor_lang::error_code;

#[error_code]
pub enum QuantumError {
    #[msg("DefaultError")]
    DefaultError,
    #[msg("Overflow detected.")]
    Overflow,
    #[msg("Underflow detected.")]
    Underflow,
    #[msg("Deposit is too low.")]
    MinDeposit,
    #[msg("Market is closed.")]
    MarketClosed,
    #[msg("Nothing to claim.")]
    NothingToClaim
}