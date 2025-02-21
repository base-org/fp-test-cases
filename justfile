set dotenv-load := true

opfp := if `which opfp || true` != "" { `which opfp` } else { "target/debug/opfp" }
op-program := if `which op-program || true` != "" { `which op-program` } else { join(env("OPTIMISM_DIR"), "op-program/bin/op-program") }
cannon-dir := if `which cannon || true` != "" { parent_directory(parent_directory(`which cannon`)) } else { join(env("OPTIMISM_DIR"), "cannon") }
cannon-bin := join(cannon-dir, "bin/cannon")
cannon-state := join(cannon-dir, "state.bin.gz")
cannon-meta := join(cannon-dir, "meta.json")
enclave := "devnet"
devnet-config-file := "devnet/standard.yaml"
account := "TEST"
name := "Writer"
script-file := name + ".s.sol"
l2-block-gas-limit := "60000000"

# Space-separated list of script arguments

script-args := "1000000"
script-signature := "run(" + replace_regex(replace_regex(replace_regex(replace_regex(script-args, "0x[0-9a-fA-F]{40}", "address"), "\\d+", "uint256"), "(true|false)", "bool"), " ", ",") + ")"
expanded-name := replace_regex(trim(name + " " + script-args), " ", "-")
fixture-file := join("fixtures", expanded-name + ".json")
op-program-output := join("output", "op-program", file_name(fixture-file))
cannon-output := join("output", "cannon", file_name(fixture-file))
verbosity := "-vv"
genesis-path := "op-deployer-configs/genesis-2151908.json"
rollup-path := "op-deployer-configs/rollup-2151908.json"

wallets-path := "op-deployer-configs/wallets.json"

# default recipe to display help information
default:
    @just --list

# Fixes and checks all workspace formatting
fmt: fmt-fix fmt-check

# Fixes the formatting of the workspace
fmt-fix:
    cargo +nightly fmt --all

# Check the formatting of the workspace
fmt-check:
    cargo +nightly fmt --all -- --check

# Run clippy lints on the workspace
clippy:
    cargo +nightly clippy --workspace --all --all-features --all-targets -- -D warnings

# Build for the native target
build *args='':
    cargo build --workspace --all $@

# Shuts down and removes the local devnet
cleanup-devnet:
    kurtosis enclave rm {{ enclave }} -f

# Creates a new local devnet
create-devnet:
    kurtosis run github.com/ethpandaops/optimism-package \
        --args-file {{ devnet-config-file }} \
        --enclave {{ enclave }}

# Generates a fixture for the given script (name) and arguments (script-args)
generate-fixture:
    #!/bin/bash
    set -e

    L2_RPC_URL={{ shell("kurtosis service inspect " + enclave + " op-el-1-op-geth-op-node-op-kurtosis | grep -- ' rpc: ' | sed 's/.*-> //'") }}
    ROLLUP_URL={{ shell("kurtosis service inspect " + enclave + " op-cl-1-op-node-op-geth-op-kurtosis | grep -- ' http: ' | sed 's/.*-> //'") }}

    forge script \
        --non-interactive \
        --password="" \
        --rpc-url $L2_RPC_URL \
        --account {{ account }} \
        --broadcast \
        --sig "{{ script-signature }}" \
        script/{{ script-file }} \
        {{ script-args }}

    rm -rf op-deployer-configs
    kurtosis files download {{ enclave }} op-deployer-configs

    L2_BLOCK_NUM=$(($(jq < broadcast/{{ script-file }}/2151908/run-latest.json '.receipts[0].blockNumber' -r)))

    while true; do
        SYNC_STATUS=$(cast rpc optimism_syncStatus --rpc-url $ROLLUP_URL)
        L2_SAFE_BLOCK_NUM=$(echo $SYNC_STATUS | jq '.safe_l2.number')
        L1_BLOCK_NUM=$(echo $SYNC_STATUS | jq '.head_l1.number')
        if [ $L2_SAFE_BLOCK_NUM -ge $(($L2_BLOCK_NUM)) ]; then
            break
        fi
        echo "Waiting for L2 block $L2_BLOCK_NUM to be safe..., currently at $L2_SAFE_BLOCK_NUM"
        sleep 10
    done

    mkdir -p {{ parent_directory(fixture-file) }}

    {{ opfp }} from-op-program \
        --op-program {{ op-program }} \
        --l2-block $L2_BLOCK_NUM \
        --l1-block $L1_BLOCK_NUM \
        --l1-rpc-url {{ "http://" + shell("kurtosis service inspect " + enclave + " el-1-geth-teku | grep -- ' rpc: ' | sed 's/.*-> //'") }} \
        --l2-rpc-url $L2_RPC_URL \
        --beacon-url {{ shell("kurtosis service inspect " + enclave + " cl-1-teku-geth | grep -- ' http: ' | sed 's/.*-> //'") }} \
        --rollup-url $ROLLUP_URL \
        --rollup-path {{ rollup-path }} \
        --genesis-path {{ genesis-path }} \
        --output {{ fixture-file }} \
        {{ verbosity }}

# Runs the given fixture through the op-program
run-fixture:
    mkdir -p {{ parent_directory(op-program-output) }}

    {{ opfp }} run-op-program \
        --op-program {{ op-program }} \
        --fixture {{ fixture-file }} \
        --output {{ op-program-output }} \
        {{ verbosity }}

# Runs the given fixture through Cannon and op-program
cannon-fixture:
    mkdir -p {{ parent_directory(cannon-output) }}

    {{ opfp }} run-op-program \
        --op-program {{ op-program }} \
        --fixture {{ fixture-file }} \
        --cannon {{ cannon-bin }} \
        --cannon-state {{ cannon-state }} \
        --cannon-meta {{ cannon-meta }} \
        --output {{ cannon-output }} \
        {{ verbosity }}

# Updates the l2 block gas limit using the value specified by l2-block-gas-limit
# e.g: `just l2-block-gas-limit=1000000 update-l2-block-gas-limit`
update-l2-block-gas-limit:
    #!/bin/bash
    set -e

    rm -rf op-deployer-configs
    kurtosis files download {{ enclave }} op-deployer-configs

    SYSTEM_CONFIG_OWNER_PRIVATE_KEY={{ shell("cat " + wallets-path + " | jq '.[].systemConfigOwnerPrivateKey'") }}
    L1_SYSTEM_CONFIG_ADDRESS={{ shell("cat " + rollup-path + " | jq '.l1_system_config_address'") }}
    L1_RPC_URL=$(kurtosis service inspect {{ enclave }} el-1-geth-teku | grep -- ' rpc: ' | sed 's/.*-> //')

    cast send \
        --private-key $SYSTEM_CONFIG_OWNER_PRIVATE_KEY \
        --rpc-url $L1_RPC_URL \
        $L1_SYSTEM_CONFIG_ADDRESS \
        "setGasLimit(uint64)" \
        {{ l2-block-gas-limit }}

    L2_BLOCK_NUM=$(($(jq < broadcast/{{ script-file }}/2151908/run-latest.json '.receipts[0].blockNumber' -r)))

# Queries the L1 SystemConfig contract to return the current L2 block gas limit
get-l2-block-gas-limit:
    #!/bin/bash
    set -e

    rm -rf op-deployer-configs
    kurtosis files download {{ enclave }} op-deployer-configs

    L1_SYSTEM_CONFIG_ADDRESS={{ shell("cat " + rollup-path + " | jq '.l1_system_config_address'") }}
    L1_RPC_URL=$(kurtosis service inspect {{ enclave }} el-1-geth-teku | grep -- ' rpc: ' | sed 's/.*-> //')

    cast call --rpc-url $L1_RPC_URL $L1_SYSTEM_CONFIG_ADDRESS  "gasLimit()(uint64)"
