use solana_program::{
    program_error::ProgramError,
    sysvar::Sysvar,
    clock::Clock,
};
use crate::error::NameRegistryError;

pub const MAX_NAME_LENGTH: usize = 32;

pub fn validate_name(name: &str) -> Result<(), ProgramError> {
    if name.is_empty() {
        return Err(NameRegistryError::InvalidNameFormat.into());
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(NameRegistryError::InvalidNameFormat.into());
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(NameRegistryError::InvalidNameFormat.into());
    }
    Ok(())
}

pub fn validate_address(address: &solana_program::pubkey::Pubkey) -> Result<(), ProgramError> {
    if address == &solana_program::pubkey::Pubkey::default() {
        return Err(NameRegistryError::InvalidAddress.into());
    }
    Ok(())
}

pub fn validate_cooldown(cooldown_until: i64) -> Result<(), ProgramError> {
    let clock = Clock::get()?;
    if clock.unix_timestamp < cooldown_until {
        return Err(NameRegistryError::CooldownNotOver.into());
    }
    Ok(())
}

pub fn get_cooldown_until() -> Result<i64, ProgramError> {
    let current_time = Clock::get()?.unix_timestamp;
    Ok(current_time + 86400) // 1 day in seconds
}

pub fn validate_owner(owner: &solana_program::pubkey::Pubkey, signer: &solana_program::pubkey::Pubkey) -> Result<(), ProgramError> {
    if owner != signer {
        return Err(NameRegistryError::NotNameOwner.into());
    }
    Ok(())
}

pub fn validate_program_owner(owner: &solana_program::pubkey::Pubkey, signer: &solana_program::pubkey::Pubkey) -> Result<(), ProgramError> {
    if owner != signer {
        return Err(NameRegistryError::NotContractOwner.into());
    }
    Ok(())
} 