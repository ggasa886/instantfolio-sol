use solana_program::{
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized, Sealed},
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct NameAccount {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub name: String,
    pub address: Pubkey,
    pub cooldown_until: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct AddressAccount {
    pub is_initialized: bool,
    pub name: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct PendingUpdateAccount {
    pub is_initialized: bool,
    pub new_address: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct ProgramConfig {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub pending_owner: Pubkey,
    pub registration_fee: u64,
}

impl Sealed for NameAccount {}
impl Sealed for AddressAccount {}
impl Sealed for PendingUpdateAccount {}
impl Sealed for ProgramConfig {}

impl IsInitialized for NameAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for AddressAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for PendingUpdateAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for ProgramConfig {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for NameAccount {
    const LEN: usize = 1 + 32 + 32 + 32 + 8 + 4; // is_initialized + owner + name (max 32) + address + cooldown + name length prefix

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let data = self.try_to_vec().unwrap();
        dst[..data.len()].copy_from_slice(&data);
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(src).map_err(|_| ProgramError::InvalidAccountData)
    }
}

impl Pack for AddressAccount {
    const LEN: usize = 1 + 4 + 32; // is_initialized + name length prefix + name (max 32)

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let data = self.try_to_vec().unwrap();
        dst[..data.len()].copy_from_slice(&data);
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(src).map_err(|_| ProgramError::InvalidAccountData)
    }
}

impl Pack for PendingUpdateAccount {
    const LEN: usize = 1 + 32; // is_initialized + new_address

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let data = self.try_to_vec().unwrap();
        dst[..data.len()].copy_from_slice(&data);
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(src).map_err(|_| ProgramError::InvalidAccountData)
    }
}

impl Pack for ProgramConfig {
    const LEN: usize = 1 + 32 + 32 + 8; // is_initialized + owner + pending_owner + fee

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let data = self.try_to_vec().unwrap();
        dst[..data.len()].copy_from_slice(&data);
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(src).map_err(|_| ProgramError::InvalidAccountData)
    }
} 