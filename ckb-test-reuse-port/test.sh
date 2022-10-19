#!/usr/bin/env bash

set -e

pwd
for i in {0..2}; do
  ssh ckb-${i} '
  cd /home/ubuntu
  rm -rf ckb-data/data/network/peer_store && echo removed peer_store
  docker stop ckb && echo stopped ckb container
  docker rm ckb && echo removed ckb container
  '

done
for i in {0..2}; do
  ssh ckb-${i} '
  cd /home/ubuntu/
  docker run -d --name ckb --network host -v "$(realpath ckb-data)":/var/lib/ckb  nervos/ckb:v0.104.1 run && echo started ckb container
  cat /dev/null > ckb-data/data/logs/run.log && echo cleared run.log
'
done