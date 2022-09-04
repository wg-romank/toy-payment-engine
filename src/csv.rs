use std::{error::Error, io};

use serde::{Deserialize, Serialize};

use crate::{
    account::Account,
    engine::{Engine, Transaction},
    types::{AccountId, Currency, TransactionId},
};

#[derive(Debug, Deserialize)]
pub struct TransactionRaw {
    #[serde(rename = "type")]
    typ: String,
    client: AccountId,
    tx: TransactionId,
    amount: Option<Currency>,
}

impl TransactionRaw {
    pub fn to_transaction(self) -> Result<(AccountId, Transaction), String> {
        use Transaction::*;

        match (self.typ.as_str(), self.amount) {
            ("withdrawal", Some(amount)) => Ok((
                self.client,
                Withdrawal {
                    id: self.tx,
                    amount,
                },
            )),
            ("deposit", Some(amount)) => Ok((
                self.client,
                Deposit {
                    id: self.tx,
                    amount,
                },
            )),
            ("dispute", None) => Ok((self.client, Dispute { id: self.tx })),
            ("resolve", None) => Ok((self.client, Resolve { id: self.tx })),
            ("chargeback", None) => Ok((self.client, Chargeback { id: self.tx })),
            _ => Err(format!("transaction is not valid {:?}", self)),
        }
    }
}

pub fn process_csv(engine: &mut Engine, path: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path)?;

    // todo: log errors
    rdr.deserialize()
        .flat_map(|raw: Result<TransactionRaw, _>| match raw {
            Ok(traw) => Some(traw),
            Err(e) => None,
        })
        .for_each(|traw| match traw.to_transaction() {
            Ok((aid, t)) => engine.process(aid, t),
            Err(_) => todo!(),
        });

    Ok(())
}

#[derive(Debug, Serialize)]
struct AccountStorage {
    client: AccountId,
    available: Currency,
    held: Currency,
    total: Currency,
    locked: bool,
}

impl AccountStorage {
    fn from(account: (&AccountId, &Account)) -> Self {
        let (id, a) = account;
        Self {
            client: *id,
            available: a.availible(),
            held: a.held,
            total: a.total,
            locked: a.locked,
        }
    }
}

pub fn dump_state(engine: &Engine) -> Result<(), Box<dyn Error>> {
    let mut wrtr = csv::Writer::from_writer(io::stdout());

    engine
        .accounts()
        .map(AccountStorage::from)
        .map(|a| wrtr.serialize(a))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}
