use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Take<'info> {
    #[account(mut)]
    maker: SystemAccount<'info>,
    #[account(mut)]
    taker: Signer<'info>,
    #[account(
        mut,
        associated_token::authority = maker,
        associated_token::mint = mint_x
    )]
    maker_ata_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::authority = maker,
        associated_token::mint = mint_y,
    )]
    maker_ata_y: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::authority = taker,
        associated_token::mint = mint_x
    )]
    taker_ata_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        associated_token::authority = taker,
        associated_token::mint = mint_y,
        payer = taker
    )]
    taker_ata_y: InterfaceAccount<'info, TokenAccount>,
    mint_x: InterfaceAccount<'info, Mint>,
    mint_y: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.escrow_bump,
        close = maker,
    )]
    escrow: Account<'info, Escrow>,
    #[account(
        associated_token::authority = escrow,
        associated_token::mint = mint_x,
    )]
    vault: InterfaceAccount<'info, TokenAccount>,
    system_program: Program<'info, System>,
    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Take<'info> {
    pub fn transfer_to_maker(&mut self) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.taker_ata_y.to_account_info(),
            to: self.maker_ata_y.to_account_info(),
            mint: self.mint_y.to_account_info(),
            authority: self.taker.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_context, self.escrow.amount_y, self.mint_y.decimals)
    }

    pub fn transfer_to_taker(&mut self) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.taker_ata_x.to_account_info(),
            mint: self.mint_x.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();

        let maker_key = self.maker.to_account_info().key();
        let seed_bytes = self.escrow.seed.to_le_bytes();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            maker_key.as_ref(),
            seed_bytes.as_ref(),
            &[self.escrow.escrow_bump],
        ]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer_checked(cpi_context, self.escrow.amount_x, self.mint_x.decimals)
    }

    pub fn close_vault(&mut self) -> Result<()> {
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();

        let maker_key = self.maker.to_account_info().key();
        let seed_bytes = self.escrow.seed.to_le_bytes();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            maker_key.as_ref(),
            seed_bytes.as_ref(),
            &[self.escrow.escrow_bump],
        ]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        close_account(cpi_context)
    }
}
