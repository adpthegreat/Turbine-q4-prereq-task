pub mod programs;
use solana_program::system_instruction::transfer;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};
use std::io::{self, BufRead};
use std::str::FromStr;
use anchor_lang::{AnchorSerialize, AnchorDeserialize};
const RPC_URL: &str = "https://api.devnet.solana.com";

pub fn get_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()[..8],
    );

    sighash
}

#[derive(AnchorSerialize, Debug)]
pub struct CompleteArgs {
    github: Vec<u8>,
}

#[derive(AnchorSerialize, Debug)]
pub struct UpdateArgs {
    github: [u8],
}

#[cfg(test)]
mod tests {

    use super::*;
    // use crate::programs::Turbin3_prereq::{CompleteArgs, Turbin3PrereqProgram, UpdateArgs};
    use solana_client::{nonblocking::rpc_client, rpc_client::RpcClient};
    use solana_sdk::{account::Account, instruction::{AccountMeta, Instruction}, message::Message, signer::keypair, system_program};

    #[test]
    fn keygen() {
        // Create a new keypair
        let kp = Keypair::new();
        println!(
            "You've generated a new Solana wallet: {}",
            kp.pubkey().to_string()
        );
        println!("");
        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }
    #[test]
    fn airdop() {
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        let client = RpcClient::new(RPC_URL);
        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(s) => {
                println!("Success! Check out your TX here:");
                println!(
                    "https://explorer.solana.com/tx/{}?cluster=devnet",
                    s.to_string()
                );
            }
            Err(e) => println!("Oops, something went wrong: {}", e.to_string()),
        };
    }
    #[test]
    fn transfer_sol() {
        // Import our keypair
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        // Define our Turbin3 public key
        let to_pubkey = Pubkey::from_str("BWZyXw2oagsEV9nYocUWsxDQzBomQvQkTm1SaQFkvJik").unwrap();
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);

        // Get recent blockhash
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, 10_000_000)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        // Send the transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature.to_string()
        )
    }
    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a wallet file byte array:");
        let stdin = io::stdin();
        let wallet = stdin
            .lock()
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim_start_matches("[")
            .trim_end_matches("]")
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();

        println!("Your private key is:");
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }
    #[test]
    fn empty_wallet() {
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        // Define our Turbin3 public key
        let to_pubkey = Pubkey::from_str("BWZyXw2oagsEV9nYocUWsxDQzBomQvQkTm1SaQFkvJik").unwrap();

        let rpc_client = RpcClient::new(RPC_URL);
        // Get balance of dev wallet
        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        // Create a test transaction to calculate fees
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );

        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");

        // Calculate exact fee rate to transfer entire SOL amount out of account minus fees let fee = rpc_client
        rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");

        // Deduct fee from lamports amount and create a TX with correct balance
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature.to_string()
        )
    }

    //program id was changed but we know the instruction name and the data we are passing into it , so we use this method
    #[test]
    fn enroll() -> Result<(), Box<dyn std::error::Error>> {
        let program_id = Pubkey::from_str("ADcaide4vBtKuyZQqdU689YqEGZMCmS4tL35bdTv9wJa")?;

        let rpc_client = RpcClient::new(RPC_URL);

        let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");
        // let prereq = Turbin3PrereqProgram::derive_program_address(&[
        //     b"prereq",
        //     signer.pubkey().to_bytes().as_ref(),
        // ]);
        
        let (prereq, bump) = Pubkey::find_program_address(&[
            b"prereq",
            signer.pubkey().to_bytes().as_ref(),
        ], &program_id);

        let mut accounts :Vec<AccountMeta> = Vec::with_capacity(3);

        accounts.push(AccountMeta::new(signer.pubkey(), false));
        accounts.push(AccountMeta::new(prereq, false));
        accounts.push(AccountMeta::new(system_program::id(), false));

        let args = CompleteArgs {
            github: b"adpthegreat".to_vec(),
        };

        let ix_discriminator = get_discriminator("global", "complete");
            let mut ix_data = Vec::with_capacity(256);
            ix_data.extend_from_slice(&ix_discriminator);
            args.serialize(&mut ix_data)?;

        let enroll_ix = Instruction {
            program_id,
            accounts,
            data: ix_data
        };
        // Get recent blockhash
        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let transaction = Transaction::new_signed_with_payer(
            &enroll_ix, 
             Some(&signer.pubkey()),
            &[&signer],
            blockhash,
        );
        // let transaction = Turbin3PrereqProgram::complete(
        //     &[&signer.pubkey(), &prereq, &system_program::id()],
        //     &args,
        //     Some(&signer.pubkey()),
        //     &[&signer],
        //     blockhash,
        // );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        println!(
            "Success! Check out your TX here:
https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
        
        Ok(())
    }
}
