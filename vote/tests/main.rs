use litesvm::{
    LiteSVM,
    types::{FailedTransactionMetadata, TransactionMetadata},
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
};
use vote::{ID, VoteAccountData};

struct T {
    svm: LiteSVM,
    keypair: Keypair,
}

impl T {
    fn vote_account(&self, name: &str) -> Pubkey {
        Pubkey::find_program_address(&[name.as_bytes(), &ID], &ID.into()).0
    }
    fn vote(&mut self, name: &str) -> Result<TransactionMetadata, FailedTransactionMetadata> {
        self.svm.expire_blockhash();
        let byte_name = name.as_bytes();
        let vote_account = self.vote_account(name);

        let data = &mut [0u8; 32];
        data[0] = 1;
        data[1] = byte_name.len() as u8;
        data[2..2 + byte_name.len()].copy_from_slice(byte_name);
        let ix = Instruction::new_with_bytes(
            ID.into(),
            data,
            vec![
                AccountMeta::new(self.keypair.pubkey(), true),
                AccountMeta::new(vote_account, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
        let tx = Transaction::new(
            &[&self.keypair],
            Message::new(&[ix], Some(&self.keypair.pubkey())),
            self.svm.latest_blockhash(),
        );
        self.svm.send_transaction(tx)
    }
}

fn setup() -> T {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(ID.into(), "../target/deploy/vote.so")
        .unwrap();

    let keypair = Keypair::new();
    let _ = svm.airdrop(&keypair.pubkey(), LAMPORTS_PER_SOL * 100);

    T { svm, keypair }
}

#[test]
fn basic_fail() {
    let mut t = setup();
    let ix = Instruction::new_with_bytes(ID.into(), &[], vec![]);
    let tx = Transaction::new(
        &[&t.keypair],
        Message::new(&[ix], Some(&t.keypair.pubkey())),
        t.svm.latest_blockhash(),
    );

    let result = t.svm.send_transaction(tx);
    assert!(result.is_err());
}

#[test]
fn vote_once() {
    let mut t = setup();
    let name = "onkel.sol";
    let vote_account = t.vote_account(name);
    let result = t.vote(name);
    assert!(result.is_ok());

    let account_data = t.svm.get_account(&vote_account).unwrap();
    let vote_account_data = VoteAccountData::try_from(account_data.data.as_slice()).unwrap();
    assert_eq!(vote_account_data.name, name);
    assert_eq!(vote_account_data.votes, 1);
}

#[test]
fn vote_twice() {
    let mut t = setup();
    let name = "onkel.sol";
    let vote_account = t.vote_account(name);

    let result = t.vote(name);
    assert!(result.is_ok());

    let account_data = t.svm.get_account(&vote_account).unwrap();
    let vote_account_data = VoteAccountData::try_from(account_data.data.as_slice()).unwrap();
    assert_eq!(vote_account_data.name, name);
    assert_eq!(vote_account_data.votes, 1);

    let result = t.vote(name);
    assert!(result.is_ok());

    let account_data = t.svm.get_account(&vote_account).unwrap();
    let vote_account_data = VoteAccountData::try_from(account_data.data.as_slice()).unwrap();
    assert_eq!(vote_account_data.name, name);
    assert_eq!(vote_account_data.votes, 2);
}
