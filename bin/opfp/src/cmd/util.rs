use alloy_eips::eip1559::BaseFeeParams;
use alloy_primitives::{Address, B256};
use alloy_provider::{Provider, ReqwestProvider};
use byteorder::{BigEndian, ReadBytesExt};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// Represents the response containing the l2 output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputResponse {
    /// The output format version.
    pub version: B256,
    /// The hash of the output.
    pub output_root: B256,
    /// The l2 block reference of this output.
    pub block_ref: L2BlockRef,
    /// The storage root of the message passer contract.
    pub withdrawal_storage_root: B256,
    /// The state root at this block reference.
    pub state_root: B256,
}

/// Represents the reference to an L2 block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L2BlockRef {
    /// The hash of the block.
    pub hash: B256,
    /// The number of the block.
    pub number: u64,
    /// The parent hash of the block.
    pub parent_hash: B256,
    /// The timestamp of the block.
    pub timestamp: u64,
    /// The l1 origin of the block.
    #[serde(rename = "l1origin")]
    pub l1_origin: BlockID,
    /// The sequence number of the block.
    pub sequence_number: u64,
}

/// Represents the response containing the safe head information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeHeadResponse {
    /// The L1 block reference of the safe head.
    pub l1_block: BlockID,
    /// The L2 block reference of the safe head.
    pub safe_head: BlockID,
}

/// A provider for the rollup node.
#[derive(Debug)]
pub struct RollupProvider {
    /// The inner Ethereum JSON-RPC provider.
    inner: ReqwestProvider,
}

impl RollupProvider {
    /// Creates a new [RollupProvider] with the given alloy provider.
    pub fn new(inner: ReqwestProvider) -> Self {
        Self { inner }
    }

    /// Returns the output at a given block number.
    pub async fn output_at_block(&self, block_number: u64) -> Result<OutputResponse> {
        let block_num_hex = format!("0x{:x}", block_number);
        let raw_output = self
            .inner
            .raw_request("optimism_outputAtBlock".into(), (block_num_hex,))
            .await?;
        let output: OutputResponse = serde_json::from_value(raw_output)?;
        Ok(output)
    }

    /// Returns the safe head at an L1 block number.
    pub async fn safe_head_at_block(&self, block_number: u64) -> Result<SafeHeadResponse> {
        let block_num_hex = format!("0x{:x}", block_number);
        let raw_resp = self
            .inner
            .raw_request("optimism_safeHeadAtL1Block".into(), (block_num_hex,))
            .await?;
        let resp: SafeHeadResponse = serde_json::from_value(raw_resp)?;
        Ok(resp)
    }

    /// Creates a new [RollupProvider] from the provided [reqwest::Url].
    pub fn new_http(url: reqwest::Url) -> Self {
        // let pb = ProviderBuilder::default().
        let inner = ReqwestProvider::new_http(url);
        Self::new(inner)
    }
}

/// RollupConfig type compatible with the Optimism rollup node.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollupConfig {
    /// The genesis information.
    pub genesis: Genesis,
    /// The block time.
    pub block_time: u64,
    /// The maximum sequencer drift.
    pub max_sequencer_drift: u64,
    /// The sequence window size.
    pub seq_window_size: u64,

    /// The channel timeout beginning with bedrock.
    #[serde(rename = "channel_timeout")]
    pub channel_timeout_bedrock: u64,
    // The L1 chain ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l1_chain_id: Option<u128>,
    // The L2 chain ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l2_chain_id: Option<u128>,

    /// The regolith activation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regolith_time: Option<u64>,
    /// The canyon activation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canyon_time: Option<u64>,
    /// The delta activation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta_time: Option<u64>,
    /// The ecotone activation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ecotone_time: Option<u64>,
    /// The fjord activation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fjord_time: Option<u64>,
    /// The granite activation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub granite_time: Option<u64>,
    /// The interop activation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interop_time: Option<u64>,
    /// The holocene_time activation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holocene_time: Option<u64>,
    /// The batch inbox address.
    pub batch_inbox_address: Address,
    /// The deposit contract address.
    pub deposit_contract_address: Address,
    /// The L1 system config address.
    pub l1_system_config_address: Address,
    /// The protocol versions address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_versions_address: Option<Address>,
    /// The DA challenge address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub da_challenge_address: Option<Address>,
}

impl From<&superchain_primitives::RollupConfig> for RollupConfig {
    fn from(cfg: &superchain_primitives::RollupConfig) -> Self {
        let syscfg = cfg.genesis.system_config.clone().unwrap();
        let genesis = Genesis {
            l1: cfg.genesis.l1.into(),
            l2: cfg.genesis.l2.into(),
            l2_time: cfg.genesis.l2_time,
            system_config: SystemConfig {
                batcher_addr: syscfg.batcher_address,
                overhead: syscfg.overhead.into(),
                scalar: syscfg.scalar.into(),
                gas_limit: syscfg.gas_limit,
            },
        };
        let rollup_config = Self {
            genesis: genesis.clone(),
            block_time: cfg.block_time,
            max_sequencer_drift: cfg.max_sequencer_drift,
            seq_window_size: cfg.seq_window_size,
            channel_timeout_bedrock: cfg.channel_timeout,
            // channel_timeout_granite: cfg.granite_channel_timeout,
            l1_chain_id: Some(cfg.l1_chain_id.into()),
            l2_chain_id: Some(cfg.l2_chain_id.into()),
            regolith_time: cfg.regolith_time,
            canyon_time: cfg.canyon_time,
            delta_time: cfg.delta_time,
            ecotone_time: cfg.ecotone_time,
            fjord_time: cfg.fjord_time,
            granite_time: cfg.granite_time,
            interop_time: None,
            holocene_time: cfg.holocene_time,
            batch_inbox_address: cfg.batch_inbox_address,
            deposit_contract_address: cfg.deposit_contract_address,
            l1_system_config_address: cfg.l1_system_config_address,
            protocol_versions_address: Some(cfg.protocol_versions_address),
            da_challenge_address: cfg.da_challenge_address,
            // da_challenge_window: 0,
            // da_resolve_window: 0,
            // use_plasma: false,
        };
        rollup_config
    }
}

impl Into<superchain_primitives::RollupConfig> for RollupConfig {
    fn into(self) -> superchain_primitives::RollupConfig {
        superchain_primitives::RollupConfig {
            genesis: superchain_primitives::ChainGenesis {
                l1: self.genesis.l1.into(),
                l2: self.genesis.l2.into(),
                l2_time: self.genesis.l2_time,
                extra_data: None,
                system_config: Some(superchain_primitives::SystemConfig {
                    batcher_address: self.genesis.system_config.batcher_addr,
                    overhead: self.genesis.system_config.overhead.into(),
                    scalar: self.genesis.system_config.scalar.into(),
                    gas_limit: self.genesis.system_config.gas_limit,
                    base_fee_scalar: None,
                    blob_base_fee_scalar: None,
                }),
            },
            block_time: self.block_time,
            max_sequencer_drift: self.max_sequencer_drift,
            seq_window_size: self.seq_window_size,
            channel_timeout: self.channel_timeout_bedrock,
            granite_channel_timeout: 50,
            l1_chain_id: u64::try_from(self.l1_chain_id.unwrap_or(0)).unwrap(),
            l2_chain_id: u64::try_from(self.l2_chain_id.unwrap_or(0)).unwrap(),
            base_fee_params: BaseFeeParams::optimism(),
            canyon_base_fee_params: Some(BaseFeeParams::optimism_canyon()),
            regolith_time: self.regolith_time,
            canyon_time: self.canyon_time,
            delta_time: self.delta_time,
            ecotone_time: self.ecotone_time,
            fjord_time: self.fjord_time,
            granite_time: self.granite_time,
            holocene_time: self.holocene_time,
            batch_inbox_address: self.batch_inbox_address,
            deposit_contract_address: self.deposit_contract_address,
            l1_system_config_address: self.l1_system_config_address,
            protocol_versions_address: self.protocol_versions_address.unwrap_or_default(),
            superchain_config_address: None,
            blobs_enabled_l1_timestamp: None,
            da_challenge_address: self.da_challenge_address,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Genesis {
    pub l1: BlockID,
    pub l2: BlockID,
    pub l2_time: u64,
    pub system_config: SystemConfig,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockID {
    pub hash: B256,
    pub number: u64,
}

impl Into<superchain_primitives::BlockID> for BlockID {
    fn into(self) -> superchain_primitives::BlockID {
        superchain_primitives::BlockID {
            hash: self.hash,
            number: self.number,
        }
    }
}

impl From<superchain_primitives::BlockID> for BlockID {
    fn from(id: superchain_primitives::BlockID) -> Self {
        Self {
            hash: id.hash,
            number: id.number,
        }
    }
}

// https://github.com/ethereum-optimism/optimism/blob/c7ad0ebae5dca3bf8aa6f219367a95c15a15ae41/op-service/eth/types.go#L371
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    pub batcher_addr: Address,
    pub overhead: B256,
    pub scalar: B256,
    pub gas_limit: u64,
}

pub trait HasStep {
    fn step(&self) -> u64;
}

pub struct VersionedState {
    pub version: u8,
    pub state: Box<dyn HasStep>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SingleThreadedFPVMState {
    pub memory: Memory,
    pub preimage_key: B256,
    // assuming 32-bit machine
    pub perimage_offset: u32,
    pub cpu: CpuScalars,
    pub heap: u32,
    pub exit_code: u8,
    pub exited: bool,
    pub step: u64,
    pub registers: [u32; 32],
    pub last_hint: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MultiThreadedV2State {
    pub memory: Memory,
    pub preimage_key: B256,
    // assuming 32-bit machine
    pub perimage_offset: u32,
    pub heap: u32,
    pub ll_reservation_status: u8,
    pub ll_address: u32,
    pub ll_owner_thread: u32,
    pub exit_code: u8,
    pub exited: bool,
    pub step: u64,
    pub steps_since_last_context_switch: u64,
    pub traverse_right: bool,
    pub left_thread_stack: Vec<ThreadState>,
    pub right_thread_stack: Vec<ThreadState>,
    pub next_thread_id: u32,
    pub last_hint: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MultiThreaded64V3 {
    pub memory: Memory,
    pub preimage_key: B256,
    // assuming 32-bit machine
    pub perimage_offset: u64,
    pub heap: u64,
    pub ll_reservation_status: u8,
    pub ll_address: u64,
    pub ll_owner_thread: u64,
    pub exit_code: u8,
    pub exited: bool,
    pub step: u64,
    pub steps_since_last_context_switch: u64,
    pub traverse_right: bool,
    pub left_thread_stack: Vec<ThreadState64>,
    pub right_thread_stack: Vec<ThreadState64>,
    pub next_thread_id: u64,
    pub last_hint: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThreadState {
    pub thread_id: u32,
    pub exit_code: u8,
    pub exited: bool,
    pub cpu: CpuScalars,
    pub registers: [u32; 32],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThreadState64 {
    pub thread_id: u64,
    pub exit_code: u8,
    pub exited: bool,
    pub cpu: CpuScalars64,
    pub registers: [u64; 32],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Memory {
    pub pages: HashMap<u32, [u8; 4096]>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CpuScalars {
    pub pc: u32,
    pub next_pc: u32,
    pub lo: u32,
    pub hi: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CpuScalars64 {
    pub pc: u64,
    pub next_pc: u64,
    pub lo: u64,
    pub hi: u64,
}

enum CannonVersion {
    SingleThreaded2 = 2,
    MultiThreadedV2 = 5,
    MultiThreaded64V3 = 6,
}

impl TryFrom<u8> for CannonVersion {
    type Error = String;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            2 => Ok(CannonVersion::SingleThreaded2),
            5 => Ok(CannonVersion::MultiThreadedV2),
            6 => Ok(CannonVersion::MultiThreaded64V3),
            _ => Err(format!("invalid cannon state version: {v}").to_string()),
        }
    }
}

trait Decodable {
    fn decode<T>(&mut self, cursor: &mut Cursor<T>) -> Result<()>
    where
        T: AsRef<[u8]>;
}

impl TryFrom<Vec<u8>> for VersionedState {
    type Error = String;

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        let mut v = VersionedState {
            version: 0,
            state: Box::new(SingleThreadedFPVMState::default()),
        };
        let mut cursor = Cursor::new(buffer);
        let result = v.decode(&mut cursor);
        return match result {
            Ok(_) => Ok(v),
            Err(err) => Err(format!("invalid versioned state encoding: {err}").to_string()),
        };
    }
}

impl Decodable for VersionedState {
    fn decode<T>(&mut self, cursor: &mut Cursor<T>) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        self.version = cursor.read_u8()?;

        let version_state_cannon = CannonVersion::try_from(self.version).unwrap();
        match version_state_cannon {
            CannonVersion::SingleThreaded2 => {
                let mut single_threaded_fpvmstate = SingleThreadedFPVMState::default();
                single_threaded_fpvmstate.decode(cursor)?;
                self.state = Box::new(single_threaded_fpvmstate);
                Ok(())
            }
            CannonVersion::MultiThreadedV2 => {
                let mut mutli_threaded_v2 = MultiThreadedV2State::default();
                mutli_threaded_v2.decode(cursor)?;
                self.state = Box::new(mutli_threaded_v2);
                Ok(())
            }
            CannonVersion::MultiThreaded64V3 => {
                let mut mutli_threaded_64_v3 = MultiThreaded64V3::default();
                mutli_threaded_64_v3.decode(cursor)?;
                self.state = Box::new(mutli_threaded_64_v3);
                Ok(())
            }
        }
    }
}

impl Decodable for SingleThreadedFPVMState {
    fn decode<T>(&mut self, cursor: &mut Cursor<T>) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        self.memory.decode(cursor)?;

        let mut preimage_key_buffer: [u8; 32] = [0; 32];
        cursor.read_exact(&mut preimage_key_buffer)?;
        self.preimage_key = B256::from(&preimage_key_buffer);
        self.perimage_offset = cursor.read_u32::<BigEndian>()?;

        self.cpu.pc = cursor.read_u32::<BigEndian>()?;
        self.cpu.next_pc = cursor.read_u32::<BigEndian>()?;
        self.cpu.lo = cursor.read_u32::<BigEndian>()?;
        self.cpu.hi = cursor.read_u32::<BigEndian>()?;

        self.heap = cursor.read_u32::<BigEndian>()?;
        self.exit_code = cursor.read_u8()?;

        self.exited = cursor.read_u8()? != 0;
        self.step = cursor.read_u64::<BigEndian>()?;

        for i in 0..self.registers.len() {
            self.registers[i] = cursor.read_u32::<BigEndian>()?;
        }

        let last_hint_len = cursor.read_u32::<BigEndian>()?;
        if last_hint_len > 0 {
            let mut slice = vec![0; last_hint_len.try_into().unwrap()];
            cursor.read_exact(&mut slice)?;

            self.last_hint = slice;
        }

        Ok(())
    }
}

impl HasStep for SingleThreadedFPVMState {
    fn step(&self) -> u64 {
        self.step
    }
}

impl Decodable for MultiThreadedV2State {
    fn decode<T>(&mut self, cursor: &mut Cursor<T>) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        self.memory.decode(cursor)?;

        let mut preimage_key_buffer: [u8; 32] = [0; 32];
        cursor.read_exact(&mut preimage_key_buffer)?;
        self.preimage_key = B256::from(&preimage_key_buffer);
        self.perimage_offset = cursor.read_u32::<BigEndian>()?;

        self.heap = cursor.read_u32::<BigEndian>()?;

        self.ll_reservation_status = cursor.read_u8()?;
        self.ll_address = cursor.read_u32::<BigEndian>()?;
        self.ll_owner_thread = cursor.read_u32::<BigEndian>()?;

        self.exit_code = cursor.read_u8()?;
        self.exited = cursor.read_u8()? != 0;

        self.step = cursor.read_u64::<BigEndian>()?;
        self.steps_since_last_context_switch = cursor.read_u64::<BigEndian>()?;

        self.traverse_right = cursor.read_u8()? != 0;
        self.next_thread_id = cursor.read_u32::<BigEndian>()?;

        let left_thread_stack_size = cursor.read_u32::<BigEndian>()?;
        let mut left_thread_stack = Vec::new();
        for _ in 0..left_thread_stack_size {
            let mut thread_state = ThreadState::default();
            thread_state.decode(cursor)?;
            left_thread_stack.push(thread_state);
        }
        self.left_thread_stack = left_thread_stack;

        let right_thread_stack_size = cursor.read_u32::<BigEndian>()?;
        let mut right_thread_stack = Vec::new();
        for _ in 0..right_thread_stack_size {
            let mut thread_state = ThreadState::default();
            thread_state.decode(cursor)?;
            right_thread_stack.push(thread_state);
        }
        self.right_thread_stack = right_thread_stack;

        let last_hint_len = cursor.read_u32::<BigEndian>()?;
        if last_hint_len > 0 {
            let mut slice = vec![0; last_hint_len.try_into().unwrap()];
            cursor.read_exact(&mut slice)?;

            self.last_hint = slice;
        }

        Ok(())
    }
}

impl HasStep for MultiThreadedV2State {
    fn step(&self) -> u64 {
        self.step
    }
}

impl Decodable for MultiThreaded64V3 {
    fn decode<T>(&mut self, cursor: &mut Cursor<T>) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        self.memory.decode(cursor)?;

        let mut preimage_key_buffer: [u8; 32] = [0; 32];
        cursor.read_exact(&mut preimage_key_buffer)?;
        self.preimage_key = B256::from(&preimage_key_buffer);
        self.perimage_offset = cursor.read_u64::<BigEndian>()?;

        self.heap = cursor.read_u64::<BigEndian>()?;

        self.ll_reservation_status = cursor.read_u8()?;
        self.ll_address = cursor.read_u64::<BigEndian>()?;
        self.ll_owner_thread = cursor.read_u64::<BigEndian>()?;

        self.exit_code = cursor.read_u8()?;
        self.exited = cursor.read_u8()? != 0;

        self.step = cursor.read_u64::<BigEndian>()?;
        self.steps_since_last_context_switch = cursor.read_u64::<BigEndian>()?;

        self.traverse_right = cursor.read_u8()? != 0;
        self.next_thread_id = cursor.read_u64::<BigEndian>()?;

        let left_thread_stack_size = cursor.read_u64::<BigEndian>()?;
        let mut left_thread_stack = Vec::new();
        for _ in 0..left_thread_stack_size {
            let mut thread_state_64 = ThreadState64::default();
            thread_state_64.decode(cursor)?;
            left_thread_stack.push(thread_state_64);
        }
        self.left_thread_stack = left_thread_stack;

        let right_thread_stack_size = cursor.read_u64::<BigEndian>()?;
        let mut right_thread_stack = Vec::new();
        for _ in 0..right_thread_stack_size {
            let mut thread_state_64 = ThreadState64::default();
            thread_state_64.decode(cursor)?;
            right_thread_stack.push(thread_state_64);
        }
        self.right_thread_stack = right_thread_stack;

        let last_hint_len = cursor.read_u32::<BigEndian>()?;
        if last_hint_len > 0 {
            let mut slice = vec![0; last_hint_len.try_into().unwrap()];
            cursor.read_exact(&mut slice)?;

            self.last_hint = slice;
        }

        Ok(())
    }
}

impl HasStep for MultiThreaded64V3 {
    fn step(&self) -> u64 {
        self.step
    }
}

impl Decodable for Memory {
    fn decode<T>(&mut self, cursor: &mut Cursor<T>) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        let page_count = cursor.read_u32::<BigEndian>()?;

        if page_count > 0 {
            self.pages = HashMap::new();
        }

        for _i in 0..page_count {
            let page_index = cursor.read_u32::<BigEndian>()?;
            let mut data: [u8; 4096] = [0; 4096];
            cursor.read_exact(&mut data)?;
            self.pages.insert(page_index, data);
        }

        Ok(())
    }
}

impl Decodable for ThreadState {
    fn decode<T>(&mut self, cursor: &mut Cursor<T>) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        self.thread_id = cursor.read_u32::<BigEndian>()?;
        self.exit_code = cursor.read_u8()?;
        self.exited = cursor.read_u8()? != 0;

        self.cpu.pc = cursor.read_u32::<BigEndian>()?;
        self.cpu.next_pc = cursor.read_u32::<BigEndian>()?;
        self.cpu.lo = cursor.read_u32::<BigEndian>()?;
        self.cpu.hi = cursor.read_u32::<BigEndian>()?;

        for i in 0..self.registers.len() {
            self.registers[i] = cursor.read_u32::<BigEndian>()?;
        }

        Ok(())
    }
}

impl Decodable for ThreadState64 {
    fn decode<T>(&mut self, cursor: &mut Cursor<T>) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        self.thread_id = cursor.read_u64::<BigEndian>()?;
        self.exit_code = cursor.read_u8()?;
        self.exited = cursor.read_u8()? != 0;

        self.cpu.pc = cursor.read_u64::<BigEndian>()?;
        self.cpu.next_pc = cursor.read_u64::<BigEndian>()?;
        self.cpu.lo = cursor.read_u64::<BigEndian>()?;
        self.cpu.hi = cursor.read_u64::<BigEndian>()?;

        for i in 0..self.registers.len() {
            self.registers[i] = cursor.read_u64::<BigEndian>()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cmd::util::{CpuScalars, Memory, SingleThreadedFPVMState, VersionedState};
    use alloy_primitives::{hex, Uint, B256};
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn test_decode_versioned_state() {
        // Test taken from: https://github.com/ethereum-optimism/optimism/blob/969382a3ff0fb577a7fda6287f3c74f8c26dce53/cannon/mipsevm/singlethreaded/state_test.go#L115
        let mut correct_memory = Memory {
            pages: HashMap::new(),
        };
        let correct_page_data_1: [u8; 4096] = [0; 4096];
        correct_memory.pages.insert(5, correct_page_data_1);

        let mut correct_page_data_2: [u8; 4096] = [0; 4096];
        correct_page_data_2[2] = 0x01;
        correct_memory.pages.insert(123, correct_page_data_2);

        let mut registers: [u32; 32] = [0; 32];
        registers[0] = 0xdeadbeef;
        registers[1] = 0xdeadbeef;
        registers[2] = 0xc0ffee;
        registers[3] = 0xbeefbabe;
        registers[4] = 0xdeadc0de;
        registers[5] = 0xbadc0de;
        registers[6] = 0xdeaddead;

        let correct_state = SingleThreadedFPVMState {
            memory: correct_memory,
            preimage_key: B256::from(hex!(
                "ff00000000000000000000000000000000000000000000000000000000000000"
            )),
            perimage_offset: 5,
            cpu: CpuScalars {
                pc: 0xff,
                next_pc: 0xff + 4,
                lo: 0xbeef,
                hi: 0xbabe,
            },
            heap: 0xc0ffee,
            exit_code: 1,
            exited: true,
            step: 0xdeadbeef,
            registers: registers,
            last_hint: vec![1u8, 2u8, 3u8, 4u8, 5u8],
        };

        let test_data = hex!("020000000200000005000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000007b00000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ff0000000000000000000000000000000000000000000000000000000000000000000005000000ff000001030000beef0000babe00c0ffee010100000000deadbeefdeadbeefdeadbeef00c0ffeebeefbabedeadc0de0badc0dedeaddead00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000050102030405");
        let test_data_vec: Vec<u8> = test_data.to_vec();
        let v = VersionedState::try_from(test_data_vec).unwrap();

        assert_eq!(v.single_threaded_fpvmstate, correct_state);
    }
}
