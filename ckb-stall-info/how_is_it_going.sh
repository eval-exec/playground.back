#!/usr/bin/env bash

set -evuo pipefail

# the first argument is a url for CKB node rpc
# the second argument is a file path for the output
# if arguments count is not 2, print a help message and exit
if [ $# -ne 2 ]; then
	echo "Usage: $0 <ckb-node-rpc-url> <output-file-path>"
	echo ""
	echo "Example: $0 http://127.0.0.1:8114 ckb_debug.log" 
	exit 1
fi

# get first argument as CKB node rpc url
CKB_NODE_RPC_URL=$1

# get second argument as output file path
OUTPUT_FILE_PATH=$2

function fetch_ckb_info {
	# get tip block number
	echo "Getting tip block number..." >>"$OUTPUT_FILE_PATH"
	curl -X POST "${CKB_NODE_RPC_URL}" -H 'Content-Type: application/json' -d '{"jsonrpc": "2.0", "method": "get_tip_header", "params": [], "id": "1"}' >>"${OUTPUT_FILE_PATH}"

	echo "Get peers..." >>"$OUTPUT_FILE_PATH"
	curl -X POST "${CKB_NODE_RPC_URL}" -H 'Content-Type: application/json' -d '{"jsonrpc": "2.0", "method": "get_peers", "params": [], "id": "1"}' >>"${OUTPUT_FILE_PATH}"


	echo "Get Sync State..." >>"$OUTPUT_FILE_PATH"
	curl -X POST "${CKB_NODE_RPC_URL}" -H 'Content-Type: application/json' -d '{"jsonrpc": "2.0", "method": "sync_state", "params": [], "id": "1"}' >>"${OUTPUT_FILE_PATH}"
}


# fetch ckb info 10 times, and sleep every 10 seconds
for _ in {1..2}; do
	fetch_ckb_info
	sleep 10
done
