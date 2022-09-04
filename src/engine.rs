use std::collections::HashMap;

use crate::{types::{TransactionId, Currency, AccountId}, account::Account};

#[derive(Debug)]
pub enum Transaction {
    Deposit { id: TransactionId, amount: Currency },
    Withdrawal { id: TransactionId, amount: Currency },
    Dispute { id: TransactionId },
    Resolve { id: TransactionId },
    Chargeback { id: TransactionId },
}

impl Transaction {
}

pub struct Engine {
    accounts: HashMap<AccountId, Account>,
}

impl Engine {
    pub fn empty() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn process(&mut self, aid: AccountId, t: Transaction) {
        self.accounts.entry(aid).or_default().process(t);
    }

    pub fn accounts(&self) -> impl Iterator<Item = (&AccountId, &Account)> {
        self.accounts.iter()
    }
}
