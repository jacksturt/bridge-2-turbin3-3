use anchor_lang::prelude::*;

#[account]
pub struct Escrow {
    pub maker: Pubkey,
    pub vault: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub amount_x: u64,
    pub amount_y: u64,
    pub seed: u64,
    pub escrow_bump: u8,
}

impl Space for Escrow {
    const INIT_SPACE: usize = 8 + 4 * 32 + 3 * 8 + 1;
}
