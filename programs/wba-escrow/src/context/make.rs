use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    maker: Signer<'info>,
    #[account(
        mut,
        associated_token::authority = maker,
        associated_token::mint = mint_x
    )]
    maker_ata_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        associated_token::authority = maker,
        associated_token::mint = mint_y,
        payer = maker
    )]
    maker_ata_y:  InterfaceAccount<'info, TokenAccount>,
    mint_x: InterfaceAccount<'info, Mint>,
    mint_y: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = maker,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space = Escrow::INIT_SPACE
    )]
    escrow: Account<'info, Escrow>,
    #[account(
        init, 
        associated_token::authority = escrow,
        associated_token::mint = mint_x,
        payer = maker,
    )]
    vault: InterfaceAccount<'info, TokenAccount>,
    system_program: Program<'info, System>,
    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Make<'info> {

    pub fn make(&mut self, seed: u64, amount_x: u64, amount_y: u64, bumps: &MakeBumps) -> Result<()> {
        self.escrow.set_inner(Escrow {
            maker: self.maker.key(),
            vault: self.vault.key(),
            mint_x: self.mint_x.key(),
            mint_y: self.mint_y.key(),
            amount_x,
            amount_y,
            seed,
            escrow_bump: bumps.escrow,
        });

        Ok(())
    }

    pub fn transfer(&mut self) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.maker_ata_x.to_account_info(), 
            to: self.vault.to_account_info(), 
            mint: self.mint_x.to_account_info(), 
            authority: self.maker.to_account_info(), 
        };
        let cpi_program = self.token_program.to_account_info();

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_context, self.escrow.amount_x, self.mint_x.decimals)
    }

}