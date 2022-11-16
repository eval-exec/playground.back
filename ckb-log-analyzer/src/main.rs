extern crate core;

use chrono::Duration;
use crossbeam_queue::SegQueue;
use itertools::Itertools;
use plotters::prelude::*;
use rayon::prelude::*;
use scan_fmt::scan_fmt;
use std::cell::RefCell;
use std::cmp::{max, max_by, min};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufRead;
use std::ops::Index;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use clap::{Args, Command, Parser, Subcommand};
use log::info;
use serde::{Deserialize, Serialize};
#[derive(clap::Parser)]
#[command(about, long_about = None)]
struct App {
    #[command(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    Draw(DrawArg),
    Analyse(AnalyseArg),
}

#[derive(Parser)]
struct AnalyseArg {
    #[clap(long, action)]
    logs_path: PathBuf,
    #[arg(short, long, default_value_t = 2000)]
    every_height: u64,
}

#[derive(Parser)]
struct DrawArg {
    #[clap(long, action)]
    logs_path: Vec<PathBuf>,
    #[clap(long, action)]
    labels: Vec<String>,
    #[clap(long, action)]
    outdir: PathBuf,
}

// fn save_context(c: &Context) {
//     // serialize c to file
//     let file = File::create("memo.cbor").unwrap();
//     let s = SContext::from(c);
//     serde_cbor::to_writer(file, &s).unwrap();
// }
//
// fn load_context(now: &Instant, path_bufs: Vec<PathBuf>) -> Context {
//     // deserialize from file
//     match File::open("memo.cbor") {
//         Ok(file) => {
//             info!("loading context from file");
//             let s: SContext = serde_cbor::from_reader(file).unwrap();
//             info!("loaded context from file");
//             Context::from(&s)
//         }
//         Err(_) => {
//             info!("building context");
//             let c = build_context(now, path_bufs);
//             info!("saving context to file");
//             save_context(&c);
//             info!("context saved to cbor file");
//             c
//         }
//     }
// }
//
fn parse_log_entry(filename: PathBuf) -> BTreeMap<u64, LogStatics> {
    let (_verifier, blocks) = parse_log_parallel(filename);
    let mut tree = BTreeMap::new();
    blocks.into_iter().for_each(|v| {
        tree.insert(
            v.block_number,
            LogStatics {
                cycles: 0,
                epoch: v.epoch,
                epoch_block_count: 0,
                tx_count: 0,
                timestamp: v.timestamp,
                block_size: 0,
            },
        );
    });
    tree
}

fn parse_log_parallel(
    filename: PathBuf,
) -> (SegQueue<EntryBlockVerifier>, SegQueue<EntryBlockProcess>) {
    let file = File::open(filename).expect("file not found");

    // read file line by line
    let reader = std::io::BufReader::new(file);

    let entry_block_verifer = EntryBlockVerifier::default();

    let entry_block_process = EntryBlockProcess::default();

    let mut verifier = SegQueue::new();
    let mut blocks = SegQueue::new();
    reader
        .lines()
        .filter_map(Result::ok)
        .par_bridge()
        .for_each(|line| {
            // if let Some(v) = entry_block_verifer
            //     .parse_line(&line) { verifier.push(v) }
            if let Some(v) = entry_block_process.parse_line(&line) {
                // if v.block_number > 6_100_000 {
                //     return;
                // }
                blocks.push(v)
            }
        });

    (verifier, blocks)
}

fn build_context(now: &Instant, file_paths: Vec<PathBuf>, labels: Vec<String>) -> Context {
    let mut join_handles = Vec::new();
    for file_path in file_paths {
        join_handles.push(thread::spawn(|| parse_log_entry(file_path)));
    }
    // let e0 = thread::spawn(|| {
    //     let block_size_mm = export_block_size();
    //     block_size_mm
    // });
    let mut datas = Vec::new();
    for jh in join_handles {
        datas.push(jh.join().unwrap());
        info!("parse ckb log {:?}", now.elapsed());
    }

    // let block_size_mm = e0.join().unwrap();
    // info!("export block size {:?}", now.elapsed());
    //
    // for (height, block_size) in block_size_mm {
    //     mm0.get_mut()
    //         .entry(height)
    //         .and_modify(|v| v.block_size = block_size);
    //     mm1.get_mut()
    //         .entry(height)
    //         .and_modify(|v| v.block_size = block_size);
    // }
    //
    // info!("fill block size {:?}", now.elapsed());
    //
    // let _mm0 = mm0.clone();
    // let j0 = thread::spawn(|| {
    //     let epoch_mm0 = height_to_epoch(&_mm0.into_inner());
    //     epoch_mm0
    // });
    //
    // let _mm1 = mm1.clone();
    // let j1 = thread::spawn(|| {
    //     let epoch_mm1 = height_to_epoch(&_mm1.into_inner());
    //     epoch_mm1
    // });
    // let epoch_mm0 = Arc::new(j0.join().unwrap());
    // info!("epoch mm0 {:?}", now.elapsed());
    // let epoch_mm1 = Arc::new(j1.join().unwrap());
    // info!("epoch mm1 {:?}", now.elapsed());

    let c = Context {
        epoch_mm0: Arc::new(BTreeMap::new()),
        mm: Arc::new(datas),
        labels: labels,
        outdir: Default::default(),
    };
    c
}

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    info!("start");
    info!(
        "current dir: {}",
        std::env::current_dir().unwrap().to_string_lossy()
    );

    let app = App::parse();
    match app.action {
        Action::Draw(draw_arg) => {
            if draw_arg.logs_path.is_empty() {
                panic!("log paths is empty");
            } else {
                info!("logs-path count: {}", &draw_arg.logs_path.len());
                for path in &draw_arg.logs_path {
                    info!("log-path: {}", path.to_string_lossy());
                }
            }

            let now = Instant::now();
            let mut context = build_context(&now, draw_arg.logs_path, draw_arg.labels);
            context.outdir = draw_arg.outdir;
            std::fs::create_dir_all(&context.outdir).unwrap();
            let ac = Arc::new(context);
            let now = Arc::new(now);

            let mut join_handles = vec![];

            let ac0 = ac.clone();
            let now0 = now.clone();
            join_handles.push(thread::spawn(move || {
                ac0.draw_time_cost();
                info!("draw time cost {:?}", now0.elapsed());
            }));

            // let ac1 = ac.clone();
            // let now1 = now.clone();
            // join_handles.push(thread::spawn(move || {
            //     ac1.draw_height_block_size();
            //     info!("draw height block size {:?}", now1.elapsed());
            // }));
            //
            // let ac2 = ac.clone();
            // let now2 = now.clone();
            // join_handles.push(thread::spawn(move || {
            //     ac2.draw_epoch_average_block_size();
            //     info!("draw epoch average block size {:?}", now2.elapsed());
            // }));
            //
            // let ac3 = ac.clone();
            // let now3 = now.clone();
            // join_handles.push(thread::spawn(move || {
            //     ac3.draw_height_cycles();
            //     info!("draw height cycles {:?}", now3.elapsed());
            // }));
            //
            // let ac4 = ac.clone();
            // let now4 = now.clone();
            // join_handles.push(thread::spawn(move || {
            //     ac4.draw_epoch_cycles();
            //     info!("draw epoch cycles {:?}", now4.elapsed());
            // }));
            //
            // let ac5 = ac.clone();
            // let now5 = now.clone();
            // join_handles.push(thread::spawn(move || {
            //     ac5.draw_height_txs_count();
            //     info!("draw height txs count {:?}", now5.elapsed());
            // }));
            //
            // let ac6 = ac.clone();
            // let now6 = now.clone();
            // join_handles.push(thread::spawn(move || {
            //     ac6.draw_epoch_average_txs_count();
            //     info!("draw epoch average txs count {:?}", now6.elapsed());
            // }));

            // join join_handles
            for jh in join_handles {
                jh.join().unwrap();
            }
        }
        Action::Analyse(analyse_arg) => {
            let now = Instant::now();
            let context =
                build_context(&now, vec![analyse_arg.logs_path], vec!["label".to_string()]);
            let mm = context.mm.get(0).unwrap();
            let mut start_time = None;
            let mut results = Vec::new();
            mm.iter().for_each(|(height, v)| {
                if start_time.is_none() {
                    start_time = Some(v.timestamp);
                    results.push((*height, v.timestamp))
                }
                if height % 100000 == 0 {
                    results.push((*height, v.timestamp))
                }
            });

            let (_, mut previous_time) = results.first().unwrap();
            results
                .iter()
                .skip(1)
                .skip_while(|(height, timestamp)| height % analyse_arg.every_height != 0)
                .for_each(|(height, timestamp)| {
                    info!("height, {} {}", height, timestamp - previous_time);
                    previous_time = *timestamp;
                })
        }
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// struct SContext {
//     epoch_mm0: BTreeMap<u64, EpochStatics>,
//     mm0: BTreeMap<u64, LogStatics>,
//
//     epoch_mm1: Option<BTreeMap<u64, EpochStatics>>,
//     mm1: Option<BTreeMap<u64, LogStatics>>,
// }
//
// impl From<&Context> for SContext {
//     fn from(c: &Context) -> Self {
//         SContext {
//             epoch_mm0: (*c.epoch_mm0).clone(),
//             mm0: (*c.mm0).clone(),
//             epoch_mm1: c.epoch_mm1.clone().map(|m| (*m).clone()),
//             mm1: c.mm1.clone().map(|m| (*m).clone()),
//         }
//     }
// }
//
// impl From<&SContext> for Context {
//     fn from(s: &SContext) -> Self {
//         Context {
//             epoch_mm0: Arc::new(s.epoch_mm0.clone()),
//             mm0: Arc::new(s.mm0.clone()),
//             mm1: s.mm1.as_ref().map(|mm| Arc::new(mm.clone())),
//         }
//     }
// }

struct Context {
    epoch_mm0: Arc<BTreeMap<u64, EpochStatics>>,
    mm: Arc<Vec<BTreeMap<u64, LogStatics>>>,
    labels: Vec<String>,
    outdir: PathBuf,
}

impl Context {
    // fn draw_epoch_average_block_size(&self) {
    //     let points: Vec<(f64, f64)> = self
    //         .epoch_mm0
    //         .iter()
    //         .map(|(epoch, status)| {
    //             (
    //                 *epoch as f64,
    //                 status.block_size as f64 / status.block_count as f64,
    //             )
    //         })
    //         .collect();
    //     let points1: Option<Vec<(f64, f64)>> = self.epoch_mm1.as_ref().map(|m| {
    //         m.iter()
    //             .map(|(epoch, status)| {
    //                 (
    //                     *epoch as f64,
    //                     status.block_size as f64 / status.block_count as f64,
    //                 )
    //             })
    //             .collect()
    //     });
    //     draw_f64(
    //         "img/epoch_average_block_size.png",
    //         "CKB Sync Status: (epoch, average_block_size)",
    //         "epoch",
    //         "average_block_size",
    //         points,
    //         points1,
    //     )
    //     .unwrap();
    // }
    //
    // fn draw_height_block_size(&self) {
    //     let points: Vec<(u64, u64)> = self
    //         .mm0
    //         .iter()
    //         .map(|(height, status)| (*height, status.block_size))
    //         .collect();
    //     let points1: Option<Vec<(u64, u64)>> = self.mm1.as_ref().map(|m| {
    //         m.iter()
    //             .map(|(height, status)| (*height, status.block_size))
    //             .collect()
    //     });
    //
    //     draw_u64(
    //         "img/epoch_average_block_size.png",
    //         "CKB Sync Status: (epoch, average block_size)",
    //         "epoch",
    //         "block_size",
    //         points,
    //         points1,
    //     )
    //     .unwrap();
    // }
    //
    // fn draw_height_txs_count(&self) {
    //     let points: Vec<(u64, u64)> = self
    //         .mm0
    //         .iter()
    //         .map(|(height, status)| (*height, status.tx_count))
    //         .collect();
    //     let points1 = self.mm1.as_ref().map(|m| {
    //         m.iter()
    //             .map(|(height, status)| (*height, status.tx_count))
    //             .collect()
    //     });
    //
    //     draw_u64(
    //         "img/height_txs_count.png",
    //         "CKB Sync Status: (height, txs_count)",
    //         "height",
    //         "txs_count",
    //         points,
    //         points1,
    //     )
    //     .unwrap();
    // }
    //
    // fn draw_epoch_average_txs_count(&self) {
    //     let points: Vec<(f64, f64)> = self
    //         .epoch_mm0
    //         .iter()
    //         .map(|(epoch, statics)| {
    //             (
    //                 *epoch as f64,
    //                 statics.tx_count as f64 / statics.block_count as f64,
    //             )
    //         })
    //         .collect();
    //
    //     let mut points_vec = Vec::new();
    //     points_vec.push(points);
    //     draw_f64(
    //         "img/epoch_average_txs_count.png",
    //         "CKB Sync Status: (epoch, average txs_count)",
    //         "epoch",
    //         "average_txs_count",
    //         points_vec
    //     )
    //     .unwrap();
    // }
    //
    // fn draw_epoch_cycles(&self) {
    //     let points: Vec<(f64, f64)> = self
    //         .epoch_mm0
    //         .iter()
    //         .map(|(k, v)| (*k as f64, v.cycles))
    //         .collect();
    //
    //     draw_f64(
    //         "img/epoch_average_cycles.png",
    //         "CKB Sync Status:(epoch,average_cycles)",
    //         "epoch",
    //         "average cycles",
    //         points,
    //         points1,
    //     )
    //     .unwrap();
    // }

    fn draw_height_cycles(&self) {
        let points_vec = self
            .mm
            .iter()
            .map(|m| {
                let points: Vec<(u64, u64)> = m.iter().map(|(k, v)| (*k, v.cycles)).collect();
                points
            })
            .collect();
        draw_u64(
            "img/height_cycles.png",
            "CKB Sync Status:(height,cycles)",
            "height",
            "cycles",
            &points_vec,
            &self.labels,
        )
        .unwrap();
    }

    fn draw_time_cost(&self) {
        let points_vec = self
            .mm
            .iter()
            .map(|m| {
                let points: Vec<(u64, u64)> = m
                    .iter()
                    .filter(|(k, v)| **k % 100 == 0)
                    .map(|(k, v)| (v.timestamp, *k))
                    .collect();
                let first = points.first().unwrap().0;
                points
                    .iter()
                    .map(|(k, v)| (Duration::seconds((*k - first) as _), *v))
                    .collect()
            })
            .collect();
        let outpath = self.outdir.join("tmp.png");
        draw_duration(
            outpath.to_str().unwrap(),
            "CKB Sync Status:(timestamp, height)",
            "timestamp(s)",
            "height",
            &points_vec,
            &self.labels,
        )
        .unwrap()
    }
}

fn export_block_size() -> BTreeMap<u64, u64> {
    let mut mm = BTreeMap::<u64, u64>::new();
    let file = File::open("epoch_number_block_size.log").unwrap();

    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        scan_fmt!(&line, "{},{},{}", u64, u64, u64)
            .map(|(_epoch, height, block_size)| {
                mm.insert(height, block_size);
            })
            .unwrap();
    }

    // let ckb_mainnet_dir = "/home/exec/Projects/github.com/nervosnetwork/ckb-run-log/ckb-main/data/db";
    // let db = RocksDB::open_in(&ckb_mainnet_dir, COLUMNS);
    // let store = ChainDB::new(db, Default::default());
    //
    // for number in 0..8162480 {
    //     let block_hash = store.get_block_hash(number).unwrap();
    //
    //     let packed_block_size = store.get_packed_block(&block_hash).unwrap().total_size();
    //     mm.insert(number, packed_block_size as u64);
    // }
    mm
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
struct LogStatics {
    cycles: u64,
    epoch: u64,
    epoch_block_count: u64,
    tx_count: u64,
    timestamp: u64,
    block_size: u64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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
        let vs: Vec<(u64, u64, u64)> = value
            .map(|v| {
                // set epoch_block_count to v.epoch_block_count
                epoch_block_count = Rc::new(v.epoch_block_count);
                (v.cycles, v.tx_count, v.block_size)
            })
            .collect();
        let average_cycles = vs.iter().map(|v| v.0).sum::<u64>() as f64 / *epoch_block_count as f64;
        let average_tx_count =
            vs.iter().map(|v| v.1).sum::<u64>() as f64 / *epoch_block_count as f64;
        let average_block_size =
            vs.iter().map(|v| v.2).sum::<u64>() as f64 / *epoch_block_count as f64;
        ret.insert(
            key,
            EpochStatics {
                block_count: *epoch_block_count,
                cycles: average_cycles,
                tx_count: average_tx_count,
                block_size: average_block_size,
            },
        );
    }
    ret
}

#[derive(Default)]
struct EntryBlockVerifier {
    block_number: u64,
    block_hash: String,
    block_cycle: u64,
    block_max_cycle: u64,
}

impl EntryBlockVerifier {
    fn parse_line(&self, line: &str) -> Option<Self> {
        if let Some(start_i) = line.find("[block_verifier]") {
            let (block_number, block_hash, _, _, block_cycle, block_max_cycle) = scan_fmt!(
                &line[start_i..],
                "[block_verifier] block number: {}, hash: Byte32({}), size:{}/{}, cycles: {}/{}",
                u64,
                String,
                u64,
                u64,
                u64,
                u64
            )
            .unwrap();
            return Some(EntryBlockVerifier {
                block_number,
                block_hash,
                block_cycle,
                block_max_cycle,
            });
        }
        None
    }
}

#[derive(Default)]
struct EntryBlockProcess {
    timestamp: u64,
    block_number: u64,
    epoch: u64,
    tx_count: u64,
    epoch_block_count: u64,
}

impl EntryBlockProcess {
    fn parse_line(&self, line: &str) -> Option<Self> {
        let start_i = line.find("INFO ckb_chain::chain  block: ")?;

        let (
            block_number,
            _hash_,
            epoch,
            _block_idx_in_epoch,
            block_count_in_epoch,
            _total_difficulty,
            txs_count,
        ) = scan_fmt!(&line[start_i..] ,  // input string
                            "INFO ckb_chain::chain  block: {}, hash: {}, epoch: {}({}/{}), total_diff: {x}, txs: {}",     // format
           u64,
                String,
                u64,
                u64,
                u64,
               String,
                u64
            )
        .unwrap();

        if block_number % 1000 != 0 {
            return None;
        }

        let time_start = line.find("2022-").unwrap();

        let time_str = &line[time_start..23].to_string();
        // parse time from string
        let time = chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S%.f")
            .unwrap_or_else(|v| {
                panic!("parse time error: {}, text: {}", v, line);
            });

        Some(EntryBlockProcess {
            timestamp: time.timestamp() as u64,
            block_number,
            epoch,
            tx_count: txs_count,
            epoch_block_count: block_count_in_epoch,
        })
    }
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
            let (block_number, _hash, _, _, cycles, _max_cycles) =
                scan_fmt!(&text[start_i..] ,  // input string
                            "[block_verifier] block number: {}, hash: Byte32({}), size:{}/{}, cycles: {}/{}",     // format
           u64,
                String,
                u64,
                u64,
                u64,
                u64
            )
                .unwrap();
            log_statics
                .entry(block_number)
                .and_modify(|v| v.cycles = cycles)
                .or_insert(LogStatics {
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
            let (
                block_number,
                _hash_,
                epoch,
                _block_idx_in_epoch,
                block_count_in_epoch,
                _total_difficulty,
                txs_count,
            ) = scan_fmt!(&text[start_i..] ,  // input string
                            "INFO ckb_chain::chain  block: {}, hash: {}, epoch: {}({}/{}), total_diff: {x}, txs: {}",     // format
           u64,
                String,
                u64,
                u64,
                u64,
               String,
                u64
            )
            .unwrap();

            let time_start = text.find("2022-").unwrap();

            let time_str = &text[time_start..23].to_string();
            // parse time from string
            let time = chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap_or_else(|v| {
                    panic!("parse time error: {}, text: {}", v, text);
                });
            if block_number % 10000 == 0 {
                info!("parse log :block_number: {}", block_number);
            }

            // find first , position in text
            log_statics
                .entry(block_number)
                .and_modify(|v| {
                    v.timestamp = time.timestamp() as u64;
                    v.epoch = epoch;
                    v.tx_count = txs_count;
                    v.epoch_block_count = block_count_in_epoch;
                })
                .or_insert(LogStatics {
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

fn draw_duration(
    filename: &str,
    chart_name: &str,
    x_description: &str,
    y_description: &str,
    datas: &Vec<Vec<(Duration, u64)>>,
    labels: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1920, 1080)).into_drawing_area();
    let (mut x_max, mut x_min, mut y_max, mut y_min) =
        (Duration::min_value(), Duration::max_value(), 0, u64::MAX);
    datas.iter().for_each(|data| {
        let y_max0 = data.iter().map(|(_, v)| *v).max().unwrap();
        let y_min0 = data.iter().map(|(_, v)| *v).min().unwrap();
        y_max = max(y_max, y_max0);
        y_min = min(y_min, y_min0);
        let x_max0 = data.last().unwrap().0;
        let x_min0 = data.first().unwrap().0;
        x_max = max(x_max, x_max0);
        x_min = min(x_min, x_min0);
    });

    root.fill(&WHITE)?;
    root.margin(10_u32, 10_u32, 20_u32, 10_u32);

    let x_range = (x_min..x_max).step(Duration::minutes(10));
    let mut chart = ChartBuilder::on(&root)
        .caption(chart_name, ("sans-serif", 100).into_font())
        .x_label_area_size(50_u32)
        .y_label_area_size(100_u32)
        .build_cartesian_2d(x_range, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_label_formatter(&|x| format!("{:02}H:{:02}M", x.num_hours(), x.num_minutes() % 60))
        .x_desc(x_description)
        .y_desc(y_description)
        .draw()?;

    const COLORS: [RGBColor; 6] = [RED, GREEN, BLUE, BLACK, CYAN, MAGENTA];
    for (i, data) in datas.iter().enumerate() {
        let color = COLORS.get(i).unwrap();
        chart
            .draw_series(PointSeries::of_element(
                data.iter().map(|v| (v.0, v.1)),
                1,
                color,
                &|c, s, st| EmptyElement::at(c) + Circle::new((0, 0), s, st.filled()),
            ))?
            .label(labels.get(i).unwrap())
            .legend(|(x, y)| {
                let color = *color;
                PathElement::new(vec![(x, y), (x + 20, y)], color)
            });
    }

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .draw()?;

    root.present().expect("unable write to file");

    Ok(())
}

fn draw_u64(
    filename: &str,
    chart_name: &str,
    x_description: &str,
    y_description: &str,
    datas: &Vec<Vec<(u64, u64)>>,
    labels: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1920, 1080)).into_drawing_area();
    let (mut x_max, mut x_min, mut y_max, mut y_min) = (0, u64::MAX, 0, u64::MAX);
    datas.iter().for_each(|data| {
        let y_max0 = data.iter().map(|(_, v)| *v).max().unwrap();
        let y_min0 = data.iter().map(|(_, v)| *v).min().unwrap();
        y_max = max(y_max, y_max0);
        y_min = min(y_min, y_min0);
        let x_max0 = data.last().unwrap().0;
        let x_min0 = data.first().unwrap().0;
        x_max = max(x_max, x_max0);
        x_min = min(x_min, x_min0);
    });

    root.fill(&WHITE)?;
    root.margin(10_u32, 10_u32, 20_u32, 10_u32);

    let mut chart = ChartBuilder::on(&root)
        .caption(chart_name, ("sans-serif", 100).into_font())
        .x_label_area_size(50_u32)
        .y_label_area_size(100_u32)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc(x_description)
        .y_desc(y_description)
        .draw()?;

    const COLORS: [RGBColor; 6] = [RED, GREEN, BLUE, BLACK, CYAN, MAGENTA];
    for (i, data) in datas.iter().enumerate() {
        let color = COLORS.get(i).unwrap();
        chart
            .draw_series(PointSeries::of_element(
                data.iter().map(|v| (v.0, v.1)),
                1,
                color,
                &|c, s, st| EmptyElement::at(c) + Circle::new((0, 0), s, st.filled()),
            ))?
            .label(labels.get(i).unwrap())
            .legend(|(x, y)| {
                let color = *color;
                PathElement::new(vec![(x, y), (x + 20, y)], color)
            });
    }

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .draw()?;

    root.present().expect("unable write to file");

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
    let root = BitMapBackend::new(filename, (1920, 1080)).into_drawing_area();

    let mut y_max = data0
        .iter()
        .map(|(_, v)| *v)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    if data1.is_some() {
        let y_max1 = data1
            .as_ref()
            .unwrap()
            .iter()
            .map(|(_, v)| *v)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
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

    chart
        .configure_mesh()
        .x_desc(x_description)
        .y_desc(y_description)
        .draw()
        .unwrap();
    chart.draw_series(PointSeries::of_element(
        data0.iter().map(|v| (v.0, v.1)),
        1,
        &BLUE,
        &|c, s, st| EmptyElement::at(c) + Circle::new((0, 0), s, st.filled()),
    ))?;

    if data1.is_some() {
        chart.draw_series(PointSeries::of_element(
            data1.unwrap().iter().map(|v| (v.0, v.1)),
            1,
            &RED,
            &|c, s, st| EmptyElement::at(c) + Circle::new((0, 0), s, st.filled()),
        ))?;
    }

    Ok(())
}
