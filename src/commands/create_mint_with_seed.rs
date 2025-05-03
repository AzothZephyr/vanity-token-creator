use solana_client::rpc_client::RpcClient;
use solana_program::{program_pack::Pack, pubkey::Pubkey, pubkey::MAX_SEED_LEN, system_instruction};
use solana_sdk::{
    signer::{keypair::read_keypair_file, Signer},
    transaction::Transaction,
};

use spl_token::{instruction::initialize_mint, state::Mint};
use std::str::FromStr;

#[derive(clap::Parser, Debug)]
pub struct CreateMintWithSeedArgs {
    /// rpc url
    #[arg(short, long, default_value = "https://api.devnet.solana.com")]
    pub rpc_url: String,

    /// payer keypair file path (pays for transaction)
    #[arg(short = 'k', long)] // changed short arg to 'k' for consistency
    pub payer: String,

    /// seed for vanity address (required)
    #[arg(short, long)] // made seed required and added short arg 's'
    pub seed: String,

    /// token decimals
    #[arg(short, long, default_value = "9")]
    pub decimals: u8,

    /// freeze authority (optional)
    #[arg(short, long)]
    pub freeze_authority: Option<String>,
}

pub fn create_mint_with_seed(args: CreateMintWithSeedArgs) {
    // create rpc client
    let client = RpcClient::new(args.rpc_url);

    // insure seed is not greater than MAX_SEED_LEN
    if args.seed.len() > MAX_SEED_LEN {
        eprintln!("seed '{}' cannot be creater than {} characters in length", args.seed, MAX_SEED_LEN);
        std::process::exit(1);
    }

    // load payer keypair
    let payer = match read_keypair_file(&args.payer) {
        Ok(kp) => kp,
        Err(e) => {
            eprintln!("error loading payer keypair file '{}': {}", args.payer, e);
            std::process::exit(1);
        }
    };

    // calculate mint account rent
    let mint_rent = client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .expect("failed to get rent exemption");

    // derive mint address using seed
    let mint_pubkey = Pubkey::create_with_seed(
        &payer.pubkey(), // use payer as base
        &args.seed,
        &spl_token::id(),
    )
    .expect("failed to create pubkey with seed");

    // create account with seed instruction
    let create_mint_account_ix = system_instruction::create_account_with_seed(
        &payer.pubkey(),
        &mint_pubkey,
        &payer.pubkey(),
        &args.seed,
        mint_rent,
        Mint::LEN as u64,
        &spl_token::id(),
    );

    // initialize mint instruction
    let freeze_authority_pubkey = args
        .freeze_authority
        .as_ref()
        .map(|s| Pubkey::from_str(s).expect("invalid freeze authority pubkey"));

    let initialize_mint_ix = initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &payer.pubkey(),
        freeze_authority_pubkey.as_ref(),
        args.decimals,
    )
    .expect("failed to create initialize mint instruction");

    // create and send transaction
    let mut transaction = Transaction::new_with_payer(
        &[create_mint_account_ix, initialize_mint_ix],
        Some(&payer.pubkey()),
    );

    let blockhash = client
        .get_latest_blockhash()
        .expect("failed to get blockhash");

    // sign transaction (only payer signs)
    transaction.sign(&[&payer], blockhash);

    // send transaction
    let signature = client
        .send_and_confirm_transaction(&transaction)
        .expect("transaction failed");

    println!("mint created successfully using seed!");
    println!("mint address: {}", mint_pubkey);
    println!("mint authority: {}", payer.pubkey());
    if let Some(fa) = freeze_authority_pubkey {
        println!("freeze authority: {}", fa);
    }
    println!("transaction signature: {}", signature);
    // removed associated token account print
}
