use super::{
    super::serde::{
        byte_array::base64::serialize as base64_ser,
        unsigned_field_element::{UfeHex, UfeHexOption},
    },
    AbiEntry, FieldElement, L1Address,
};

use serde::{Deserialize, Serialize, Serializer};
use serde_with::serde_as;
use std::sync::Arc;

#[serde_as]
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct AddTransactionResult {
    pub code: AddTransactionResultCode,
    #[serde_as(as = "UfeHex")]
    pub transaction_hash: FieldElement,
    #[serde(default)]
    #[serde_as(as = "UfeHexOption")]
    pub address: Option<FieldElement>,
    #[serde(default)]
    #[serde_as(as = "UfeHexOption")]
    pub class_hash: Option<FieldElement>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub enum AddTransactionResultCode {
    #[serde(rename = "TRANSACTION_RECEIVED")]
    TransactionReceived,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionRequest {
    Declare(DeclareTransaction),
    InvokeFunction(InvokeFunctionTransaction),
    DeployAccount(DeployAccountTransaction),
}

/// Represents a transaction in the StarkNet network that is originated from an action of an
/// account.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountTransaction {
    Declare(DeclareTransaction),
    InvokeFunction(InvokeFunctionTransaction),
    DeployAccount(DeployAccountTransaction),
}

/// Represents a contract function call in the StarkNet network.
#[serde_as]
#[derive(Debug, Serialize)]
pub struct CallFunction {
    #[serde_as(as = "UfeHex")]
    pub contract_address: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub entry_point_selector: FieldElement,
    pub calldata: Vec<FieldElement>,
}

/// Represents an L1 handler call in the StarkNet network.
#[serde_as]
#[derive(Debug, Serialize)]
pub struct CallL1Handler {
    // The sequencer excepts the address in decimal representation
    #[serde(serialize_with = "l1_addr_as_dec")]
    pub from_address: L1Address,
    #[serde_as(as = "UfeHex")]
    pub to_address: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub entry_point_selector: FieldElement,
    #[serde_as(as = "Vec<UfeHex>")]
    pub payload: Vec<FieldElement>,
}

#[derive(Debug)]
pub struct DeclareTransaction {
    pub contract_class: Arc<ContractDefinition>,
    /// The address of the account contract sending the declaration transaction.
    pub sender_address: FieldElement,
    /// The maximal fee to be paid in Wei for declaring a contract class.
    pub max_fee: FieldElement,
    /// Additional information given by the caller that represents the signature of the transaction.
    pub signature: Vec<FieldElement>,
    /// A sequential integer used to distinguish between transactions and order them.
    pub nonce: FieldElement,
}

#[derive(Debug)]
pub struct InvokeFunctionTransaction {
    pub contract_address: FieldElement,
    pub calldata: Vec<FieldElement>,
    pub signature: Vec<FieldElement>,
    pub max_fee: FieldElement,
    pub nonce: FieldElement,
}

#[derive(Debug)]
pub struct DeployAccountTransaction {
    pub class_hash: FieldElement,
    pub contract_address_salt: FieldElement,
    pub constructor_calldata: Vec<FieldElement>,
    // The maximal fee to be paid in Wei for executing the transaction.
    pub max_fee: FieldElement,
    // The signature of the transaction.
    pub signature: Vec<FieldElement>,
    // The nonce of the transaction.
    pub nonce: FieldElement,
}

#[derive(Debug, Serialize)]
pub struct ContractDefinition {
    #[serde(serialize_with = "base64_ser")]
    pub program: Vec<u8>,
    pub entry_points_by_type: EntryPointsByType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<Vec<AbiEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct EntryPointsByType {
    pub constructor: Vec<EntryPoint>,
    pub external: Vec<EntryPoint>,
    pub l1_handler: Vec<EntryPoint>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct EntryPoint {
    #[serde_as(as = "UfeHex")]
    pub offset: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub selector: FieldElement,
}

impl Serialize for DeclareTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[serde_as]
        #[derive(Serialize)]
        struct Versioned<'a> {
            #[serde_as(as = "UfeHex")]
            version: FieldElement,
            contract_class: &'a ContractDefinition,
            #[serde_as(as = "UfeHex")]
            sender_address: &'a FieldElement,
            #[serde_as(as = "UfeHex")]
            max_fee: &'a FieldElement,
            signature: &'a Vec<FieldElement>,
            #[serde_as(as = "UfeHex")]
            nonce: &'a FieldElement,
        }

        let versioned = Versioned {
            version: FieldElement::ONE,
            contract_class: &self.contract_class,
            sender_address: &self.sender_address,
            max_fee: &self.max_fee,
            signature: &self.signature,
            nonce: &self.nonce,
        };

        Versioned::serialize(&versioned, serializer)
    }
}

impl Serialize for InvokeFunctionTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[serde_as]
        #[derive(Serialize)]
        struct Versioned<'a> {
            #[serde_as(as = "UfeHex")]
            version: FieldElement,
            #[serde_as(as = "UfeHex")]
            contract_address: &'a FieldElement,
            calldata: &'a Vec<FieldElement>,
            signature: &'a Vec<FieldElement>,
            #[serde_as(as = "UfeHex")]
            max_fee: &'a FieldElement,
            #[serde_as(as = "UfeHex")]
            nonce: &'a FieldElement,
        }

        let versioned = Versioned {
            version: FieldElement::ONE,
            contract_address: &self.contract_address,
            calldata: &self.calldata,
            signature: &self.signature,
            max_fee: &self.max_fee,
            nonce: &self.nonce,
        };

        Versioned::serialize(&versioned, serializer)
    }
}

impl Serialize for DeployAccountTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[serde_as]
        #[derive(Serialize)]
        struct Versioned<'a> {
            #[serde_as(as = "UfeHex")]
            version: FieldElement,
            #[serde_as(as = "UfeHex")]
            class_hash: &'a FieldElement,
            #[serde_as(as = "UfeHex")]
            contract_address_salt: &'a FieldElement,
            constructor_calldata: &'a Vec<FieldElement>,
            #[serde_as(as = "UfeHex")]
            max_fee: &'a FieldElement,
            signature: &'a Vec<FieldElement>,
            #[serde_as(as = "UfeHex")]
            nonce: &'a FieldElement,
        }

        let versioned = Versioned {
            version: FieldElement::ONE,
            class_hash: &self.class_hash,
            contract_address_salt: &self.contract_address_salt,
            constructor_calldata: &self.constructor_calldata,
            max_fee: &self.max_fee,
            signature: &self.signature,
            nonce: &self.nonce,
        };

        Versioned::serialize(&versioned, serializer)
    }
}

fn l1_addr_as_dec<S>(value: &L1Address, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut buffer = [0u8; 32];
    buffer[12..].copy_from_slice(&value.0);

    // Unwrapping is safe here as it's never out of range
    let addr_in_felt = FieldElement::from_bytes_be(&buffer).unwrap();

    serializer.serialize_str(&addr_in_felt.to_string())
}
