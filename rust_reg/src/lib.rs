mod programs;

#[cfg(test)]
mod tests {
    use crate::programs::Turbin3_prereq::{CompleteArgs, Turbin3PrereqProgram, UpdateArgs};
    use bs58;
    use solana_client::rpc_client::RpcClient;
    use solana_program::pubkey::Pubkey;
    use solana_program::system_instruction::transfer;
    use solana_program::system_program;
    use solana_sdk::{
        message::Message,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::Transaction,
    };
    use std::io::{self, BufRead};
    use std::str::FromStr;
    const RPC_URL: &str = "https://api.devnet.solana.com";

    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as base58:");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your wallet file is:");
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("{:?}", wallet);
    }

    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a wallet file byte array (e.g., [1, 2, 3, ...]):");

        // Read the input from stdin
        let stdin = io::stdin();
        let input = stdin.lock().lines().next().unwrap().unwrap();

        // Remove square brackets, split by commas, and parse into a Vec<u8>
        let wallet = input
            .trim_start_matches('[') // Remove the starting '['
            .trim_end_matches(']') // Remove the ending ']'
            .split(',') // Split by commas
            .map(|s| s.trim().parse::<u8>()) // Parse each value into u8
            .collect::<Result<Vec<u8>, _>>(); // Collect into a Vec<u8>

        match wallet {
            Ok(wallet) => {
                // Convert the byte array to a Base58 encoded string
                let base58 = bs58::encode(wallet).into_string();
                println!("Your private key in Base58 is: {:?}", base58);
            }
            Err(e) => {
                // Handle any errors that occur during parsing
                println!("Error parsing input: {}", e);
            }
        }
    }

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
        let byte_string = format!("{:?}", kp.to_bytes()); // Convert byte array to a string with spaces
        let no_space_string = byte_string.replace(" ", ""); // Remove spaces from the string
        println!("{}", no_space_string); // Print the string without spaces
    }

    #[test]
    fn airdrop() {
        // Correct function name

        let keypair = read_keypair_file("5fnGpjAUSYDdgSFqhzTZjJwmBQDHJGBgZxEwQdnNjT16.json")
            .expect("Couldn't find wallet file");
        let client = RpcClient::new(RPC_URL);
        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(signature) => println!(
                "Success! Check your TX at https://explorer.solana.com/tx/{}?cluster=devnet",
                signature
            ),
            Err(e) => println!("Error requesting airdrop: {}", e),
        }
    }

    #[test]
    fn transfer_sol() {
        let keypair = read_keypair_file("5fnGpjAUSYDdgSFqhzTZjJwmBQDHJGBgZxEwQdnNjT16.json")
            .expect("Couldn't find wallet file");
        let to_pubkey = Pubkey::from_str("6xoDRsfc8gc4cBCbgMpEj2RtTcoUNLGM9uVDgXG7YFwo").unwrap();
        let rpc_client: RpcClient = RpcClient::new(RPC_URL);

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, 1_000_000)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        // Send the transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        println!(
            "Success! Check your TX at https://explorer.solana.com/tx/{}?cluster=devnet",
            signature
        );
    }

    #[test]
    fn empty_dev_wallet() {
        let rpc_client: RpcClient = RpcClient::new(RPC_URL);

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let keypair = read_keypair_file("5fnGpjAUSYDdgSFqhzTZjJwmBQDHJGBgZxEwQdnNjT16.json")
            .expect("Couldn't find wallet file");

        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");
        println!("Balance: {}", balance);

        let to_pubkey = Pubkey::from_str("6xoDRsfc8gc4cBCbgMpEj2RtTcoUNLGM9uVDgXG7YFwo").unwrap();

        // Create a test transaction to calculate fees
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );
        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");
        println!("Fee: {}", fee);

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
            "Success! Check your TX at https://explorer.solana.com/tx/{}?cluster=devnet",
            signature
        );
    }

    #[test]
    fn prereq() {
        let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");
        let prereq = Turbin3PrereqProgram::derive_program_address(&[
            b"prereq",
            signer.pubkey().to_bytes().as_ref(),
        ]);
        // Define our instruction data
        let args = CompleteArgs {
            github: b"jameslin101".to_vec(),
        };
        let rpc_client: RpcClient = RpcClient::new(RPC_URL);

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let transaction = Turbin3PrereqProgram::complete(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],
            recent_blockhash,
        );
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        println!(
            "Success! Check your TX at https://explorer.solana.com/tx/{}?cluster=devnet",
            signature
        );
    }
}
