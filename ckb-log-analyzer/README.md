# ckb-log-analyzer

A simple tool to analyse ckb node log, draw a (timestamp->tip_height) synchronize graph.
```bash

$ ckb-log-analyzer --help
Usage: ckb-log-analyzer <COMMAND>

Commands:
  draw
  analyse
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help information
```

## Example:
```bash
ckb-log-analyzer draw \
  --logs-path ckb-develop/data/logs/run.log --labels "Develop Branch" \
  --logs-path ckb-block-queue-20ms/data/logs/run.log  --labels "Use ArrayQueue Branch" \
  --outdir /tmp/
```



# CKB Log Analyze Result

## height and timestamp on sync progress
![inline](static/time_height.png)
## epoch and average block size on sync progress
![inline](static/epoch_average_block_size.png)
## block_size and height on sync progress
![inline image from static/full.png](static/height_block_size.png)
## epoch and average txs count on sync progress
![inline](static/epoch_average_txs_count.png)
## height and txs count on sync progress
![inline](static/height_txs_count.png)
## epoch and average cycles on sync progress
![inline](static/epoch_average_cycles.png)
## height and cycles on sync progress
![inline](static/height_cycles.png)
## Red: yamux-1M window size vs Blue(big ArrayQueue)
![inline](static/time_heihgt_big_queue_vs_yamux.png)
## All
- red: nothing changed
- green: change yamux window size to 1M
- blue: change yamux window size to 1M & use crossbeam_queue:ArrayQueue
- black: disable p2p module, load headers and blocks from a latest-top-synced ChainDb
![inline](static/all.png)

