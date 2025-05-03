#!/usr/bin/env ts-node
import { Command } from 'commander';
import { 
  publicKey, 
  signerIdentity, 
  keypairIdentity,
  generateSigner,
  Keypair,
  percentAmount
} from '@metaplex-foundation/umi';
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults';
import { 
  fetchMetadataFromSeeds, 
  updateV1, 
  mplTokenMetadata,
  findMetadataPda,
  createV1,
  TokenStandard
} from '@metaplex-foundation/mpl-token-metadata';
import { fromWeb3JsKeypair } from '@metaplex-foundation/umi-web3js-adapters';
import { readFileSync } from 'fs';
import * as web3 from '@solana/web3.js';

// Setup the CLI
const program = new Command();

program
  .name('update-token-metadata')
  .description('CLI to update token metadata for SPL tokens')
  .version('1.0.0');

program
  .command('update')
  .description('Update token metadata')
  .requiredOption('-m, --mint <string>', 'Token mint address')
  .requiredOption('-k, --keypair <path>', 'Path to update authority keypair file')
  .option('-n, --name <string>', 'New token name')
  .option('-s, --symbol <string>', 'New token symbol')
  .option('-u, --uri <string>', 'New URI for the token metadata JSON')
  .option('-r, --rpc <string>', 'RPC URL', 'https://api.devnet.solana.com')
  .action(async (options) => {
    try {
      // Load the update authority keypair
      const keypairData = JSON.parse(readFileSync(options.keypair, 'utf-8'));
      const web3JsKeypair = web3.Keypair.fromSecretKey(
        Uint8Array.from(keypairData)
      );
      
      // Create UMI instance with RPC endpoint
      const umi = createUmi(options.rpc)
        .use(mplTokenMetadata())
        .use(keypairIdentity(fromWeb3JsKeypair(web3JsKeypair)));
      
      // Parse mint address
      const mintAddress = publicKey(options.mint);
      
      console.log(`\nFetching current metadata for token: ${options.mint}`);
      console.log(`Using RPC endpoint: ${options.rpc}`);
      
      try {
        // First check if metadata exists
        const metadataPda = findMetadataPda(umi, { mint: mintAddress });
        console.log(`Metadata address: ${metadataPda[0]}`);
        
        // Fetch current metadata
        const currentMetadata = await fetchMetadataFromSeeds(umi, { mint: mintAddress });
        console.log("\nCurrent metadata:");
        console.log(`Name: ${currentMetadata.name}`);
        console.log(`Symbol: ${currentMetadata.symbol}`);
        console.log(`URI: ${currentMetadata.uri}`);
        
        // Prepare updated metadata
        const updatedMetadata = {
          ...currentMetadata,
          name: options.name || currentMetadata.name,
          symbol: options.symbol || currentMetadata.symbol,
          uri: options.uri || currentMetadata.uri,
        };
        
        console.log("\nUpdating to:");
        console.log(`Name: ${updatedMetadata.name}`);
        console.log(`Symbol: ${updatedMetadata.symbol}`);
        console.log(`URI: ${updatedMetadata.uri}`);
        
        // Update metadata
        const txResult = await updateV1(umi, {
          mint: mintAddress,
          authority: umi.identity,
          data: updatedMetadata,
        }).sendAndConfirm(umi);
        
        console.log(`\nMetadata updated successfully!`);
        console.log(`Transaction ID: ${txResult.signature.toString()}`);
        
      } catch (error: unknown) {
        // If metadata doesn't exist yet, we need to create it
        if (error instanceof Error && error.message.includes("Account not found")) {
          console.log("Metadata doesn't exist yet. Please create metadata first using createV1 method.");
          console.log(`Example: ./create-token-metadata.ts create -m ${options.mint} -k ${options.keypair} -n "Token Name" -s "SYM" -u "https://your-metadata-url.json"`);
          return;
        }
        throw error;
      }
      
    } catch (error: unknown) {
      if (error instanceof Error) {
        console.error(`Error: ${error.message}`);
        console.error(error);
      } else {
        console.error('An unknown error occurred');
      }
      process.exit(1);
    }
  });

// Command to create initial metadata if it doesn't exist
program
  .command('create')
  .description('Create initial token metadata')
  .requiredOption('-m, --mint <string>', 'Token mint address')
  .requiredOption('-k, --keypair <path>', 'Path to update authority keypair file')
  .requiredOption('-n, --name <string>', 'Token name')
  .requiredOption('-s, --symbol <string>', 'Token symbol')
  .requiredOption('-u, --uri <string>', 'URI for the token metadata JSON')
  .option('-r, --rpc <string>', 'RPC URL', 'https://api.devnet.solana.com')
  .action(async (options) => {
    try {
      // Load the authority keypair
      const keypairData = JSON.parse(readFileSync(options.keypair, 'utf-8'));
      const web3JsKeypair = web3.Keypair.fromSecretKey(
        Uint8Array.from(keypairData)
      );
      
      // Create UMI instance with RPC endpoint
      const umi = createUmi(options.rpc)
        .use(mplTokenMetadata())
        .use(keypairIdentity(fromWeb3JsKeypair(web3JsKeypair)));
      
      // Parse mint address
      const mintAddress = publicKey(options.mint);
      
      // Import the createV1 function
      const { createV1 } = await import('@metaplex-foundation/mpl-token-metadata');
      
      console.log(`\nCreating metadata for token: ${options.mint}`);
      console.log(`Using RPC endpoint: ${options.rpc}`);
      console.log(`Name: ${options.name}`);
      console.log(`Symbol: ${options.symbol}`);
      console.log(`URI: ${options.uri}`);
      
      try {
        // First check if metadata already exists
        const metadataPda = findMetadataPda(umi, { mint: mintAddress });
        console.log(`Metadata PDA: ${metadataPda[0]}`);
        
        // Verify token state
        const tokenAccount = await umi.rpc.getAccount(mintAddress);
        if (!tokenAccount.exists) {
          throw new Error('Token account does not exist');
        }
        
        console.log('\nToken account details:');
        console.log(`Owner: ${tokenAccount.owner}`);
        console.log(`Executable: ${tokenAccount.executable}`);
        console.log(`Lamports: ${tokenAccount.lamports}`);
        console.log(`Data length: ${tokenAccount.data.length}`);
        
        // Verify token program
        const tokenProgramId = new web3.PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
        if (tokenAccount.owner.toString() !== tokenProgramId.toString()) {
          throw new Error(`Token is not owned by the standard SPL Token program. Owner: ${tokenAccount.owner}`);
        }

        // Get the update authority from the keypair
        const updateAuthority = umi.identity.publicKey;
        console.log('\nUpdate authority details:');
        console.log(`Update authority: ${updateAuthority}`);
        console.log(`Is signer: ${umi.identity.publicKey.toString() === updateAuthority.toString()}`);
        
        // Create metadata with explicit program ID
        const txResult = await createV1(umi, {
          mint: mintAddress,
          name: options.name,
          symbol: options.symbol,
          uri: options.uri,
          sellerFeeBasisPoints: percentAmount(0), // 0% royalty fee
          isMutable: true, // Allow future updates
          authority: umi.identity,
          updateAuthority: updateAuthority,
          payer: umi.identity,
          tokenStandard: TokenStandard.Fungible // Correct token standard for SPL tokens
        }).sendAndConfirm(umi);
        
        console.log(`\nMetadata created successfully!`);
        console.log(`Transaction ID: ${txResult.signature.toString()}`);
        console.log(`Update authority set to: ${updateAuthority}`);
        
      } catch (error: unknown) {
        if (error instanceof Error) {
          console.error(`Error: ${error.message}`);
          console.error(error);
        } else {
          console.error('An unknown error occurred');
        }
        process.exit(1);
      }
      
    } catch (error: unknown) {
      if (error instanceof Error) {
        console.error(`Error: ${error.message}`);
        console.error(error);
      } else {
        console.error('An unknown error occurred');
      }
      process.exit(1);
    }
  });

program.parse(process.argv);