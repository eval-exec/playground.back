use std::collections::{BTreeMap};
use plotters::prelude::*;
use std::fs::File;
use std::io::BufRead;
use std::rc::Rc;
use ckb_db::RocksDB;
use ckb_db_schema::{COLUMNS};
use ckb_store::{ChainStore, ChainDB};
use itertools::Itertools;
use scan_fmt::scan_fmt;


fn main() {
    let mut mm0 = parse_info_level_log("/home/exec/Projects/github.com/nervosnetwork/ckb-run-log/ckb-run_change-cpu-turbo-state.log");
    let mut mm1 = parse_info_level_log("/home/exec/Projects/github.com/nervosnetwork/ckb-yamux-1M/data/logs/run.log");
    let block_size_mm = export_block_size();
    // fill block_size_mm to mm
    for (height, block_size) in block_size_mm {
        mm0.entry(height).and_modify(|v| v.block_size = block_size);
        mm1.entry(height).and_modify(|v| v.block_size = block_size);
    }

    let epoch_mm0 = height_to_epoch(&mm0);
    let epoch_mm1 = height_to_epoch(&mm1);
    let c = Context {
        epoch_mm0,
        mm0,
        epoch_mm1: Some(epoch_mm1),
        mm1: Some(mm1),
    };
    c.draw_time_cost();

    c.draw_height_block_size();
    c.draw_epoch_average_block_size();

    c.draw_height_cycles();
    c.draw_epoch_cycles();

    c.draw_height_txs_count();
    c.draw_epoch_average_txs_count();
}

struct Context {
    epoch_mm0: BTreeMap<u64, EpochStatics>,
    mm0: BTreeMap<u64, LogStatics>,

    epoch_mm1: Option<BTreeMap<u64, EpochStatics>>,
    mm1: Option<BTreeMap<u64, LogStatics>>,
}

impl Context {
    fn draw_epoch_average_block_size(&self) {
        let points: Vec<(f64, f64)> = self.epoch_mm0.iter().map(|(epoch, status)| {
            (*epoch as f64, status.block_size as f64 / status.block_count as f64)
        }).collect();
        let points1: Option<Vec<(f64, f64)>> = self.epoch_mm1.as_ref().map(|m| m.iter().map(|(epoch, status)| {
            (*epoch as f64, status.block_size as f64 / status.block_count as f64)
        }).collect());
        draw_f64("img/epoch_average_block_size.png",
                 "CKB Sync Status: (epoch, average_block_size)",
                 "epoch",
                 "average_block_size",
                 points,
                 points1,
        ).unwrap();
    }

    fn draw_height_block_size(&self) {
        let points: Vec<(u64, u64)> = self.mm0.iter().map(|(height, status)| (*height, status.block_size)).collect();
        let points1: Option<Vec<(u64, u64)>> = self.mm1.as_ref().map(|m| m.iter().map(|(height, status)| (*height, status.block_size)).collect());

        draw_u64(
            "img/epoch_average_block_size.png",
            "CKB Sync Status: (epoch, average block_size)",
            "epoch",
            "block_size",
            points,
            points1,
        ).unwrap();
    }

    fn draw_height_txs_count(&self) {
        let points: Vec<(u64, u64)> = self.mm0.iter().map(|(height, status)| {
            (*height, status.tx_count)
        }).collect();
        let points1 = self.mm1.as_ref().map(|m| m.iter().map(|(height, status)| {
            (*height, status.tx_count)
        }).collect());

        draw_u64(
            "img/height_txs_count.png",
            "CKB Sync Status: (height, txs_count)",
            "height",
            "txs_count",
            points,
            points1,
        ).unwrap();
    }

    fn draw_epoch_average_txs_count(&self) {
        let points: Vec<(f64, f64)> = self.epoch_mm0.iter().map(|(epoch, statics)| {
            (*epoch as f64, statics.tx_count as f64 / statics.block_count as f64)
        }).collect();
        let points1 = self.epoch_mm1.as_ref().map(|m| m.iter().map(|(epoch, statics)| {
            (*epoch as f64, statics.tx_count as f64 / statics.block_count as f64)
        }).collect());
        draw_f64(
            "img/epoch_average_txs_count.png",
            "CKB Sync Status: (epoch, average txs_count)",
            "epoch",
            "average_txs_count",
            points,
            points1,
        ).unwrap();
    }

    fn draw_epoch_cycles(&self) {
        let points: Vec<(f64, f64)> = self.epoch_mm0.iter().map(|(k, v)| {
            (*k as f64, v.cycles)
        }).collect();

        let points1 = self.epoch_mm1.as_ref().map(|m| m.iter().map(|(k, v)| {
            (*k as f64, v.cycles)
        }).collect());
        draw_f64(
            "img/epoch_average_cycles.png",
            "CKB Sync Status:(epoch,average_cycles)",
            "epoch",
            "average cycles",
            points,
            points1,
        ).unwrap();
    }

    fn draw_height_cycles(&self) {
        let points: Vec<(u64, u64)> = self.mm0.iter().map(|(k, v)| {
            (*k, v.cycles)
        }).collect();
        let points1 = self.mm1.as_ref().map(|m| m.iter().map(|(k, v)| {
            (*k, v.cycles)
        }).collect());
        draw_u64(
            "img/height_cycles.png",
            "CKB Sync Status:(height,cycles)",
            "height",
            "cycles",
            points,
            points1,
        ).unwrap();
    }

    fn draw_time_cost(&self) {
        let points: Vec<(u64, u64)> = self.mm0.iter().filter(|(k, _)| **k % 100 == 0).map(|(k, v)| (*k, v.timestamp)).collect();
        let points1: Option<Vec<(u64, u64)>> = self.mm1.as_ref().map(|mm| mm.iter().filter(|(k, _)| **k % 100 == 0).map(|(k, v)| (*k, v.timestamp)).collect());
        draw_u64(
            "img/time_height.png",
            "CKB Sync Status:(timestamp, height)",
            "timestamp",
            "height",
            points,
            points1,
        ).unwrap()
    }
}

fn export_block_size() -> BTreeMap<u64, u64> {
    let ckb_mainnet_dir = "/home/exec/Projects/github.com/nervosnetwork/ckb-run-log/ckb-main/data/db";
    let db = RocksDB::open_in(&ckb_mainnet_dir, COLUMNS);
    let store = ChainDB::new(db, Default::default());

    let mut mm = BTreeMap::<u64, u64>::new();
    for number in 0..8162480 {
        let block_hash = store.get_block_hash(number).unwrap();

        let packed_block_size = store.get_packed_block(&block_hash).unwrap().total_size();
        mm.insert(number, packed_block_size as u64);
    }
    mm
}

#[derive(Clone, Copy)]
struct LogStatics {
    cycles: u64,
    epoch: u64,
    epoch_block_count: u64,
    tx_count: u64,
    timestamp: u64,
    block_size: u64,
}

struct EpochStatics {
    block_count: u64,
    cycles: f64,
    tx_count: f64,
    block_size: f64,
}


fn height_to_epoch(m: &BTreeMap<u64, LogStatics>) -> BTreeMap<u64, EpochStatics> {
    let mut ret = BTreeMap::<u64, EpochStatics>::new();
    for (key, value) in &m.values().group_by(|v| v.epoch) {
        let mut epoch_block_count = Rc::new(0_u64);
        let vs: Vec<(u64, u64)> = value.map(|v| {
            // set epoch_block_count to v.epoch_block_count
            epoch_block_count = Rc::new(v.epoch_block_count);
            (v.cycles, v.tx_count)
        }).collect();
        let average_cycles = vs.iter().map(|v| v.0).sum::<u64>() as f64 / vs.len() as f64;
        let average_tx_count = vs.iter().map(|v| v.1).sum::<u64>() as f64 / vs.len() as f64;
        let average_block_size = vs.iter().map(|v| v.1).sum::<u64>() as f64 / vs.len() as f64;
        ret.insert(key, EpochStatics {
            block_count: *epoch_block_count,
            cycles: average_cycles,
            tx_count: average_tx_count,
            block_size: average_block_size,
        });
    }
    ret
}

fn parse_info_level_log(log_file: &str) -> BTreeMap<u64, LogStatics> {
    let file = File::open(log_file).expect("file not found");

    // read file line by line
    let reader = std::io::BufReader::new(file);

    let mut log_statics = BTreeMap::<u64, LogStatics>::new();

    for text in reader.lines() {
        let text = text.unwrap();
        if text.contains("[block_verifier]") {
            let start_i = text.find("[block_verifier]").unwrap();
            let (block_number, _hash, _, _, cycles, _max_cycles) = scan_fmt!(&text[start_i..] ,  // input string
                            "[block_verifier] block number: {}, hash: Byte32({}), size:{}/{}, cycles: {}/{}",     // format
           u64,
                String,
                u64,
                u64,
                u64,
                u64
            ).unwrap();
            log_statics.entry(block_number).and_modify(|v| {
                v.cycles = cycles
            }).or_insert(LogStatics {
                cycles,
                epoch: 0,
                epoch_block_count: 0,
                tx_count: 0,
                timestamp: 0,
                block_size: 0,
            });
        } else if text.contains(" INFO ckb_chain::chain  block: ")
            && text.contains("hash: 0x")
            && text.contains(", epoch: ")
            && text.contains(", total_diff: 0x")
            && text.contains(", txs: ")
        {
            let start_i = text.find("INFO ckb_chain::chain  block: ").unwrap();
            let (block_number, _hash_, epoch, _block_idx_in_epoch, block_count_in_epoch, _total_difficulty, txs_count) = scan_fmt!(&text[start_i..] ,  // input string
                            "INFO ckb_chain::chain  block: {}, hash: {}, epoch: {}({}/{}), total_diff: {x}, txs: {}",     // format
           u64,
                String,
                u64,
                u64,
                u64,
               String,
                u64
            ).unwrap();


            let time_str = &text[7..30].to_string();
            // parse time from string
            let time = chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap_or_else(|v| {
                    panic!("parse time error: {}, text: {}", v, text);
                });

            // find first , position in text
            log_statics.entry(block_number).and_modify(|v| {
                v.timestamp = time.timestamp() as u64;
                v.epoch = epoch;
                v.tx_count = txs_count;
                v.epoch_block_count = block_count_in_epoch;
            }).or_insert(LogStatics {
                cycles: 0,
                epoch,
                epoch_block_count: block_count_in_epoch,
                tx_count: txs_count,
                timestamp: time.timestamp() as u64,
                block_size: 0,
            });
        }
    }
    log_statics
}


fn draw_u64(filename: &str, chart_name: &str, x_description: &str, y_description: &str, data0: Vec<(u64, u64)>, data1: Option<Vec<(u64, u64)>>) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1280, 720)).into_drawing_area();

    let mut y_max = data0.iter().map(|(_, v)| *v).max().unwrap();
    if data1.is_some() {
        let y_max1 = data1.as_ref().unwrap().iter().map(|(_, v)| *v).max().unwrap();
        if y_max1 > y_max {
            y_max = y_max1;
        }
    }

    root.fill(&WHITE)?;
    root.margin(10, 10, 20, 10);

    let mut chart = ChartBuilder::on(&root)
        .caption(chart_name, ("sans-serif", 50).into_font())
        .x_label_area_size(50)
        .y_label_area_size(100)
        .build_cartesian_2d(
            data0.first().unwrap().0..data0.last().unwrap().0,
            0..y_max,
        )?;
    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .draw()?;

    chart.configure_mesh()
        .x_desc(x_description)
        .y_desc(y_description)
        .draw().unwrap();
    chart.draw_series(PointSeries::of_element(
        data0.iter().map(|v| (v.0, v.1)),
        1,
        &RED,
        &|c, s, st| {
            EmptyElement::at(c) + Circle::new((0, 0), s, st.filled())
        },
    ))?;

    if data1.is_some() {
        chart.draw_series(PointSeries::of_element(
            data1.unwrap().iter().map(|v| (v.0, v.1)),
            1,
            &BLUE,
            &|c, s, st| {
                EmptyElement::at(c) + Circle::new((0, 0), s, st.filled())
            },
        ))?;
    }

    Ok(())
}

fn draw_f64(
    filename: &str,
    chart_name: &str,
    x_description: &str,
    y_description: &str,
    data0: Vec<(f64, f64)>,
    data1: Option<Vec<(f64, f64)>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1280, 720)).into_drawing_area();

    let mut y_max = data0.iter().map(|(_, v)| *v).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    if data1.is_some() {
        let y_max1 = data1.as_ref().unwrap().iter().map(|(_, v)| *v).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        if y_max1 > y_max {
            y_max = y_max1;
        }
    }

    root.fill(&WHITE)?;
    root.margin(10, 10, 20, 10);

    let mut chart = ChartBuilder::on(&root)
        .caption(chart_name, ("sans-serif", 50).into_font())
        .x_label_area_size(50)
        .y_label_area_size(100)
        .build_cartesian_2d(
            data0.first().unwrap().0..data0.last().unwrap().0,
            0.0..y_max,
        )?;
    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .draw()?;

    chart.configure_mesh()
        .x_desc(x_description)
        .y_desc(y_description)
        .draw().unwrap();
    chart.draw_series(PointSeries::of_element(
        data0.iter().map(|v| (v.0, v.1)),
        1,
        &RED,
        &|c, s, st| {
            EmptyElement::at(c) + Circle::new((0, 0), s, st.filled())
        },
    ))?;

    if data1.is_some() {
        chart.draw_series(PointSeries::of_element(
            data1.unwrap().iter().map(|v| (v.0, v.1)),
            1,
            &BLUE,
            &|c, s, st| {
                EmptyElement::at(c) + Circle::new((0, 0), s, st.filled())
            },
        ))?;
    }

    Ok(())
}
