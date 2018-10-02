// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! EIP712 structs
//!
use serde_json::{Value};
use serde::de;
use std::fmt;
use std::collections::HashMap;
use ethereum_types::{U256, H256, Address};
use regex::Regex;

pub(crate) type MessageTypes = HashMap<String, Vec<FieldType>>;

lazy_static! {
	// match solidity identifier with the addition of '[' & ']'
	static ref TYPE_REGEX: Regex = Regex::new(r"^[a-zA-Z_$][a-zA-Z_$0-9\[\]]*$").unwrap();
}

#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct EIP712Domain {
	pub(crate) name: String,
	pub(crate) version: String,
	pub(crate) chain_id: U256,
	pub(crate) verifying_contract: Address,
	#[serde(skip_serializing_if="Option::is_none")]
	pub(crate) salt: Option<H256>,
}
/// EIP-712 struct
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[derive(Deserialize, Debug, Clone)]
pub struct EIP712 {
	pub(crate) types: MessageTypes,
	pub(crate) primary_type: String,
	pub(crate) message: Value,
	pub(crate) domain: EIP712Domain,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct FieldType {
	#[serde(deserialize_with = "deserialize_ident")]
	pub name: String,
	#[serde(rename = "type")]
	#[serde(deserialize_with = "deserialize_ident")]
	pub type_: String,
}


fn deserialize_ident<'de, D>(deserializer: D) -> Result<String, D::Error>
	where
		D: de::Deserializer<'de>,
{
	struct FieldNameVisitor;

	impl<'de> de::Visitor<'de> for FieldNameVisitor {
		type Value = String;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("A valid solidity identifier")
		}

		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
		{
			if !TYPE_REGEX.is_match(v) {
				return Err(E::custom(format!("Invalid Identifier '{}'", v)))
			}

			Ok(v.to_owned())
		}
	}

	deserializer.deserialize_any(FieldNameVisitor)
}


#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::from_str;

	#[test]
	fn test_ident_regex() {
		let test_cases = vec!["unint bytes32", "Seun\\[]", ""];
		for case in test_cases {
			assert_eq!(TYPE_REGEX.is_match(case), false)
		}

		let test_cases = vec!["bytes32", "Foo[]", "bytes1", "bytes32[][]"];
		for case in test_cases {
			assert_eq!(TYPE_REGEX.is_match(case), true)
		}
	}

	#[test]
	fn test_deserialization() {
		let string = r#"{
            "primaryType": "Mail",
			"domain": {
				"name": "Ether Mail",
				"version": "1",
				"chainId": "0x1",
				"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
			},
			"message": {
				"from": {
					"name": "Cow",
					"wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
				},
				"to": {
					"name": "Bob",
					"wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
				},
				"contents": "Hello, Bob!"
			},
			"types": {
				"EIP712Domain": [
				    { "name": "name", "type": "string" },
					{ "name": "version", "type": "string" },
					{ "name": "chainId", "type": "uint256" },
					{ "name": "verifyingContract", "type": "address" }
				],
				"Person": [
					{ "name": "name", "type": "string" },
					{ "name": "wallet", "type": "address" }
				],
				"Mail": [
					{ "name": "from", "type": "Person" },
					{ "name": "to", "type": "Person" },
					{ "name": "contents", "type": "string" }
				]
			}
        }"#;
		let _ = from_str::<EIP712>(string).unwrap();
	}
}
