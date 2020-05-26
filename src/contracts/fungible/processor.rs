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

use chrono::Utc;
use core::convert::TryFrom;
use std::collections::BTreeMap;

use serde::Deserialize;

use lnpbp::bp;
use lnpbp::rgb::prelude::*;

use super::schema::{self, AssignmentsType, FieldType};
use super::{Asset, Coins, Outcoins};

use crate::error::{BootstrapError, ServiceErrorDomain};
use crate::util::SealSpec;
use crate::{field, type_map};

pub struct Processor {}

#[derive(Debug, Deserialize)]
pub enum IssueStructure {
    SingleIssue,
    MultipleIssues {
        max_supply: f32,
        reissue_control: SealSpec,
    },
}

impl Processor {
    pub fn new() -> Result<Self, BootstrapError> {
        debug!("Instantiating RGB asset manager ...");

        let me = Self {};
        /*
        let storage = rgb_storage.clone();
        let me = Self {
            rgb_storage,
            asset_storage,
        };
         */
        let _schema = schema::schema();
        //if !me.rgb_storage.lock()?.has_schema(schema.schema_id())? {
        info!("RGB fungible assets schema file not found, creating one");
        //storage.lock()?.add_schema(&schema)?;
        //}

        Ok(me)
    }

    pub fn issue(
        &mut self,
        network: bp::Network,
        ticker: String,
        name: String,
        description: Option<String>,
        issue_structure: IssueStructure,
        allocations: Vec<Outcoins>,
        precision: u8,
        prune_seals: Vec<SealSpec>,
        dust_limit: Option<Amount>,
    ) -> Result<(Asset, Genesis), ServiceErrorDomain> {
        let now = Utc::now().timestamp();
        let mut metadata = type_map! {
            FieldType::Ticker => field!(String, ticker),
            FieldType::Name => field!(String, name),
            FieldType::FractionalBits => field!(U8, precision),
            FieldType::DustLimit => field!(U64, dust_limit.unwrap_or(0)),
            FieldType::Timestamp => field!(U32, now as u32)
        };
        if let Some(description) = description {
            metadata.insert(-FieldType::Description, field!(String, description));
        }

        let mut issued_supply = 0u64;
        let allocations = allocations
            .into_iter()
            .map(|outcoins| {
                let amount = Coins::transmutate(outcoins.coins, precision);
                issued_supply += amount;
                (outcoins.seal_definition(), amount)
            })
            .collect();
        let mut assignments = BTreeMap::new();
        assignments.insert(
            -AssignmentsType::Assets,
            AssignmentsVariant::zero_balanced(allocations, 0),
        );
        metadata.insert(-FieldType::IssuedSupply, field!(U64, issued_supply));

        let mut total_supply = issued_supply;
        if let IssueStructure::MultipleIssues {
            max_supply,
            reissue_control,
        } = issue_structure
        {
            total_supply = Coins::transmutate(max_supply, precision);
            if total_supply < issued_supply {
                Err(ServiceErrorDomain::Schema(format!(
                    "Total supply ({}) should be greater than the issued supply ({})",
                    total_supply, issued_supply
                )))?;
            }
            metadata.insert(-FieldType::TotalSupply, field!(U64, total_supply));
            assignments.insert(
                -AssignmentsType::Issue,
                AssignmentsVariant::Void(bset![Assignment::Revealed {
                    seal_definition: reissue_control.seal_definition(),
                    assigned_state: data::Void
                }]),
            );
        } else {
            metadata.insert(-FieldType::TotalSupply, field!(U64, total_supply));
        }

        assignments.insert(
            -AssignmentsType::Prune,
            AssignmentsVariant::Void(
                prune_seals
                    .into_iter()
                    .map(|seal_spec| Assignment::Revealed {
                        seal_definition: seal_spec.seal_definition(),
                        assigned_state: data::Void,
                    })
                    .collect(),
            ),
        );

        let genesis = Genesis::with(
            schema::schema().schema_id(),
            network,
            metadata,
            assignments,
            vec![],
        );
        //self.rgb_storage.lock()?.add_genesis(&genesis)?;

        let asset = Asset::try_from(genesis.clone())?;
        //self.asset_storage.lock()?.add_asset(asset.clone())?;

        Ok((asset, genesis))
    }

    /*
    pub fn assets(&self) -> Result<Vec<Asset>, InteroperableError> {
        Ok(self
            .asset_storage
            .lock()?
            .assets()?
            .into_iter()
            .map(Asset::clone)
            .collect())
    }

    pub fn import(&self, data: ExchangableData) -> Result<MagicNumber, InteroperableError> {
        unimplemented!()
    }

    pub fn pay(&self, invoice: Invoice) -> Result<(), InteroperableError> {
        let assets = self.asset_storage.lock()?.assets()?;
    }

     */
}
