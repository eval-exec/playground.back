#!/usr/bin/env bash
set -eux
test -f /usr/share/nvm/init-nvm.sh && source /usr/share/nvm/init-nvm.sh >&/dev/null || echo "nvm not initialized"
SOLC_0_5_16="/home/vory/Downloads/solc_bin/solc-linux-amd64-v0.5.16"
SOLC_0_6_6="/home/vory/Downloads/solc_bin/solc-linux-amd64-v0.6.6"
test -f ${SOLC_0_5_16} || echo solc v0.5.16 not found
command -v abigen || (echo abigen not found in PATH && exit 1)
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
echo ${script_dir}
echo execute ./run.sh on ${script_dir}
mkdir -p ${script_dir}/contracts/core
mkdir -p ${script_dir}/contracts/periphery

cd v2-core
nvm use v10.24.1
yarn
abigen --solc ${SOLC_0_5_16} --pkg core --sol contracts/UniswapV2Factory.sol --lang go --out ${script_dir}/contracts/core/factory.go

cd ${script_dir}
cd v2-periphery
yarn
cd node_modules
abigen --solc /home/vory/Downloads/solc_bin/solc-linux-amd64-v0.6.6 --pkg periphery --solc ../contracts/UniswapV2Router02.sol --out ${script_dir}/contracts/periphery/router.go


go mod tidy