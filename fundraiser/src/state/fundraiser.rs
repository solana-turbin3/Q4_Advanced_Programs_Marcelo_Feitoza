use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;
use pinocchio::ProgramResult;

pub struct Fundraiser(*mut u8);

impl Fundraiser {
    // Total = 8 + 8 + 32 + 32 + 8 + 1 + 1 = 90
    pub const LEN: usize = 8 // current_amount
        + 8  // time_started
        + 32 // maker
        + 32 // mint_to_raise
        + 8  // amount_to_raise
        + 1  // duration
        + 1; // bump

    #[inline(always)]
    pub fn init(&self, data: &[u8; Self::LEN]) -> ProgramResult {
        unsafe { *(self.0 as *mut [u8; Self::LEN]) = *data };
        Ok(())
    }

    #[inline(always)]
    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    pub fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        assert_eq!(*account_info.owner(), crate::ID);
        assert_eq!(account_info.data_len(), Self::LEN);
        Ok(Self::from_account_info_unchecked(account_info))
    }

    pub fn maker(&self) -> Pubkey {
        unsafe { *(self.0 as *const Pubkey) } // 0 - 32
    }
    // pub fn mint_to_raise(&self) -> Pubkey {
    //     unsafe { *(self.0.add(32) as *const Pubkey) } // 32 - 64
    // }
    // pub fn amount_to_raise(&self) -> u64 {
    //     unsafe { *(self.0.add(64) as *const u64) } // 64 - 72
    // }
    pub fn current_amount(&self) -> u64 {
        unsafe { *(self.0.add(72) as *const u64) } // 72 - 80
    }
    pub fn time_started(&self) -> i64 {
        unsafe { *(self.0.add(80) as *const i64) } // 80 - 88
    }
    pub fn duration(&self) -> u8 {
        unsafe { *(self.0.add(88) as *const u8) } // 88 - 89
    }
    pub fn bump(&self) -> u8 {
        unsafe { *(self.0.add(89) as *const u8) } // 89 - 90
    }
}
