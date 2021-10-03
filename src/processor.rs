use solana_program::{
    borsh,
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::{invoke, invoke_signed},
};

use ::borsh::{BorshSerialize, BorshDeserialize};

use crate::{
    instruction::NiftInstruction,
    error::NiftError,
    state::NiftyState,
};

pub struct Processor;

use std::str::FromStr;


const arweave_address: &str = "https://arweave.net/m2OjD2warNh2MaL-kv0z4ebI5I7gTY_wu60f0TQGTao";

const SOL_LAMPORTS: u64 = 1_000_000_000;
const FEE_LAMPORTS: u64 = 100_000_000;
const NFT_LIMIT: u64 = 1000;

impl Processor {
    pub fn process(
        program_id: &Pubkey, 
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = NiftInstruction::unpack(instruction_data)?;

        match instruction {
            //NiftInstruction::ExampleInstructionWithArgument { nonce } => {
            //    msg!("Instruction: ExampleInstructionWithArgument");
            //    Self::process_example_instruction_with_argument(
	    //        accounts,
	    //        nonce,
	    //        program_id,
	    //    )
            //},
            MintNFT => {
                msg!("Minting NFT");
                Self::process_mint_nft(accounts, program_id)
            }
        }
    }

    fn arweave_address_for_num(n: u64) -> String {
        format!(
            "{}/punk_{}.json",
            arweave_address,
            n,
        )
    }

    fn process_mint_nft(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        // Pubkey for cashier account that holds num of tokens issued
        let expected_key = Pubkey::from_str("BkHFN4TvyWgDx3UsaJJoUbnAi4uKAniBvvwQUPPe2UDo").unwrap();
        // Pubkey for payment destination of fees
        let expected_dest_key = Pubkey::from_str("7keeykNopXVgtLK97nCbarhaetE2351gZ8q7nzBnffJr").unwrap();

        let account_info_iter = &mut accounts.iter();

        let signer = next_account_info(account_info_iter)?; // signing transaction

        let mint_authority_acct = next_account_info(account_info_iter)?;

        let source_info = next_account_info(account_info_iter)?;
        
        let dest_info = next_account_info(account_info_iter)?;
        let state = next_account_info(account_info_iter)?;

        let (mint_authority, mint_authority_bump_seed) = Pubkey::find_program_address(&[b"mint_authority"], program_id);
        msg!("mint authority");
        mint_authority.log();
 
        // No trying to get us to transfer money out of our own accounts
        if *source_info.key == expected_key {
            msg!("Tried to steal from cashier");
            return Err(ProgramError::IncorrectProgramId);
        }
        if *source_info.key == *program_id {
            msg!("Tried to steal from program");
            return Err(ProgramError::IncorrectProgramId);
        }
        
        let dest_key = *dest_info.key;
        msg!("Dest account");
        dest_key.log();
        msg!("Expected");
        expected_dest_key.log();
        if dest_key != expected_dest_key {
            return Err(ProgramError::IncorrectProgramId);
        }

        let state_key = *state.key;
        msg!("State account");
        state_key.log();
        msg!("Expected");
        expected_key.log();
        if state_key != expected_key {
            return Err(ProgramError::IncorrectProgramId);
        }

        //
        // 
        // DO DATA MUNGING
        //
        let mut nifty_state: NiftyState = NiftyState::try_from_slice(&state.data.borrow())?;
        if !nifty_state.is_initialized {
            // First we need to create the account
            nifty_state.is_initialized = true;
            nifty_state.next_num = 1;
        }
        msg!("Deserialized ok");

        let punk_num = nifty_state.next_num;

        if punk_num <= NFT_LIMIT {
            nifty_state.next_num += 1;
        } else {
            return Err(NiftError::OutOfNFTs.into());
        }

        nifty_state.serialize(&mut &mut state.data.borrow_mut()[..])?;

        /////////
        //  CHARGE THE USER FOR THEIR NFT
        ////////

        // Will error if the user didn't actually pay
        **source_info.try_borrow_mut_lamports()? -= FEE_LAMPORTS;
        **dest_info.try_borrow_mut_lamports()? += FEE_LAMPORTS;

        //.checked_sub(amount)
        //.ok_or(TokenError::Overflow)?;

        msg!("Sending lamports to contract");

        let token_program = next_account_info(account_info_iter)?;
        let mint_acct = next_account_info(account_info_iter)?;

        msg!("Minting new NFT.");

        let rent_acct = next_account_info(account_info_iter)?;


        let create_mint_ix = spl_token::instruction::initialize_mint(
            token_program.key,
            mint_acct.key, // Mint location
            &mint_authority, // We're the authority,
            None,
            0,
        )?;
        invoke(
            &create_mint_ix,
            &[
                mint_acct.clone(),
                rent_acct.clone(),
            ],
        )?;
        msg!("Created mint.");

        let final_acct = next_account_info(account_info_iter)?;

        msg!("Creating token account");

        let initialize_account_ix = spl_token::instruction::initialize_account(
            token_program.key,
            final_acct.key,
            mint_acct.key,
            signer.key,
        )?;
        invoke(
            &initialize_account_ix,
            &[
                final_acct.clone(),
                mint_acct.clone(),
                signer.clone(),
                rent_acct.clone(),
            ],
        );

        
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
        msg!("Minted a single token");

        msg!("Now register metadata");
        let metadata_account = next_account_info(account_info_iter)?;

        let token_metadata_acct = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        // check token_metadata_acct is the real address
        let metadata_seeds = &[
            "metadata".as_bytes(),
            metadata_account.key.as_ref(),
            mint_acct.key.as_ref(),
        ];
        let (metadata_key, metadata_bump_seed) =
            Pubkey::find_program_address(metadata_seeds, metadata_account.key);

        /// ////

        msg!("Minting Punk {}", punk_num);
        let save_metadata_ix = spl_token_metadata::instruction::create_metadata_accounts(
            *metadata_account.key,
            *token_metadata_acct.key,
            *mint_acct.key,
            *mint_authority_acct.key,
            *signer.key,
            *mint_authority_acct.key,
            format!("Glitch Punk {}", punk_num).to_string(),
            "".to_string(),
            Self::arweave_address_for_num(punk_num),
            None,
            0,
            true,
            false,
        );
        invoke_signed(
            &save_metadata_ix,
            &[
                token_metadata_acct.clone(),
                mint_acct.clone(),
                mint_authority_acct.clone(),
                signer.clone(),
                mint_authority_acct.clone(),
                system_program.clone(),
                rent_acct.clone(),
            ],
            &[&[b"mint_authority", &[mint_authority_bump_seed]]],
        );

        msg!("Done");

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
}
