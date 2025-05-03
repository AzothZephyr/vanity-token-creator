use clap::{Parser, Subcommand};
use commands::{create_mint_with_seed, mint_tokens, burn_tokens};

mod commands;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// create a new token mint using a seed
    CreateMint(create_mint_with_seed::CreateMintWithSeedArgs),
    /// mint additional tokens to a wallet
    MintTokens(mint_tokens::MintTokensArgs),
    /// burn tokens from a wallet
    BurnTokens(burn_tokens::BurnTokensArgs),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::CreateMint(args) => create_mint_with_seed::create_mint_with_seed(args),
        Commands::MintTokens(args) => mint_tokens::mint_tokens(args),
        Commands::BurnTokens(args) => burn_tokens::burn_tokens(args),
    }
}
