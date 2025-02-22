//! Module containing the fault proof test fixture.

use std::collections::BTreeMap;

use alloy_primitives::{Address, BlockHash, BlockNumber, Bytes, ChainId, B256, U256};
use maili_genesis::RollupConfig;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// The fault proof fixture is the top-level object that contains
/// everything needed to run a fault proof test.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FaultProofFixture {
    /// The inputs to the fault proof test.
    pub inputs: FaultProofInputs,
    /// The expected status of the fault proof test.
    pub expected_status: FaultProofStatus,
    /// The witness data for the fault proof test.
    pub witness_data: BTreeMap<B256, Bytes>,
}

/// The fault proof inputs are the inputs to the fault proof test.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FaultProofInputs {
    /// The L1 head block hash.
    pub l1_head: BlockHash,
    /// The L2 head block hash.
    pub l2_head: BlockHash,
    /// The claimed L2 output root to validate.
    pub l2_claim: B256,
    /// The agreed L2 output root to start derivation from.
    pub l2_output_root: B256,
    /// The L2 block number that the claim is from.
    pub l2_block_number: BlockNumber,
    /// The chain definition
    pub chain_definition: ChainDefinition,
}

/// The chain definition, either named or unnamed with a rollup config and genesis.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ChainDefinition {
    /// Named chain definition.
    Named(String),
    /// Unnamed chain definition with a rollup config and genesis.
    Unnamed(RollupConfig, Genesis),
}

impl Default for ChainDefinition {
    fn default() -> Self {
        ChainDefinition::Named("base-mainnet".to_string())
    }
}

/// The genesis block information.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    /// The chain configuration.
    pub config: ChainConfig,
    /// The nonce of the genesis block.
    pub nonce: U256,
    /// The timestamp of the genesis block.
    pub timestamp: U256,
    /// The extra data of the genesis block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_data: Option<Bytes>,
    /// The gas limit of the genesis block.
    pub gas_limit: U256,
    /// The difficulty of the genesis block.
    pub difficulty: U256,
    /// The mix hash of the genesis block.
    pub mix_hash: B256,
    /// The coinbase address of the genesis block.
    pub coinbase: Address,
    /// The allocated accounts in the genesis block.
    pub alloc: BTreeMap<Address, AccountState>,
    /// The genesis block number.
    pub number: U256,
    /// The gas used in the genesis block.
    pub gas_used: U256,
    /// The parent hash of the genesis block.
    pub parent_hash: BlockHash,
    /// The base fee per gas of the genesis block.
    #[serde(rename = "baseFeePerGas")]
    pub base_fee: U256,
    /// The excess blob gas of the genesis block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excess_blob_gas: Option<U256>,
    /// The blob gas used of the genesis block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_gas_used: Option<U256>,
    /// The state hash of the genesis block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_hash: Option<B256>,
}

/// The chain configuration.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChainConfig {
    /// The chain ID.
    pub chain_id: ChainId,
    /// The homestead block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homestead_block: Option<BlockNumber>,
    /// The dao fork block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dao_fork_block: Option<BlockNumber>,
    /// The dao fork support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dao_fork_support: Option<bool>,
    /// The eip150 block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip150_block: Option<BlockNumber>,
    /// The eip155 block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip155_block: Option<BlockNumber>,
    /// The eip158 block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip158_block: Option<BlockNumber>,
    /// The byzantium block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byzantium_block: Option<BlockNumber>,
    /// The constantinople block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constantinople_block: Option<BlockNumber>,
    /// The petersburg block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub petersburg_block: Option<BlockNumber>,
    /// The istanbul block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub istanbul_block: Option<BlockNumber>,
    /// The muir glacier block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muir_glacier_block: Option<BlockNumber>,
    /// The berlin block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub berlin_block: Option<BlockNumber>,
    /// The london block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub london_block: Option<BlockNumber>,
    /// The shanghai block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow_glacier_block: Option<BlockNumber>,
    /// The gray glacier block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gray_glacier_block: Option<BlockNumber>,
    /// The kiev block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_netsplit_block: Option<BlockNumber>,
    /// The shanghai block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shanghai_time: Option<u64>,
    /// The cancun block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancun_time: Option<u64>,
    /// The prague block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prague_time: Option<u64>,
    /// The verkle block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verkle_time: Option<u64>,
    /// The bedrock block number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bedrock_block: Option<BlockNumber>,
    /// The regolith block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regolith_time: Option<u64>,
    /// The canyon block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canyon_time: Option<u64>,
    /// The ecotone block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecotone_time: Option<u64>,
    /// The fjord block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fjord_time: Option<u64>,
    /// The granite block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granite_time: Option<u64>,
    /// The holocene block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holocene_time: Option<u64>,
    /// The interop block time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interop_time: Option<u64>,
    /// The terminal total difficulty (TTD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_total_difficulty: Option<u128>,
    /// Whether the terminal total difficulty has passed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_total_difficulty_passed: Option<bool>,
    /// The optimism configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimism: Option<OptimismConfig>,
}

/// The optimism configuration.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OptimismConfig {
    /// The eip1559 elasticity.
    eip1559_elasticity: u64,
    /// The eip1559 denominator.
    eip1559_denominator: u64,
    /// The eip1559 elasticity beginning with the canyon fork.
    #[serde(skip_serializing_if = "Option::is_none")]
    eip1559_denominator_canyon: Option<u64>,
}

/// Represents the state of an account.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountState {
    /// The optional balance of the account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<U256>,
    /// The optional code of the account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<Bytes>,
    /// The optional nonce of the account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,
    /// The storage of the account.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub storage: BTreeMap<B256, B256>,
}

/// The fault proof status is the result of executing the fault proof program.
#[derive(Serialize_repr, Deserialize_repr, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FaultProofStatus {
    /// The claim is valid.
    #[default]
    Valid = 0,
    /// The claim is invalid.
    Invalid = 1,
    /// Executing the program resulted in a panic.
    Panic = 2,
    /// The program has not exited.
    Unfinished = 3,
    /// The status is unknown.
    Unknown,
}

impl TryFrom<u8> for FaultProofStatus {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FaultProofStatus::Valid),
            1 => Ok(FaultProofStatus::Invalid),
            2 => Ok(FaultProofStatus::Panic),
            3 => Ok(FaultProofStatus::Unfinished),
            _ => Ok(FaultProofStatus::Unknown),
        }
    }
}

impl From<FaultProofStatus> for u8 {
    fn from(status: FaultProofStatus) -> u8 {
        status as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_fault_proof_status() {
        let statuses = vec![
            FaultProofStatus::Valid,
            FaultProofStatus::Invalid,
            FaultProofStatus::Panic,
            FaultProofStatus::Unfinished,
            FaultProofStatus::Unknown,
        ];

        for status in statuses {
            let serialized_status =
                serde_json::to_string(&status).expect("failed to serialize status");
            let deserialized_status = serde_json::from_str::<FaultProofStatus>(&serialized_status)
                .expect("failed to deserialize status");
            assert_eq!(status, deserialized_status);
        }
    }

    #[test]
    fn test_serialize_fault_proof_inputs() {
        let inputs = FaultProofInputs {
            l1_head: B256::from([1; 32]),
            l2_head: B256::from([2; 32]),
            l2_claim: B256::from([3; 32]),
            l2_output_root: B256::from([4; 32]),
            l2_block_number: 1337,
            chain_definition: Default::default(),
        };

        let serialized_inputs = serde_json::to_string(&inputs).expect("failed to serialize inputs");
        let deserialized_inputs = serde_json::from_str::<FaultProofInputs>(&serialized_inputs)
            .expect("failed to deserialize inputs");
        assert_eq!(inputs, deserialized_inputs);
    }

    #[test]
    fn test_serialize_fault_proof_fixture() {
        let mut witness_data = BTreeMap::new();
        witness_data.insert(B256::from([1; 32]), Bytes::from([1; 32]));
        witness_data.insert(B256::from([2; 32]), Bytes::from([2; 32]));

        let fixture = FaultProofFixture {
            inputs: FaultProofInputs {
                l1_head: B256::from([1; 32]),
                l2_head: B256::from([2; 32]),
                l2_claim: B256::from([3; 32]),
                l2_output_root: B256::from([4; 32]),
                l2_block_number: 1337,
                chain_definition: Default::default(),
            },
            expected_status: FaultProofStatus::Valid,
            witness_data,
        };

        let serialized_fixture =
            serde_json::to_string(&fixture).expect("failed to serialize fixture");
        let deserialized_fixture = serde_json::from_str::<FaultProofFixture>(&serialized_fixture)
            .expect("failed to deserialize fixture");
        assert_eq!(fixture, deserialized_fixture);
    }
}
