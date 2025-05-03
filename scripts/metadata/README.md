# token metadata tool

a cli tool for creating and updating metadata for spl tokens.

## prerequisites

- node.js
- yarn
- solana keypair file

## setup

```bash
yarn install
```

## usage

### create metadata

```bash
yarn ts-node metadata.ts create \
  -m <mint-address> \
  -k <keypair-path> \
  -n <token-name> \
  -s <token-symbol> \
  -u <metadata-uri> \
  -r <rpc-url>
```

### update metadata

```bash
yarn ts-node metadata.ts update \
  -m <mint-address> \
  -k <keypair-path> \
  [-n <new-name>] \
  [-s <new-symbol>] \
  [-u <new-uri>] \
  -r <rpc-url>
```

## notes

- keypair file must be in json format, not base58
- rpc url defaults to devnet if not specified
- update authority must match the keypair used 