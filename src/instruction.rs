use std::convert::TryInto;
use solana_program::program_error::ProgramError;

use crate::error::NiftError::InvalidInstruction;

pub enum NiftInstruction {
    MintNFT {},
    //ExampleInstructionWithArgument {
    //    nonce: u64
    //},

}

impl NiftInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            //0 => Self::ExampleInstructoinWithArgument {
            //    nonce: Self::unpack_nonce(rest)?,
            //},
	    // Self::unpack_nonce would be a function that decodes a &[u8] to produce a u64
            1 => Self::MintNFT {

            },
            _ => return Err(InvalidInstruction.into()),
        })
    }
}

