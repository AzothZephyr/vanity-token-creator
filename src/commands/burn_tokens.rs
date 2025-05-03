use serde_json;
use solana_client::rpc_client::RpcClient;
use solana_program::{
    program_pack::Pack, pubkey::Pubkey,
};
use solana_sdk::{
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::{
    instruction::burn,
    state::Mint,
};
use std::str::FromStr;

#[derive(clap::Parser, Debug)]
pub struct BurnTokensArgs {
    /// rpc url
    #[arg(short, long, default_value = "https://api.devnet.solana.com")]
    pub rpc_url: String,

    /// mint address
    #[arg(short, long)]
    pub mint: String,

    /// keypair file path (owner of the tokens)
    #[arg(short, long)]
    pub keypair: String,

    /// amount to burn
    #[arg(short, long)]
    pub amount: u64,
}

pub fn burn_tokens(args: BurnTokensArgs) {
    // create rpc client
    let client = RpcClient::new(args.rpc_url);

    // load keypair from file
    let keypair_file = std::fs::read_to_string(&args.keypair).expect("failed to read keypair file");
    let private_key_bytes: Vec<u8> = serde_json::from_str(&keypair_file).expect("failed to parse keypair bytes");
    let owner = Keypair::from_bytes(&private_key_bytes).expect("failed to create keypair from bytes");

    // parse mint address
    let mint_address = Pubkey::from_str(&args.mint).unwrap();

    // get mint info to determine decimals
    let mint_account = client.get_account(&mint_address).expect("failed to get mint account");
    let mint_data = Mint::unpack(&mint_account.data).expect("failed to unpack mint data");
    let decimals = mint_data.decimals;
    let amount = args.amount * 10u64.pow(decimals as u32);

    // get associated token account
    let associated_token_account = get_associated_token_address(
        &owner.pubkey(),
        &mint_address,
    );

    // verify ata exists
    if client.get_account(&associated_token_account).is_err() {
        println!("error: associated token account not found");
        println!("ata address: {}", associated_token_account);
        return;
    }

    // ask for confirmation before burning
    println!("\nare you sure you'd like to burn {} {} tokens? (y/yes to confirm)", args.amount, args.mint);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("failed to read input");
    
    if !["y", "yes"].contains(&input.trim().to_lowercase().as_str()) {
        println!("burning cancelled");
        return;
    }

    // burn tokens
    let burn_ix = burn(
        &spl_token::id(),
        &associated_token_account,
        &mint_address,
        &owner.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[burn_ix],
        Some(&owner.pubkey()),
    );

    let blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&owner], blockhash);

    let signature = client.send_and_confirm_transaction(&transaction).unwrap();

    println!("tokens burned successfully!");
    println!("mint address: {}", mint_address);
    println!("token owner: {}", owner.pubkey());
    println!("associated token account: {}", associated_token_account);
    println!("amount burned: {}", args.amount);
    println!("transaction signature: {}", signature);
} 