use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum NameRegistryInstruction {
    /// Initialize the program
    /// Accounts expected:
    /// 0. `[signer]` The account of the person initializing the program
    /// 1. `[writable]` The program config account
    /// 2. `[]` The system program
    Initialize {
        registration_fee: u64,
    },

    /// Register a new name
    /// Accounts expected:
    /// 0. `[signer]` The account of the person registering the name
    /// 1. `[writable]` The name account
    /// 2. `[writable]` The address account
    /// 3. `[writable]` The program config account
    /// 4. `[]` The system program
    RegisterName {
        name: String,
    },

    /// Request an address update
    /// Accounts expected:
    /// 0. `[signer]` The current name owner
    /// 1. `[writable]` The name account
    /// 2. `[writable]` The pending update account
    RequestAddressUpdate {
        new_address: Pubkey,
    },

    /// Complete an address update
    /// Accounts expected:
    /// 0. `[signer]` The new address owner
    /// 1. `[writable]` The name account
    /// 2. `[writable]` The address account
    /// 3. `[writable]` The pending update account
    CompleteAddressUpdate,

    /// Rename a name
    /// Accounts expected:
    /// 0. `[signer]` The current name owner
    /// 1. `[writable]` The old name account
    /// 2. `[writable]` The new name account
    /// 3. `[writable]` The address account
    RenameName {
        new_name: String,
    },

    /// Update registration fee
    /// Accounts expected:
    /// 0. `[signer]` The program owner
    /// 1. `[writable]` The program config account
    SetRegistrationFee {
        new_fee: u64,
    },

    /// Change program owner
    /// Accounts expected:
    /// 0. `[signer]` The current program owner
    /// 1. `[writable]` The program config account
    ChangeProgramOwner {
        new_owner: Pubkey,
    },

    /// Accept program ownership
    /// Accounts expected:
    /// 0. `[signer]` The pending program owner
    /// 1. `[writable]` The program config account
    AcceptProgramOwnership,

    /// Resolve address by name
    /// Accounts expected:
    /// 0. `[]` The name account
    ResolveAddress,

    /// Get contract owner
    /// Accounts expected:
    /// 0. `[]` The program config account
    GetContractOwner,

    /// Get registration fee
    /// Accounts expected:
    /// 0. `[]` The program config account
    GetRegistrationFee,

    /// Get pending contract owner
    /// Accounts expected:
    /// 0. `[]` The program config account
    GetPendingContractOwner,

    /// Withdraw accumulated fees
    /// Accounts expected:
    /// 0. `[signer]` The program owner
    /// 1. `[writable]` The program config account
    Withdraw,
}

impl NameRegistryInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(input).map_err(|_| ProgramError::InvalidInstructionData)
    }
} 