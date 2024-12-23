use crate::constants::MIN_AMOUNT_TO_RAISE;
use crate::state::Fundraiser;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};
use pinocchio_token::instructions::Transfer;

pub fn process_contribute_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let amount: u64 = unsafe { *(data.as_ptr() as *const u64) };
    assert!(amount >= MIN_AMOUNT_TO_RAISE, "Amount too low");

    let [signer, contributor, signer_ta, fundraiser, vault, _token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let fundraiser_account = Fundraiser::from_account_info_unchecked(fundraiser);
    assert!(
        fundraiser_account.time_started() > 0,
        "Fundraiser not started yet"
    );
    assert!(
        fundraiser_account.time_started() + i64::from(fundraiser_account.duration()) > 0,
        "Fundraiser ended"
    );

    Transfer {
        from: signer_ta,
        to: vault,
        authority: signer,
        amount,
    }
    .invoke()?;

    unsafe {
        *(fundraiser.borrow_mut_data_unchecked().as_mut_ptr().add(72) as *mut u64) += amount;
        *(contributor.borrow_mut_data_unchecked().as_mut_ptr() as *mut u64) += amount;
    }

    Ok(())
}
