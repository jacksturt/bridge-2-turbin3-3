use anchor_lang::prelude::*;

declare_id!("FWroafTkQ2MVVEgaj3L3mvTkiyPuwJy3eYka4Pz6YWQG");

pub mod context;
pub mod state;

use context::*;
use state::*;

#[program]
pub mod wba_escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, amount_x: u64, amount_y: u64) -> Result<()> {
        ctx.accounts.make(seed, amount_x, amount_y, &ctx.bumps)?;
        ctx.accounts.transfer()?;
        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.close()?;
        ctx.accounts.close_vault()?;
        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.transfer_to_maker()?;
        ctx.accounts.transfer_to_taker()?;
        ctx.accounts.close_vault()?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
