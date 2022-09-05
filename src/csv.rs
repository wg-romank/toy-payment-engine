use std::{error::Error, io};

use serde::{Deserialize, Serialize};

use crate::{
    account::Account,
    engine::*,
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
    pub fn to_transaction(&self) -> Result<(AccountId, Transaction), String> {
        match (self.typ.as_str(), self.amount) {
            ("withdrawal", Some(amount)) => Ok((self.client, withdrawal(self.tx, amount))),
            ("deposit", Some(amount)) => Ok((self.client, deposit(self.tx, amount))),
            ("dispute", None) => Ok((self.client, dispute(self.tx))),
            ("resolve", None) => Ok((self.client, resolve(self.tx))),
            ("chargeback", None) => Ok((self.client, chargeback(self.tx))),
            _ => Err(format!("transaction is not valid {:?}", self)),
        }
    }
}

pub fn process_csv(engine: &mut Engine, path: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path)?;

    // todo: logging should be done via some loggin framework, like tokio's tracing
    rdr.deserialize()
        .flat_map(|raw: Result<TransactionRaw, _>| match raw {
            Ok(traw) => Some(traw),
            Err(_) => {
                // println!("error parsing transaction: \n{}", e);
                None
            }
        })
        .flat_map(|traw| match traw.to_transaction() {
            Ok((aid, t)) => Some((aid, t)),
            Err(_) => {
                // println!("error validating transaction: \n{}", e);
                None
            }
        })
        .for_each(|(aid, t)| {
            #[allow(clippy::single_match)]
            match engine.process(aid, t) {
                Ok(_) => (),
                Err(_) => {
                    // println!("error processing transaction: \n{}", e);
                }
            }
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
            available: a.availible().round_dp(4),
            held: a.held.round_dp(4),
            total: a.total.round_dp(4),
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
