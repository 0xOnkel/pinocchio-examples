#![allow(unexpected_cfgs)]
#![no_std]

use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg, no_allocator, nostd_panic_handler, program_entrypoint,
    program_error::ProgramError,
    pubkey::{Pubkey, find_program_address},
};

program_entrypoint!(process_instruction);
nostd_panic_handler!();
no_allocator!();

// 22222222222222222222222222222222222222222222
// inspired by blueshift.gg
pub const ID: Pubkey = [
    0x0f, 0x1e, 0x6b, 0x14, 0x21, 0xc0, 0x4a, 0x07, 0x04, 0x31, 0x26, 0x5c, 0x19, 0xc5, 0xbb, 0xee,
    0x19, 0x92, 0xba, 0xe8, 0xaf, 0xd1, 0xcd, 0x07, 0x8e, 0xf8, 0xaf, 0x70, 0x47, 0xdc, 0x11, 0xf7,
];

struct VoteAccounts<'a> {
    owner: &'a AccountInfo,
    vote_account: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for VoteAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, vote_account, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(VoteAccounts {
            owner,
            vote_account,
        })
    }
}

struct Vote<'a> {
    accounts: VoteAccounts<'a>,
    instruction_data: VoteInstructionData<'a>,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Vote<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = VoteAccounts::try_from(accounts)?;
        let instruction_data = VoteInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

struct VoteInstructionData<'a> {
    pub name: &'a str,
}

impl<'a> TryFrom<&'a [u8]> for VoteInstructionData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let name_len = data[0] as usize;
        let name_bytes = &data[1..1 + name_len];
        let name =
            core::str::from_utf8(name_bytes).map_err(|_| ProgramError::InvalidInstructionData)?;
        Ok(VoteInstructionData { name })
    }
}

pub struct VoteAccountData<'a> {
    pub name: &'a str,
    pub votes: u64,
}

impl<'a> VoteAccountData<'a> {
    const SIZE: usize = 64;
}

impl<'a> TryFrom<&'a [u8]> for VoteAccountData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let name_len = data[0] as usize;
        let name_bytes = &data[1..1 + name_len];
        let name =
            core::str::from_utf8(name_bytes).map_err(|_| ProgramError::InvalidInstructionData)?;
        let votes = u64::from_le_bytes(
            data[2 + name_len..2 + 8 + name_len]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        Ok(VoteAccountData { name, votes })
    }
}

impl<'a> TryFrom<&'a VoteAccountData<'a>> for [u8; VoteAccountData::SIZE] {
    type Error = ProgramError;

    fn try_from(vote_account_data: &'a VoteAccountData<'a>) -> Result<Self, Self::Error> {
        let name_len = vote_account_data.name.len();
        let votes = vote_account_data.votes.to_le_bytes();
        let mut data = [0u8; VoteAccountData::SIZE];
        data[0] = name_len as u8;
        data[1..name_len + 1].copy_from_slice(vote_account_data.name.as_bytes());
        data[2 + name_len..2 + 8 + name_len].copy_from_slice(&votes);

        Ok(data)
    }
}

impl<'a> Vote<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        let (key, bump) =
            find_program_address(&[self.instruction_data.name.as_bytes(), &ID], &crate::ID);

        if &key != self.accounts.vote_account.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        let bump = [bump];

        let seeds = [
            Seed::from(self.instruction_data.name.as_bytes()),
            Seed::from(&ID),
            Seed::from(&bump),
        ];
        let signers = [Signer::from(&seeds)];

        if self.accounts.vote_account.data_is_empty()
            && self.accounts.vote_account.lamports().eq(&0)
        {
            msg!("init of vote_account");
            pinocchio_system::instructions::CreateAccount {
                from: self.accounts.owner,
                to: self.accounts.vote_account,
                lamports: 500_000_000,
                space: VoteAccountData::SIZE as u64,
                owner: &ID,
            }
            .invoke_signed(&signers)?;

            let vote_account_data = VoteAccountData {
                name: self.instruction_data.name,
                votes: 1,
            };
            let mut data = self.accounts.vote_account.try_borrow_mut_data()?;
            let src: [u8; VoteAccountData::SIZE] = (&vote_account_data).try_into().unwrap();
            data.copy_from_slice(&src);
        } else {
            let mut data = self.accounts.vote_account.try_borrow_mut_data()?;
            let mut vote_account_data = VoteAccountData::try_from(data.as_ref())?;
            vote_account_data.votes += 1;
            let src: [u8; VoteAccountData::SIZE] = (&vote_account_data).try_into().unwrap();
            data.copy_from_slice(&src);
        };

        Ok(())
    }
}

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Vote::DISCRIMINATOR, data)) => Vote::try_from((data, accounts))?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
