use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum NiftError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Not Rent Exempt")]
    NotRentExempt,
}

impl From<NiftError> for ProgramError {
    fn from(e: NiftError) -> Self {
        ProgramError::Custom(e as u32)
    }
}