# vanity token mint

a cli tool for creating deterministic vanity mints and managing spl tokens on solana.

this tool uses create_account_with_seed which takes an ascii string, and a base address, to produce a token address. this allows you to create tokens from seeds you [bruteforce](https://github.com/cavemanloverboy/vanity), or use a static input seed to produce deterministic mints for testing. 

## setup

```bash
cargo build
```

## usage

the tool provides the following commands:

### create-mint

creates a new token mint using a pda derived from a seed.

```bash
cargo run -- create-mint \
  -r/--rpc-url <rpc-url> \
  -k/--payer <payer-keypair-path> \
  -s/--seed <seed-string> \
  -d/--decimals <decimals> \
  -f/--freeze-authority <pubkey> # optional
```
### mint-tokens

mints additional tokens to a specified wallet.

```bash
cargo run -- mint-tokens \
  -r/--rpc-url <rpc-url> \
  -m/--mint <mint-address> \
  -k/--keypair <keypair-path> \
  -a/--amount <amount>
```

### burn-tokens

burns tokens from a specified wallet.

```bash
cargo run -- burn-tokens \
  -r/--rpc-url <rpc-url> \
  -m/--mint <mint-address> \
  -k/--keypair <keypair-path> \
  -a/--amount <amount>
```

## notes

- rpc url defaults to `https://api.devnet.solana.com` if not specified
- decimals defaults to 9 if not specified
- freeze authority is optional, defaults to none
- the payer keypair (`-k/--payer`) must have enough sol for account creation and transaction fees

## vanity specific notes:

- a base is required and is the pubkey that will be the signer
- a seed is required, must be an ASCII string non greater in length than Pubkey::MAX_SEED_LEN (32 char)