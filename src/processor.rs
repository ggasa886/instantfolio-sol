use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::{invoke},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    error::NameRegistryError,
    instruction::NameRegistryInstruction,
    state::{AddressAccount, NameAccount, PendingUpdateAccount, ProgramConfig},
    validation::*,
};

pub struct Processor;

impl Processor {
    pub fn process(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction: NameRegistryInstruction,
    ) -> ProgramResult {
        match instruction {
            NameRegistryInstruction::Initialize { registration_fee } => {
                Self::process_initialize(_program_id, accounts, registration_fee)
            }
            NameRegistryInstruction::RegisterName { name } => {
                Self::process_register_name(_program_id, accounts, name)
            }
            NameRegistryInstruction::RequestAddressUpdate { new_address } => {
                Self::process_request_address_update(_program_id, accounts, new_address)
            }
            NameRegistryInstruction::CompleteAddressUpdate => {
                Self::process_complete_address_update(_program_id, accounts)
            }
            NameRegistryInstruction::RenameName { new_name } => {
                Self::process_rename_name(_program_id, accounts, new_name)
            }
            NameRegistryInstruction::SetRegistrationFee { new_fee } => {
                Self::process_set_registration_fee(_program_id, accounts, new_fee)
            }
            NameRegistryInstruction::ChangeProgramOwner { new_owner } => {
                Self::process_change_program_owner(_program_id, accounts, new_owner)
            }
            NameRegistryInstruction::AcceptProgramOwnership => {
                Self::process_accept_program_ownership(_program_id, accounts)
            }
            NameRegistryInstruction::ResolveAddress => {
                Self::process_resolve_address(_program_id, accounts)
            }
            NameRegistryInstruction::GetContractOwner => {
                Self::process_get_contract_owner(_program_id, accounts)
            }
            NameRegistryInstruction::GetRegistrationFee => {
                Self::process_get_registration_fee(_program_id, accounts)
            }
            NameRegistryInstruction::GetPendingContractOwner => {
                Self::process_get_pending_contract_owner(_program_id, accounts)
            }
            NameRegistryInstruction::Withdraw => {
                Self::process_withdraw(_program_id, accounts)
            }
        }
    }

    fn process_initialize(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        registration_fee: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;
        let config_account = next_account_info(account_info_iter)?;
        let _system_program = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut config = ProgramConfig::unpack_unchecked(&config_account.data.borrow())?;
        if config.is_initialized {
            return Err(NameRegistryError::AlreadyInitialized.into());
        }

        config.is_initialized = true;
        config.owner = *initializer.key;
        config.pending_owner = Pubkey::default();
        config.registration_fee = registration_fee;

        ProgramConfig::pack(config, &mut config_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_register_name(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        name: String,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let registrant = next_account_info(account_info_iter)?;
        let name_account = next_account_info(account_info_iter)?;
        let address_account = next_account_info(account_info_iter)?;
        let config_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !registrant.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Verify system program
        if system_program.key != &solana_program::system_program::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        validate_name(&name)?;

        let config = ProgramConfig::unpack(&config_account.data.borrow())?;
        let registration_fee = config.registration_fee;

        let mut name_data = NameAccount::unpack_unchecked(&name_account.data.borrow())?;
        if name_data.is_initialized {
            return Err(NameRegistryError::NameTaken.into());
        }

        let mut address_data = AddressAccount::unpack_unchecked(&address_account.data.borrow())?;
        if address_data.is_initialized {
            return Err(NameRegistryError::NameAlreadyRegistered.into());
        }

        // Transfer registration fee from registrant to config account
        invoke(
            &system_instruction::transfer(
                registrant.key,
                config_account.key,
                registration_fee,
            ),
            &[registrant.clone(), config_account.clone()],
        )?;

        name_data.is_initialized = true;
        name_data.owner = *registrant.key;
        name_data.name = name.clone();
        name_data.address = *registrant.key;
        name_data.cooldown_until = Clock::get()?.unix_timestamp;

        address_data.is_initialized = true;
        address_data.name = name;

        NameAccount::pack(name_data, &mut name_account.data.borrow_mut())?;
        AddressAccount::pack(address_data, &mut address_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_request_address_update(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        new_address: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let current_owner = next_account_info(account_info_iter)?;
        let name_account = next_account_info(account_info_iter)?;
        let pending_update_account = next_account_info(account_info_iter)?;

        if !current_owner.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        validate_address(&new_address)?;

        let name_data = NameAccount::unpack(&name_account.data.borrow())?;
        validate_owner(&name_data.owner, current_owner.key)?;
        validate_cooldown(name_data.cooldown_until)?;

        let mut pending_update = PendingUpdateAccount::unpack_unchecked(&pending_update_account.data.borrow())?;
        pending_update.is_initialized = true;
        pending_update.new_address = new_address;

        PendingUpdateAccount::pack(pending_update, &mut pending_update_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_complete_address_update(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let new_owner = next_account_info(account_info_iter)?;
        let name_account = next_account_info(account_info_iter)?;
        let address_account = next_account_info(account_info_iter)?;
        let pending_update_account = next_account_info(account_info_iter)?;

        if !new_owner.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let pending_update = PendingUpdateAccount::unpack(&pending_update_account.data.borrow())?;
        if !pending_update.is_initialized {
            return Err(NameRegistryError::NoPendingUpdate.into());
        }

        if pending_update.new_address != *new_owner.key {
            return Err(NameRegistryError::NotPendingAddress.into());
        }

        let mut name_data = NameAccount::unpack(&name_account.data.borrow())?;
        let address_data = AddressAccount::unpack(&address_account.data.borrow())?;

        name_data.address = *new_owner.key;
        name_data.owner = *new_owner.key;
        name_data.cooldown_until = Clock::get()?.unix_timestamp;

        NameAccount::pack(name_data, &mut name_account.data.borrow_mut())?;
        AddressAccount::pack(address_data, &mut address_account.data.borrow_mut())?;

        // Clear pending update
        let mut pending_update = PendingUpdateAccount::unpack(&pending_update_account.data.borrow())?;
        pending_update.is_initialized = false;
        pending_update.new_address = Pubkey::default();
        PendingUpdateAccount::pack(pending_update, &mut pending_update_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_rename_name(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        new_name: String,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let current_owner = next_account_info(account_info_iter)?;
        let old_name_account = next_account_info(account_info_iter)?;
        let new_name_account = next_account_info(account_info_iter)?;
        let address_account = next_account_info(account_info_iter)?;

        if !current_owner.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        validate_name(&new_name)?;

        let old_name_data = NameAccount::unpack(&old_name_account.data.borrow())?;
        validate_owner(&old_name_data.owner, current_owner.key)?;
        validate_cooldown(old_name_data.cooldown_until)?;

        let new_name_data = NameAccount::unpack_unchecked(&new_name_account.data.borrow())?;
        if new_name_data.is_initialized {
            return Err(NameRegistryError::NameTaken.into());
        }

        let mut address_data = AddressAccount::unpack(&address_account.data.borrow())?;

        // Update new name account
        let mut new_name_data = NameAccount::default();
        new_name_data.is_initialized = true;
        new_name_data.owner = *current_owner.key;
        new_name_data.name = new_name.clone();
        new_name_data.address = old_name_data.address;
        new_name_data.cooldown_until = Clock::get()?.unix_timestamp;

        // Update address account
        address_data.name = new_name;

        // Clear old name account
        let mut old_name_data = NameAccount::unpack(&old_name_account.data.borrow())?;
        old_name_data.is_initialized = false;
        old_name_data.owner = Pubkey::default();
        old_name_data.name = String::new();
        old_name_data.address = Pubkey::default();
        old_name_data.cooldown_until = 0;

        NameAccount::pack(new_name_data, &mut new_name_account.data.borrow_mut())?;
        AddressAccount::pack(address_data, &mut address_account.data.borrow_mut())?;
        NameAccount::pack(old_name_data, &mut old_name_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_set_registration_fee(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        new_fee: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let owner = next_account_info(account_info_iter)?;
        let config_account = next_account_info(account_info_iter)?;

        if !owner.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut config = ProgramConfig::unpack(&config_account.data.borrow())?;
        validate_program_owner(&config.owner, owner.key)?;

        config.registration_fee = new_fee;
        ProgramConfig::pack(config, &mut config_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_change_program_owner(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        new_owner: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let current_owner = next_account_info(account_info_iter)?;
        let config_account = next_account_info(account_info_iter)?;

        if !current_owner.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        validate_address(&new_owner)?;

        let mut config = ProgramConfig::unpack(&config_account.data.borrow())?;
        validate_program_owner(&config.owner, current_owner.key)?;

        config.pending_owner = new_owner;
        ProgramConfig::pack(config, &mut config_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_accept_program_ownership(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let pending_owner = next_account_info(account_info_iter)?;
        let config_account = next_account_info(account_info_iter)?;

        if !pending_owner.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut config = ProgramConfig::unpack(&config_account.data.borrow())?;
        if config.pending_owner != *pending_owner.key {
            return Err(NameRegistryError::NotPendingContractOwner.into());
        }

        config.owner = *pending_owner.key;
        config.pending_owner = Pubkey::default();
        ProgramConfig::pack(config, &mut config_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_resolve_address(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let name_account = next_account_info(account_info_iter)?;

        let name_data = NameAccount::unpack(&name_account.data.borrow())?;
        if !name_data.is_initialized {
            return Err(NameRegistryError::NameNotFound.into());
        }

        // Return the address through program return data
        let return_data = name_data.address.to_bytes();
        solana_program::program::set_return_data(&return_data);

        Ok(())
    }

    fn process_get_contract_owner(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let config_account = next_account_info(account_info_iter)?;

        let config = ProgramConfig::unpack(&config_account.data.borrow())?;
        let return_data = config.owner.to_bytes();
        solana_program::program::set_return_data(&return_data);

        Ok(())
    }

    fn process_get_registration_fee(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let config_account = next_account_info(account_info_iter)?;

        let config = ProgramConfig::unpack(&config_account.data.borrow())?;
        let return_data = config.registration_fee.to_le_bytes();
        solana_program::program::set_return_data(&return_data);

        Ok(())
    }

    fn process_get_pending_contract_owner(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let config_account = next_account_info(account_info_iter)?;

        let config = ProgramConfig::unpack(&config_account.data.borrow())?;
        let return_data = config.pending_owner.to_bytes();
        solana_program::program::set_return_data(&return_data);

        Ok(())
    }

    fn process_withdraw(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let owner = next_account_info(account_info_iter)?;
        let config_account = next_account_info(account_info_iter)?;

        if !owner.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let config = ProgramConfig::unpack(&config_account.data.borrow())?;
        validate_program_owner(&config.owner, owner.key)?;

        // Transfer all lamports from config account to owner
        let config_lamports = config_account.lamports();
        if config_lamports == 0 {
            return Err(NameRegistryError::NothingToWithdraw.into());
        }

        **config_account.lamports.borrow_mut() = 0;
        **owner.lamports.borrow_mut() = owner.lamports().checked_add(config_lamports)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        Ok(())
    }
} 