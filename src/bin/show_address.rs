use ethers::signers::{LocalWallet, Signer};

fn main() {
    // Load .env file if present
    dotenv::dotenv().ok();

    let private_key = std::env::var("PRIVATE_KEY")
        .expect("PRIVATE_KEY environment variable not set");

    let wallet: LocalWallet = private_key.parse()
        .expect("Failed to parse private key");

    println!("Your wallet address: {:?}", wallet.address());
}
