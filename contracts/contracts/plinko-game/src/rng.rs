use cosmwasm_std::{Env, MessageInfo};
use sha2::{Digest, Sha256};

/// Generate provably fair random path for the ball
/// Uses block height, timestamp, sender, and nonce for randomness
pub fn generate_ball_path(
    env: &Env,
    info: &MessageInfo,
    nonce: u64,
    rows: u8,
) -> Vec<u8> {
    let mut path = Vec::new();
    
    // Create seed from multiple sources
    let mut hasher = Sha256::new();
    hasher.update(env.block.height.to_be_bytes());
    hasher.update(env.block.time.nanos().to_be_bytes());
    hasher.update(info.sender.as_bytes());
    hasher.update(nonce.to_be_bytes());
    
    let mut seed = hasher.finalize();
    
    // Generate path (0 = left, 1 = right)
    for _ in 0..rows {
        let mut hasher = Sha256::new();
        hasher.update(seed);
        seed = hasher.finalize();
        
        // Use first byte to determine direction
        let direction = if seed[0] % 2 == 0 { 0 } else { 1 };
        path.push(direction);
    }
    
    path
}

/// Calculate final bucket index from path
pub fn calculate_bucket_index(path: &[u8]) -> usize {
    path.iter().filter(|&&x| x == 1).count()
}
