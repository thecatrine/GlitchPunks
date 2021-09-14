use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::{invoke, invoke_signed},
};

use crate::{
    instruction::NiftInstruction,
    error::NiftError,
    state::Escrow,
};

pub struct Processor;

use std::str::FromStr;


const arweave_address: &str = "https://u25ca6rd2tvqvldxsnvbjfabm5qtve7iidmcmbeu2ukfunyg3xpq.arweave.net/progeiPU6wqsd5NqFJQBZ2E6k-hA2CYElNUUWjcG3d8/";

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

        let signer = next_account_info(account_info_iter)?; // signing transaction

        let mint_authority_acct = next_account_info(account_info_iter)?;

        let source_info = next_account_info(account_info_iter)?;
        let dest_info = next_account_info(account_info_iter)?;

        let (mint_authority, mint_authority_bump_seed) = Pubkey::find_program_address(&[b"mint_authority"], program_id);
        msg!("mint authority");
        mint_authority.log();
 
        let expected_key = Pubkey::from_str("7EEtiweAtCmqiEw6UefkEdiPNPSjV5ssgEgv7ynyPon6").unwrap();
        expected_key.log();
        
        let dest_key = *dest_info.key;
        dest_key.log();
        if *dest_info.key != expected_key {
            return Err(ProgramError::IncorrectProgramId);
        }

        **source_info.try_borrow_mut_lamports()? -= 1000;
        // Deposit five lamports into the destination
        **dest_info.try_borrow_mut_lamports()? += 1000;

        // RSI remember to actually error if you don't pay

        msg!("Sending lamports to contract");

        let token_program = next_account_info(account_info_iter)?;
        let mint_acct = next_account_info(account_info_iter)?;
        msg!("Would call to create NFT if this were real.");

        // RSI have this actually increment


        let rent_acct = next_account_info(account_info_iter)?;


        let create_mint_ix = spl_token::instruction::initialize_mint(
            token_program.key,
            mint_acct.key, // Mint location
            &mint_authority, // We're the authority,
            None,
            0,
        )?;
        msg!("Calling token program to create mint");
        invoke(
            &create_mint_ix,
            &[
                mint_acct.clone(),
                rent_acct.clone(),
            ],
        )?;
        msg!("Created mint.");

        let final_acct = next_account_info(account_info_iter)?;

        let initialize_account_ix = spl_token::instruction::initialize_account(
            token_program.key,
            final_acct.key,
            mint_acct.key,
            signer.key, // It's owned by the person who initially started it.
        )?;
        msg!("Calling token program to initialize account");
        invoke(
            &initialize_account_ix,
            &[
                final_acct.clone(),
                mint_acct.clone(),
                signer.clone(),
                rent_acct.clone(),
            ],
        );

        msg!("Minting a single token");
        let mint_nft_ix = spl_token::instruction::mint_to(
            token_program.key,
            mint_acct.key,
            final_acct.key,
            mint_authority_acct.key,
            &[],
            1,
        )?;
        invoke_signed(
            &mint_nft_ix,
            &[
                mint_acct.clone(),
                final_acct.clone(),
                mint_authority_acct.clone(), // signing authority
            ],
            &[&[b"mint_authority", &[mint_authority_bump_seed]]],
        );

        msg!("Would then register metadata.");


        /*
        let rent = &Rent::from_account_info(
            next_account_info(account_info_iter)? // 4
        )?;

        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(NiftError::NotRentExempt.into());
        }

        // Initialize a "mint" for a single NFT
        let token_program = next_account_info(account_info_iter)?;
        
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
