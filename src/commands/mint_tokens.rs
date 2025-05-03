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
    instruction::mint_to,
    state::Mint,
};
use std::str::FromStr;

#[derive(clap::Parser, Debug)]
pub struct MintTokensArgs {
    /// rpc url
    #[arg(short, long, default_value = "https://api.devnet.solana.com")]
    pub rpc_url: String,

    /// mint address
    #[arg(short, long)]
    pub mint: String,

    /// mint authority keypair file path
    #[arg(short, long)]
    pub keypair: String,

    /// amount to mint
    #[arg(short, long)]
    pub amount: u64,
}

pub fn mint_tokens(args: MintTokensArgs) {
    // create rpc client
    let client = RpcClient::new(args.rpc_url);

    // load mint authority keypair from file
    let keypair_file = std::fs::read_to_string(&args.keypair).expect("failed to read keypair file");
    let private_key_bytes: Vec<u8> =
        serde_json::from_str(&keypair_file).expect("failed to parse keypair bytes");
    let mint_authority =
        Keypair::from_bytes(&private_key_bytes).expect("failed to create keypair from bytes");

    // parse mint address
    let mint_address = Pubkey::from_str(&args.mint).unwrap();

    // get mint info to determine decimals
    let mint_account = client.get_account(&mint_address).expect("failed to get mint account");
    let mint_data = Mint::unpack(&mint_account.data).expect("failed to unpack mint data");
    let decimals = mint_data.decimals;
    let amount = args.amount * 10u64.pow(decimals as u32);

    // ask for confirmation before minting
    println!("\nare you sure you'd like to mint {} {} tokens? (y/yes to confirm)", args.amount, args.mint);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("failed to read input");
    
    if !["y", "yes"].contains(&input.trim().to_lowercase().as_str()) {
        println!("minting cancelled");
        return;
    }

    // get associated token account
    let associated_token_account =
        get_associated_token_address(&mint_authority.pubkey(), &mint_address);

    // check if ata exists, create if not
    if client.get_account(&associated_token_account).is_err() {
        println!("creating associated token account...");
        let create_ata_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                &mint_authority.pubkey(),
                &mint_authority.pubkey(),
                &mint_address,
                &spl_token::id(),
            );

        let mut transaction =
            Transaction::new_with_payer(&[create_ata_ix], Some(&mint_authority.pubkey()));

        let blockhash = client.get_latest_blockhash().unwrap();
        transaction.sign(&[&mint_authority], blockhash);

        match client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                println!(
                    "created associated token account: {}",
                    associated_token_account
                );
                println!("ata creation signature: {}", signature);
            }
            Err(e) => {
                println!("error creating associated token account: {}", e);
                return;
            }
        }
    } else {
        println!(
            "associated token account exists: {}",
            associated_token_account
        );
    }

    // mint tokens
    let mint_to_ix = mint_to(
        &spl_token::id(),
        &mint_address,
        &associated_token_account,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    let mut transaction =
        Transaction::new_with_payer(&[mint_to_ix], Some(&mint_authority.pubkey()));

    let blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&mint_authority], blockhash);

    let signature = client.send_and_confirm_transaction(&transaction).unwrap();

    println!("tokens minted successfully!");
    println!("mint address: {}", mint_address);
    println!("mint authority: {}", mint_authority.pubkey());
    println!("associated token account: {}", associated_token_account);
    println!("amount minted: {}", args.amount);
    println!("transaction signature: {}", signature);
} 