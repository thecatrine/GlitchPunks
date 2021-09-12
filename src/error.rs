use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum NiftError {
    #[error("Invalid Instruction")],
    InvalidInstruction,
}

impl From<NiftError> for ProgramError {
    fn from(e: NiftError) -> Self {
        ProgramError::Custom(e as u32)
    }
}