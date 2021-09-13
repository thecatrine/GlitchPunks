use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::invoke
};

use crate::{
    instruction::NiftInstruction,
    error::NiftError,
    state::Escrow,
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey, 
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = NiftInstruction::unpack(instruction_data)?;

        match instruction {
            NiftInstruction::InitEscrow { amount } => {
                msg!("Instruction: InitEscrow");
                Self::process_init_escrow(accounts, amount, program_id)
            },
            MintNFT => {
                msg!("Minting NFT");
                Self::process_mint_nft(accounts, program_id)
            }
        }
    }

    fn process_mint_nft(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let source_info = next_account_info(account_info_iter)?;
        let dest_info = next_account_info(account_info_iter)?;

        if *dest_info.key != *program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        **source_info.try_borrow_mut_lamports()? -= 5;
        // Deposit five lamports into the destination
        **dest_info.try_borrow_mut_lamports()? += 5;

        msg!("Sending lamports to contract");

        let token_program = next_account_info(account_info_iter)?;

        msg!("Would call to create NFT if this were real.");

        /*
        let rent = &Rent::from_account_info(
            next_account_info(account_info_iter)? // 4
        )?;

        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(NiftError::NotRentExempt.into());
        }

        // Initialize a "mint" for a single NFT
        let token_program = next_account_info(account_info_iter)?;
        let create_mint_ix = spl_token::instruction::initialize_mint(
            token_program.key,
        )
        */
        Ok(())
    }

    fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?; // 0

        // Check first account is signer
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Get temp account
        let temp_token_account = next_account_info(account_info_iter)?; // 1

        let token_to_receive_account = next_account_info(account_info_iter)?; // 2
        
        // ?
        if *token_to_receive_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let escrow_account = next_account_info(account_info_iter)?; // 3

        let rent = &Rent::from_account_info(
            next_account_info(account_info_iter)? // 4
        )?;

        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(NiftError::NotRentExempt.into());
        }

        // Unpack data from the field
        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;
        if escrow_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        escrow_info.is_initialized = true;
        escrow_info.initializer_pubkey = *initializer.key;
        escrow_info.temp_token_account_pubkey = *temp_token_account.key;
        escrow_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
        escrow_info.expected_amount = amount;

        // Pack into account?
        Escrow::pack(escrow_info, &mut escrow_account.data.borrow_mut())?;

        // ???
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

        // Call into token program
        let token_program = next_account_info(account_info_iter)?; // 5
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        )?;

        msg!("Calling the token program to transfer token account ownership...");
        invoke(
            &owner_change_ix,
            &[
                temp_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }
}
