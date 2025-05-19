use anchor_lang::error_code;

#[error_code]
pub enum QuantumError {
    #[msg("DefaultError")]
    DefaultError,
    #[msg("Overflow detected.")]
    Overflow,
}