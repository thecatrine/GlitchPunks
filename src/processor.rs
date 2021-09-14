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


// RSI vary this with number
const arweave_address: &str = "https://at53kwzgkaslzxnuzzwudu3c462njhwt5pyqnlreixreqo3i2i4q.arweave.net/BPu1WyZQJLzdtM5tQdNi57TUntPr8QauJEXiSDto0jk/punk_1.json";

const SOL_LAMPORTS: u64 = 1_000_000_000;

const FEE_LAMPORTS: u64 = 1000;

impl Processor {
    pub fn process(
        program_id: &Pubkey, 
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = NiftInstruction::unpack(instruction_data)?;

        match instruction {
            //NiftInstruction::InitEscrow { amount } => {
            //    msg!("Instruction: InitEscrow");
            //    Self::process_init_escrow(accounts, amount, program_id)
            //},
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
        let state = next_account_info(account_info_iter)?;

        let (mint_authority, mint_authority_bump_seed) = Pubkey::find_program_address(&[b"mint_authority"], program_id);
        msg!("mint authority");
        mint_authority.log();
 
        let expected_key = Pubkey::from_str("AuK2wzBzM5ZToXdoAigrKQHFVzZfavbzPo82NU2cawnj").unwrap();
        let expected_dest_key = Pubkey::from_str("7keeykNopXVgtLK97nCbarhaetE2351gZ8q7nzBnffJr").unwrap();
        
        let dest_key = *dest_info.key;
        dest_key.log();
        if dest_key != expected_dest_key {
            return Err(ProgramError::IncorrectProgramId);
        }

        let state_key = *state.key;
        state_key.log();
        if state_key != expected_key {
            return Err(ProgramError::IncorrectProgramId);
        }

        **source_info.try_borrow_mut_lamports()? -= FEE_LAMPORTS;
        // Deposit five lamports into the destination
        **dest_info.try_borrow_mut_lamports()? += FEE_LAMPORTS;

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
        msg!("Expected metadata key");
        msg!("{:?}", metadata_seeds);
        metadata_key.log();
        token_metadata_acct.key.log();

        ///////
        /// 
        /// DO DATA MUNGING
        ///////
        let mut nifty_state: NiftyState = NiftyState::try_from_slice(&state.data.borrow())?;
        if !nifty_state.is_initialized {
            // First we need to create the account
            nifty_state.is_initialized = true;
            nifty_state.next_num = 0;
        }
        msg!("Deserialized ok");

        let punk_num = nifty_state.next_num;

        nifty_state.next_num += 1;

        // Save again
        nifty_state.serialize(&mut &mut state.data.borrow_mut()[..])?;


        /// ////
        let save_metadata_ix = spl_token_metadata::instruction::create_metadata_accounts(
            *metadata_account.key,
            *token_metadata_acct.key,
            *mint_acct.key,
            *mint_authority_acct.key,
            *signer.key,
            *mint_authority_acct.key,
            format!("Punk {}", punk_num).to_string(),
            "".to_string(),
            arweave_address.to_string(),
            None, // TODO creators
            500,
            true,
            false,
        );
        msg!("Minting Punk {}", punk_num);
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
