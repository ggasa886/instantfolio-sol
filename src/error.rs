use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum NameRegistryError {
    #[error("Invalid name format")]
    InvalidNameFormat,
    
    #[error("Name already taken")]
    NameTaken,
    
    #[error("Insufficient fee")]
    InsufficientFee,
    
    #[error("Name already registered for address")]
    NameAlreadyRegistered,
    
    #[error("Not name owner")]
    NotNameOwner,
    
    #[error("Invalid address")]
    InvalidAddress,
    
    #[error("Cooldown period not over")]
    CooldownNotOver,
    
    #[error("No pending update")]
    NoPendingUpdate,
    
    #[error("Not the pending address")]
    NotPendingAddress,
    
    #[error("Not contract owner")]
    NotContractOwner,
    
    #[error("Invalid new owner")]
    InvalidNewOwner,
    
    #[error("Not the pending contract owner")]
    NotPendingContractOwner,
    
    #[error("Account not initialized")]
    NotInitialized,
    
    #[error("Account already initialized")]
    AlreadyInitialized,
    
    #[error("Name not found")]
    NameNotFound,
    
    #[error("Nothing to withdraw")]
    NothingToWithdraw,
}

impl From<NameRegistryError> for ProgramError {
    fn from(e: NameRegistryError) -> Self {
        ProgramError::Custom(e as u32)
    }
} 