use std::collections::{BTreeMap, HashMap};
use plotters::prelude::*;
use std::fs::File;
use std::io::BufRead;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread;
use ckb_db::RocksDB;
use ckb_db_schema::{COLUMNS};
use ckb_store::{ChainStore, ChainDB};
use scan_fmt::scan_fmt;


struct HeightStatus {
    time: u64,
    epoch: u64,
    block_size: u64,
}

fn main() {
    draw_height_block_size()
    // draw_time_cost()
}

fn draw_height_block_size() {
    let mm = Arc::new(Mutex::new(BTreeMap::<u64, HeightStatus>::new()));

    let m1 = mm.clone();
    let h1 = thread::spawn(move || {
        let timestamp_height = File::open("timestamp_height_txs-count.log").expect("file not found");
        let time_reader = std::io::BufReader::new(timestamp_height);
        for text in time_reader.lines() {
            let text = text.unwrap();
            let (timestamp, height, txs_count) = scan_fmt!(&text, "{},{},{}", u64, u64, u64).unwrap();

            // insert to mm
            let status = HeightStatus {
                time: timestamp,
                epoch: 0,
                block_size: 0,
            };
            // insert to mm
            m1.lock().unwrap().entry(height).and_modify(|v| v.time = timestamp).or_insert(status);
        }
    });

    let m2 = mm.clone();
    let h2 = thread::spawn(move || {
        let epoch_size_file = File::open("epoch_number_block_size.log").expect("file not found");
        let epoch_reader = std::io::BufReader::new(epoch_size_file);
        for text in epoch_reader.lines() {
            let text = text.unwrap();
            let (epoch, height, block_size) = scan_fmt!(&text, "{},{},{}", u64, u64, u64).unwrap();
            m2.lock().unwrap().entry(height).and_modify(|e| {
                e.epoch = epoch;
                e.block_size = block_size
            }).or_insert(HeightStatus { time: 0, epoch, block_size });
        }
    });
    h1.join().unwrap();
    h2.join().unwrap();

    //
    // {
    //     let mut ymax = 0_u64;
    //     for (height, status) in mm.lock().unwrap().iter() {
    //         if status.block_size > ymax {
    //             ymax = status.block_size;
    //         }
    //     }
    //     let time_block_size: Vec<(u64, u64)> =
    //         mm.lock().unwrap().iter().map(|(height, status)| (*height, status.block_size)).collect();
    //     draw("img/height_block_size.png", &time_block_size, "CKB Sync Status: (height, block_size)", ymax,
    //          "height",
    //          "block_size",
    //     ).unwrap();
    // }
    {
        let mut epoch_total_size = BTreeMap::<u64, (u64, u64)>::new();
        mm.lock().unwrap().iter().for_each(|(h, status)| {
            epoch_total_size.entry(status.epoch).and_modify(|v| {
                v.0 = v.0 + status.block_size;
                v.1 = v.1 + 1;
            }
            ).or_insert((0, 0));
        });
        let results: Vec<(u64, u64)> = epoch_total_size.iter().map(|v| {
            if v.1.1 == 0 {
                return (*v.0, 0);
            }
            (*v.0, v.1.0 / v.1.1)
        }).collect();

        let mut y_max = 0_u64;
        for (_, v) in &results {
            if *v > y_max {
                y_max = *v;
            }
        }


        draw("img/epoch_average_block_size.png", &results, "CKB Sync Status: (epoch, average block_size)", y_max, "epoch", "average_block_size").unwrap();
    }
}


fn export_block_size() {
    let ckb_mainnet_dir = "/home/exec/Projects/github.com/nervosnetwork/ckb-run-log/ckb-main/data/db";
    let db = RocksDB::open_in(&ckb_mainnet_dir, COLUMNS);
    let store = ChainDB::new(db, Default::default());

    let txn = store.begin_transaction();
    let latest_ext = store.get_current_epoch_ext().unwrap().number();
    for number in 0..8162480 {
        let block_hash = store.get_block_hash(number).unwrap();
        let epoch_number = store.get_block_epoch(&block_hash).unwrap().number();
        let packed_block_size = store.get_packed_block(&block_hash).unwrap().total_size();
        println!("{},{},{}", epoch_number, number, packed_block_size)
    }
}

fn draw_time_cost() {
    // read first arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file>", args[0]);
        return;
    }

    // open file readonly

    // get first arguemnt
    let filename = &args[1];

    let file = File::open(filename).expect("file not found");

    // read file line by line
    let reader = std::io::BufReader::new(file);

    // 2022-09-21 09:57:29.339 +00:00 ChainService INFO ckb_chain::chain  block: 3366025, hash: 0x678f7e40679b70034efee7f7171b638303e439d9ea8bdb8d08dde92aba9075c1, epoch: 2273(922/931), total_diff: 0x32ae03e89cc3999010f5, txs: 1

    let mut previous_time: Option<chrono::NaiveDateTime> = None;
    let mut max_time_diff: chrono::Duration = chrono::Duration::seconds(0);
    let mut time_diffs: Vec<u64> = vec![0; 100];

    let mut points = Vec::new();
    for text in reader.lines() {
        let text = text.unwrap();


        if text.contains(" INFO ckb_chain::chain  block: ")
            && text.contains("hash: 0x")
            && text.contains(", epoch: ")
            && text.contains(", total_diff: 0x")
            && text.contains(", txs: ")
        {
            let start_i = text.find("INFO ckb_chain::chain  block: ").unwrap();
            let (block_number, hash, epoch, _, _, total_difficulty, txs_count) = scan_fmt!(&text[start_i..] ,  // input string
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


            if block_number % 100 != 0 {
                continue;
            }

            if previous_time.is_none() {
                previous_time = Some(time);
                continue;
            }
            //
            // let time_diff = time - previous_time.unwrap();
            // if time_diff > max_time_diff {
            //     max_time_diff = time_diff;
            //     // println!(
            //     //     "max time diff: {}s, text: {}",
            //     //     time_diff.num_seconds(),
            //     //     text
            //     // );
            // }
            // time_diffs[time_diff.num_seconds() as usize] += 1;
            //
            previous_time = Some(time);

            points.push((time.timestamp() as u64, block_number));
        }
    }

    draw("img/time_height.png", &points, "CKB Sync Status:(timestamp, height)", points.last().unwrap().1, "timestamp", "height").unwrap()
}

fn draw(filename: &str, points: &[(u64, u64)], chart_name: &str, y_max: u64, x_description: &str, y_description: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1280, 720)).into_drawing_area();

    root.fill(&WHITE)?;
    root.margin(10, 10, 10, 10);

    // get second item of last of points

    let mut chart = ChartBuilder::on(&root)
        .caption(chart_name, ("sans-serif", 50).into_font())
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(
            points.first().unwrap().0..points.last().unwrap().0,
            0_u64..y_max,
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
        points.iter().map(|v| (v.0, v.1)),
        1,
        &RED,
        &|c, s, st| {
            EmptyElement::at(c) + Circle::new((0, 0), s, st.filled())
            // + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
        },
    ))?;
    Ok(())
}
