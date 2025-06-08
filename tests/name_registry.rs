use solana_program::{
    instruction::AccountMeta,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::Instruction,
};
use borsh::BorshSerialize;
use instant_folio::{
    instruction::NameRegistryInstruction,
    state::{AddressAccount, NameAccount, ProgramConfig},
};

const REGISTRATION_FEE: u64 = 1_000_000; // 0.001 SOL
const HIGH_FEE: u64 = 2_000_000; // 0.002 SOL
const COOLDOWN_PERIOD: i64 = 86400; // 1 day in seconds

async fn setup_program() -> (ProgramTestContext, Keypair, Keypair, Pubkey) {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "instant_folio",
        program_id,
        processor!(instant_folio::process_instruction),
    );

    let initializer = Keypair::new();
    let config_account = Keypair::new();

    program_test.add_account(
        initializer.pubkey(),
        Account {
            lamports: 1_000_000_000,
            owner: solana_program::system_program::id(),
            ..Account::default()
        },
    );

    program_test.add_account(
        config_account.pubkey(),
        Account {
            lamports: 0,
            owner: program_id,
            ..Account::default()
        },
    );

    let context = program_test.start_with_context().await;
    (context, initializer, config_account, program_id)
}

fn convert_instruction(
    ix: NameRegistryInstruction,
    program_id: &Pubkey,
    accounts: &[(&Keypair, bool)],
    system_program: &Pubkey,
) -> Instruction {
    let mut account_metas = accounts
        .iter()
        .map(|(keypair, is_signer)| {
            AccountMeta::new(
                keypair.pubkey(),
                *is_signer,
            )
        })
        .collect::<Vec<_>>();

    // Add system program if needed
    account_metas.push(AccountMeta::new_readonly(*system_program, false));

    Instruction {
        program_id: *program_id,
        accounts: account_metas,
        data: ix.try_to_vec().unwrap(),
    }
}

async fn initialize_program(
    context: &mut ProgramTestContext,
    program_id: &Pubkey,
    initializer: &Keypair,
    config_account: &Keypair,
    registration_fee: u64,
) {
    // Create initialize instruction
    let instruction = NameRegistryInstruction::Initialize {
        registration_fee,
    };

    // Create transaction
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            instruction,
            program_id,
            &[
                (&initializer, true),
                (&config_account, false),
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();
}

async fn register_name(
    context: &mut ProgramTestContext,
    program_id: &Pubkey,
    registrant: &Keypair,
    name_account: &Keypair,
    address_account: &Keypair,
    config_account: &Keypair,
    name: String,
) {
    // Create register name instruction
    let instruction = NameRegistryInstruction::RegisterName { name };

    // Create transaction
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            instruction,
            program_id,
            &[
                (&registrant, true),
                (&name_account, false),
                (&address_account, false),
                (&config_account, false),
            ],
            &solana_program::system_program::id(),
        )],
        Some(&registrant.pubkey()),
    );
    transaction.sign(&[&registrant], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();
}

async fn add_account(
    context: &mut ProgramTestContext,
    keypair: &Keypair,
    owner: &Pubkey,
    lamports: u64,
    account_type: &str,
) {
    let space = match account_type {
        "config" => ProgramConfig::LEN,
        "name" => NameAccount::LEN,
        "address" => AddressAccount::LEN,
        "pending_update" => PendingUpdateAccount::LEN,
        _ => panic!("Unknown account type: {}", account_type),
    };
    
    let create_account_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &keypair.pubkey(),
        lamports,
        space as u64,
        owner,
    );

    let mut transaction = Transaction::new_with_payer(
        &[create_account_ix],
        Some(&context.payer.pubkey()),
    );
    transaction.sign(&[&context.payer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();
}

#[tokio::test]
async fn test_initialize() {
    let (mut context, initializer, config_account, program_id) = setup_program().await;

    // Initialize program
    initialize_program(&mut context, &program_id, &initializer, &config_account, REGISTRATION_FEE).await;

    // Create name and address accounts
    let name_account = Keypair::new();
    let address_account = Keypair::new();
    add_account(&mut context, &name_account, &program_id, 0, "name").await;
    add_account(&mut context, &address_account, &program_id, 0, "address").await;

    // Verify config account
    let config_account = context
        .banks_client
        .get_account(config_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let config = ProgramConfig::unpack(&config_account.data).unwrap();
    assert!(config.is_initialized);
    assert_eq!(config.owner, initializer.pubkey());
    assert_eq!(config.registration_fee, REGISTRATION_FEE);
}

#[tokio::test]
async fn test_register_name() {
    let (mut context, initializer, config_account, program_id) = setup_program().await;

    // Initialize program
    initialize_program(&mut context, &program_id, &initializer, &config_account, REGISTRATION_FEE).await;

    // Create name and address accounts
    let name_account = Keypair::new();
    let address_account = Keypair::new();
    add_account(&mut context, &name_account, &program_id, 0, "name").await;
    add_account(&mut context, &address_account, &program_id, 0, "address").await;

    // Register name
    let instruction = NameRegistryInstruction::RegisterName {
        name: "test-name".to_string(),
    };

    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            instruction,
            &program_id,
            &[
                (&initializer, true),
                (&name_account, false),
                (&address_account, false),
                (&config_account, false),
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify name account
    let name_account = context
        .banks_client
        .get_account(name_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let name_data = NameAccount::unpack(&name_account.data).unwrap();
    assert!(name_data.is_initialized);
    assert_eq!(name_data.owner, initializer.pubkey());
    assert_eq!(name_data.name, "test-name");
    assert_eq!(name_data.address, initializer.pubkey());

    // Verify address account
    let address_account = context
        .banks_client
        .get_account(address_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let address_data = AddressAccount::unpack(&address_account.data).unwrap();
    assert!(address_data.is_initialized);
    assert_eq!(address_data.name, "test-name");
}

#[tokio::test]
async fn test_request_address_update() {
    let (mut context, initializer, config_account, program_id) = setup_program().await;

    // Initialize program
    initialize_program(&mut context, &program_id, &initializer, &config_account, REGISTRATION_FEE).await;

    // Create name and address accounts
    let name_account = Keypair::new();
    let address_account = Keypair::new();
    add_account(&mut context, &name_account, &program_id, 0, "name").await;
    add_account(&mut context, &address_account, &program_id, 0, "address").await;

    // Register name
    register_name(
        &mut context,
        &program_id,
        &initializer,
        &name_account,
        &address_account,
        &config_account,
        "test-name".to_string(),
    ).await;

    // Create new owner
    let new_owner = Keypair::new();
    add_account(&mut context, &new_owner, &program_id, 0, "name").await;

    // Create pending update account
    let pending_update_account = Keypair::new();
    add_account(&mut context, &pending_update_account, &program_id, 0, "pending_update").await;

    // Request address update
    let instruction = NameRegistryInstruction::RequestAddressUpdate {
        new_address: new_owner.pubkey(),
    };

    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            instruction,
            &program_id,
            &[
                (&initializer, true),  // [signer] current name owner
                (&name_account, false),  // [writable] name account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify name account
    let name_account = context
        .banks_client
        .get_account(name_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let name_data = NameAccount::unpack(&name_account.data).unwrap();
    assert_eq!(name_data.address, new_owner.pubkey());
}

#[tokio::test]
async fn test_complete_address_update() {
    let (mut context, initializer, config_account, program_id) = setup_program().await;

    // Initialize program
    initialize_program(&mut context, &program_id, &initializer, &config_account, REGISTRATION_FEE).await;

    // Create name and address accounts
    let name_account = Keypair::new();
    let address_account = Keypair::new();
    add_account(&mut context, &name_account, &program_id, 0, "name").await;
    add_account(&mut context, &address_account, &program_id, 0, "address").await;

    // Register name
    register_name(
        &mut context,
        &program_id,
        &initializer,
        &name_account,
        &address_account,
        &config_account,
        "test-name".to_string(),
    ).await;

    // Create new owner
    let new_owner = Keypair::new();
    add_account(&mut context, &new_owner, &program_id, 0, "name").await;

    // Create pending update account
    let pending_update_account = Keypair::new();
    add_account(&mut context, &pending_update_account, &program_id, 0, "pending_update").await;

    // Request address update
    let request_ix = NameRegistryInstruction::RequestAddressUpdate {
        new_address: new_owner.pubkey(),
    };

    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            request_ix,
            &program_id,
            &[
                (&initializer, true),  // [signer] current name owner
                (&name_account, false),  // [writable] name account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Complete address update
    let complete_ix = NameRegistryInstruction::CompleteAddressUpdate;

    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            complete_ix,
            &program_id,
            &[
                (&new_owner, true),  // [signer] new owner
                (&name_account, false),  // [writable] name account
                (&address_account, false),  // [writable] address account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&new_owner.pubkey()),
    );
    transaction.sign(&[&new_owner], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify name account
    let name_account = context
        .banks_client
        .get_account(name_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let name_data = NameAccount::unpack(&name_account.data).unwrap();
    assert_eq!(name_data.address, new_owner.pubkey());
}

#[tokio::test]
async fn test_rename_name() {
    let (mut context, initializer, config_account, program_id) = setup_program().await;

    // Initialize program
    initialize_program(&mut context, &program_id, &initializer, &config_account, REGISTRATION_FEE).await;

    // Create name and address accounts
    let name_account = Keypair::new();
    let address_account = Keypair::new();
    add_account(&mut context, &name_account, &program_id, 0, "name").await;
    add_account(&mut context, &address_account, &program_id, 0, "address").await;

    // Register name
    register_name(
        &mut context,
        &program_id,
        &initializer,
        &name_account,
        &address_account,
        &config_account,
        "test-name".to_string(),
    ).await;

    // Create new name account
    let new_name_account = Keypair::new();
    add_account(&mut context, &new_name_account, &program_id, 0, "name").await;

    // Create pending update account
    let pending_update_account = Keypair::new();
    add_account(&mut context, &pending_update_account, &program_id, 0, "pending_update").await;

    // Rename name
    let instruction = NameRegistryInstruction::RenameName {
        new_name: "new-test-name".to_string(),
    };

    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            instruction,
            &program_id,
            &[
                (&initializer, true),  // [signer] current name owner
                (&name_account, false),  // [writable] old name account
                (&new_name_account, false),  // [writable] new name account
                (&address_account, false),  // [writable] address account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify new name account
    let new_name_account = context
        .banks_client
        .get_account(new_name_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let name_data = NameAccount::unpack(&new_name_account.data).unwrap();
    assert!(name_data.is_initialized);
    assert_eq!(name_data.owner, initializer.pubkey());
    assert_eq!(name_data.name, "new-test-name");
    assert_eq!(name_data.address, initializer.pubkey());

    // Verify address account
    let address_account = context
        .banks_client
        .get_account(address_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let address_data = AddressAccount::unpack(&address_account.data).unwrap();
    assert_eq!(address_data.name, "new-test-name");
}

#[tokio::test]
async fn test_set_registration_fee() {
    let (mut context, initializer, config_account, program_id) = setup_program().await;

    // Initialize program
    initialize_program(&mut context, &program_id, &initializer, &config_account, REGISTRATION_FEE).await;

    // Create pending update account
    let pending_update_account = Keypair::new();
    add_account(&mut context, &pending_update_account, &program_id, 0, "pending_update").await;

    // Set new fee
    let new_fee = 2_000_000; // 0.002 SOL
    let set_fee_ix = NameRegistryInstruction::SetRegistrationFee {
        new_fee,
    };
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            set_fee_ix,
            &program_id,
            &[
                (&initializer, true),  // [signer] program owner
                (&config_account, false),  // [writable] config account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify config account
    let config_account = context
        .banks_client
        .get_account(config_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let config = ProgramConfig::unpack(&config_account.data).unwrap();
    assert_eq!(config.registration_fee, new_fee);
}

#[tokio::test]
async fn test_change_program_owner() {
    let (mut context, initializer, config_account, program_id) = setup_program().await;

    // Initialize program
    initialize_program(&mut context, &program_id, &initializer, &config_account, REGISTRATION_FEE).await;

    // Create new owner
    let new_owner = Keypair::new();
    add_account(&mut context, &new_owner, &program_id, 10_000_000_000, "name").await;

    // Create pending update account
    let pending_update_account = Keypair::new();
    add_account(&mut context, &pending_update_account, &program_id, 0, "pending_update").await;

    // Change owner
    let change_owner_ix = NameRegistryInstruction::ChangeProgramOwner {
        new_owner: new_owner.pubkey(),
    };
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            change_owner_ix,
            &program_id,
            &[
                (&initializer, true),  // [signer] current owner
                (&config_account, false),  // [writable] config account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify config account
    let config_account_data = context
        .banks_client
        .get_account(config_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let config = ProgramConfig::unpack(&config_account_data.data).unwrap();
    assert_eq!(config.pending_owner, new_owner.pubkey());

    // Accept ownership
    let accept_ix = NameRegistryInstruction::AcceptProgramOwnership;
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            accept_ix,
            &program_id,
            &[
                (&new_owner, true),  // [signer] new owner
                (&config_account, false),  // [writable] config account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&new_owner.pubkey()),
    );
    transaction.sign(&[&new_owner], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify config account again
    let config_account_data = context
        .banks_client
        .get_account(config_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let config = ProgramConfig::unpack(&config_account_data.data).unwrap();
    assert_eq!(config.owner, new_owner.pubkey());
    assert_eq!(config.pending_owner, Pubkey::default());
}

#[tokio::test]
async fn test_resolve_address() {
    let (mut context, initializer, config_account, program_id) = setup_program().await;

    // Initialize program
    initialize_program(&mut context, &program_id, &initializer, &config_account, REGISTRATION_FEE).await;

    // Create name and address accounts
    let name_account = Keypair::new();
    let address_account = Keypair::new();
    add_account(&mut context, &name_account, &program_id, 0, "name").await;
    add_account(&mut context, &address_account, &program_id, 0, "address").await;

    // Register name
    register_name(
        &mut context,
        &program_id,
        &initializer,
        &name_account,
        &address_account,
        &config_account,
        "test-name".to_string(),
    ).await;

    // Create pending update account
    let pending_update_account = Keypair::new();
    add_account(&mut context, &pending_update_account, &program_id, 0, "pending_update").await;

    // Resolve address
    let resolve_ix = NameRegistryInstruction::ResolveAddress;
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            resolve_ix,
            &program_id,
            &[
                (&name_account, false),  // [writable] name account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify return data
    let account = context
        .banks_client
        .get_account(name_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let name_data = NameAccount::unpack(&account.data).unwrap();
    let resolved_address = name_data.address;
    assert_eq!(resolved_address, initializer.pubkey());
}

#[tokio::test]
async fn test_withdraw() {
    let (mut context, initializer, config_account, program_id) = setup_program().await;

    // Initialize program
    initialize_program(&mut context, &program_id, &initializer, &config_account, REGISTRATION_FEE).await;

    // Create name and address accounts
    let name_account = Keypair::new();
    let address_account = Keypair::new();
    add_account(&mut context, &name_account, &program_id, 0, "name").await;
    add_account(&mut context, &address_account, &program_id, 0, "address").await;

    // Register name to accumulate fees
    register_name(
        &mut context,
        &program_id,
        &initializer,
        &name_account,
        &address_account,
        &config_account,
        "test-name".to_string(),
    ).await;

    // Create pending update account
    let pending_update_account = Keypair::new();
    add_account(&mut context, &pending_update_account, &program_id, 0, "pending_update").await;

    // Get initial balance
    let initial_account = context
        .banks_client
        .get_account(initializer.pubkey())
        .await
        .unwrap()
        .unwrap();
    let initial_balance = initial_account.lamports;

    // Withdraw
    let withdraw_ix = NameRegistryInstruction::Withdraw;
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            withdraw_ix,
            &program_id,
            &[
                (&initializer, true),  // [signer] program owner
                (&config_account, false),  // [writable] config account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Verify balance increased
    let final_account = context
        .banks_client
        .get_account(initializer.pubkey())
        .await
        .unwrap()
        .unwrap();
    let final_balance = final_account.lamports;
    assert!(final_balance > initial_balance);

    // Verify config account is empty
    let config_account = context
        .banks_client
        .get_account(config_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(config_account.lamports, 0);
}

#[tokio::test]
async fn test_error_cases() {
    let (mut context, initializer, config_account, program_id) = setup_program().await;

    // Test registering with insufficient fee
    let name_account = Keypair::new();
    let address_account = Keypair::new();
    add_account(&mut context, &name_account, &program_id, 0, "name").await;
    add_account(&mut context, &address_account, &program_id, 0, "address").await;

    // Create pending update account
    let pending_update_account = Keypair::new();
    add_account(&mut context, &pending_update_account, &program_id, 0, "pending_update").await;

    // Initialize with higher fee
    let init_ix = NameRegistryInstruction::Initialize {
        registration_fee: HIGH_FEE,
    };
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            init_ix,
            &program_id,
            &[
                (&initializer, true),  // [signer] initializer
                (&config_account, false),  // [writable] config account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    context.banks_client.process_transaction(transaction).await.unwrap();

    // Try to register with insufficient balance (0 lamports)
    let register_ix = NameRegistryInstruction::RegisterName {
        name: "test-name".to_string(),
    };
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            register_ix,
            &program_id,
            &[
                (&initializer, true),  // [signer] registrant
                (&name_account, false),  // [writable] name account
                (&address_account, false),  // [writable] address account
                (&config_account, false),  // [writable] config account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    let result = context.banks_client.process_transaction(transaction).await;
    assert!(result.is_err());

    // Test invalid name format
    let register_ix = NameRegistryInstruction::RegisterName {
        name: "INVALID-NAME".to_string(), // Uppercase not allowed
    };
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            register_ix,
            &program_id,
            &[
                (&initializer, true),  // [signer] registrant
                (&name_account, false),  // [writable] name account
                (&address_account, false),  // [writable] address account
                (&config_account, false),  // [writable] config account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    let result = context.banks_client.process_transaction(transaction).await;
    assert!(result.is_err());

    // Test unauthorized owner change
    let unauthorized = Keypair::new();
    add_account(&mut context, &unauthorized, &program_id, 10_000_000_000, "name").await;

    let change_owner_ix = NameRegistryInstruction::ChangeProgramOwner {
        new_owner: unauthorized.pubkey(),
    };
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            change_owner_ix,
            &program_id,
            &[
                (&unauthorized, true),  // [signer] unauthorized user
                (&config_account, false),  // [writable] config account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&unauthorized.pubkey()),
    );
    transaction.sign(&[&unauthorized], context.last_blockhash);
    let result = context.banks_client.process_transaction(transaction).await;
    assert!(result.is_err());

    // Test resolving non-existent name
    let non_existent_name = Keypair::new();
    add_account(&mut context, &non_existent_name, &program_id, 0, "name").await;

    let resolve_ix = NameRegistryInstruction::ResolveAddress;
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            resolve_ix,
            &program_id,
            &[
                (&non_existent_name, false),  // [writable] non-existent name account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    let result = context.banks_client.process_transaction(transaction).await;
    assert!(result.is_err());

    // Test withdrawing with empty balance
    let withdraw_ix = NameRegistryInstruction::Withdraw;
    let mut transaction = Transaction::new_with_payer(
        &[convert_instruction(
            withdraw_ix,
            &program_id,
            &[
                (&initializer, true),  // [signer] program owner
                (&config_account, false),  // [writable] config account
                (&pending_update_account, false),  // [writable] pending update account
            ],
            &solana_program::system_program::id(),
        )],
        Some(&initializer.pubkey()),
    );
    transaction.sign(&[&initializer], context.last_blockhash);
    let result = context.banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
} 