// RGB standard library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::str::FromStr;
use regex::Regex;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

use lnpbp::bitcoin::{OutPoint, Txid};
use lnpbp::bp::blind::{OutpointHash, OutpointReveal};
use lnpbp::hex::FromHex;
use lnpbp::rgb::SealDefinition;

use super::AccountingValue;
use crate::error::ParseError;

#[derive(Clone, Debug, PartialEq, StrictEncode, StrictDecode)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize,),
    serde(crate = "serde_crate")
)]
pub struct SealCoins {
    pub coins: AccountingValue,
    pub vout: u32,
    pub txid: Option<Txid>,
}

#[derive(Clone, Debug, PartialEq, Display, StrictEncode, StrictDecode)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize,),
    serde(crate = "serde_crate")
)]
#[display("{coins}@{outpoint}")]
pub struct OutpointCoins {
    pub coins: AccountingValue,
    pub outpoint: OutPoint,
}

#[derive(Clone, Debug, PartialEq, Display, StrictEncode, StrictDecode)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize,),
    serde(crate = "serde_crate")
)]
#[display("{coins}@{seal_confidential}")]
pub struct ConsealCoins {
    pub coins: AccountingValue,
    pub seal_confidential: OutpointHash,
}

impl SealCoins {
    pub fn seal_definition(&self) -> SealDefinition {
        use lnpbp::bitcoin::secp256k1::rand::{self, RngCore};
        let mut rng = rand::thread_rng();
        let entropy = rng.next_u64(); // Not an amount blinding factor but outpoint blinding
        match self.txid {
            Some(txid) => SealDefinition::TxOutpoint(OutpointReveal {
                blinding: entropy,
                txid,
                vout: self.vout,
            }),
            None => SealDefinition::WitnessVout {
                vout: self.vout,
                blinding: entropy,
            },
        }
    }
}

impl OutpointCoins {
    pub fn seal_definition(&self) -> SealDefinition {
        use lnpbp::bitcoin::secp256k1::rand::{self, RngCore};
        let mut rng = rand::thread_rng();
        let entropy = rng.next_u64(); // Not an amount blinding factor but outpoint blinding
        SealDefinition::TxOutpoint(OutpointReveal {
            blinding: entropy,
            txid: self.outpoint.txid,
            vout: self.outpoint.vout,
        })
    }
}

impl Display for SealCoins {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}@", self.coins)?;
        if let Some(txid) = self.txid {
            write!(f, "{}:", txid)?;
        }
        f.write_str(&self.vout.to_string())
    }
}

impl FromStr for SealCoins {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(
            r"(?x)
                ^(?P<coins>[\d.,_']+) # float amount
                @
                ((?P<txid>[a-f\d]{64}) # Txid
                :)
                (?P<vout>\d+)$ # Vout
            ",
        )
        .expect("Regex parse failure");
        if let Some(m) = re.captures(&s.to_ascii_lowercase()) {
            match (m.name("coins"), m.name("txid"), m.name("vout")) {
                (Some(amount), Some(txid), Some(vout)) => Ok(Self {
                    coins: amount.as_str().parse()?,
                    vout: vout.as_str().parse()?,
                    txid: Some(Txid::from_hex(txid.as_str())?),
                }),
                (Some(amount), None, Some(vout)) => Ok(Self {
                    coins: amount.as_str().parse()?,
                    vout: vout.as_str().parse()?,
                    txid: None,
                }),
                _ => Err(ParseError),
            }
        } else {
            Err(ParseError)
        }
    }
}

impl FromStr for OutpointCoins {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('@');
        match (iter.next(), iter.next(), iter.next()) {
            (Some(amount), Some(outpoint), None) => Ok(Self {
                coins: amount.parse()?,
                outpoint: outpoint.parse()?,
            }),
            (Some(_), Some(_), _) => Err(ParseError),
            _ => Err(ParseError),
        }
    }
}

impl FromStr for ConsealCoins {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(
            r"(?x)
                ^(?P<coins>[\d.,_']+) # float amount
                @
                ((?P<seal>[a-f\d]{64}))$ # Confidential seal: outpoint hash
            ",
        )
        .expect("Regex parse failure");
        if let Some(m) = re.captures(&s.to_ascii_lowercase()) {
            match (m.name("coins"), m.name("seal")) {
                (Some(amount), Some(seal)) => Ok(Self {
                    coins: amount.as_str().parse()?,
                    seal_confidential: OutpointHash::from_hex(seal.as_str())?,
                }),
                _ => Err(ParseError),
            }
        } else {
            Err(ParseError)
        }
    }
}

impl Eq for OutpointCoins {}

impl PartialOrd for OutpointCoins {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OutpointCoins {
    fn cmp(&self, other: &Self) -> Ordering {
        self.outpoint.cmp(&other.outpoint)
    }
}

impl Hash for OutpointCoins {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.outpoint.hash(state);
    }
}
