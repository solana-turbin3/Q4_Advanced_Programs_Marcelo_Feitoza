use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::clock::Clock,
    ProgramResult,
};
use pinocchio::sysvars::Sysvar;
use crate::constants::{MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS};
use crate::errors::FundraiserError;
use crate::state::{Fundraiser, Contributor};

pub fn process_contribute_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [contributor, mint_to_raise, fundraiser, contributor_account, contributor_ata, vault, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let amount = unsafe { *(data.as_ptr() as *const u64) };

    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let fundraiser_data = Fundraiser::from_account_info(fundraiser)?;

    if fundraiser_data.mint_to_raise() != *mint_to_raise.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    let min_contribution = 1_u64;
    if amount < min_contribution {
        return Err(ProgramError::Custom(FundraiserError::ContributionTooSmall as u32));
    }
    let max_contribution = (fundraiser_data.amount_to_raise() * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER;
    if amount > max_contribution {
        return Err(ProgramError::Custom(FundraiserError::ContributionTooBig as u32));
    }

    let clock = unsafe { *(Clock::get().borrow().as_ptr() as *const Clock) };
    let current_time = clock.unix_timestamp;
    let fundraiser_end_time = fundraiser_data.time_started() + fundraiser_data.duration() as i64 * SECONDS_TO_DAYS;
    if current_time > fundraiser_end_time {
        return Err(ProgramError::Custom(FundraiserError::FundraiserEnded as u32));
    }

    let cpi_accounts = spl_token::instruction::Transfer {
        from: contributor_ata.clone(),
        to: vault.clone(),
        authority: contributor.clone(),
    };
    let cpi_program = token_program.clone();

    let seeds = &[b"fundraiser", fundraiser.key().as_ref(), &[fundraiser_data.bump()]];
    let signer_seeds = &[&seeds[..]];
    let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    transfer(cpi_context)?;

    fundraiser_data.current_amount() += amount;
    contributor_account.amount() += amount;

    Ok(())
}