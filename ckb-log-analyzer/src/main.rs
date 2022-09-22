use chrono::NaiveDateTime;
use plotters::drawing::DrawingAreaErrorKind::LayoutError;
use plotters::prelude::*;
use std::fs::File;
use std::io::BufRead;
use std::ops::Deref;

fn main() {
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
            let time_str = &text[7..30].to_string();
            // parse time from string
            let time = chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap_or_else(|v| {
                    panic!("parse time error: {}, text: {}", v, text);
                });

            // find first , position in text
            let comma_pos = text.find(',').unwrap();
            let height_str = &text[96..comma_pos];
            let height = height_str.parse::<u64>().unwrap();

            if height % 100 != 0 {
                continue;
            }

            // if previous_time.is_none() {
            //     previous_time = Some(time);
            //     continue;
            // }
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
            // previous_time = Some(time);

            points.push((time.timestamp() as u64, height));
        }
    }

    draw(&points).unwrap()
}

fn draw(points: &[(u64, u64)]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("img/4.png", (1280, 720)).into_drawing_area();

    root.fill(&WHITE)?;
    root.margin(10, 10, 10, 10);

    // get second item of last of points

    let mut chart = ChartBuilder::on(&root)
        .caption("catption", ("sans-serif", 50).into_font())
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(
            points.first().unwrap().0..points.last().unwrap().0,
            0_u64..points.last().unwrap().1,
        )?;
    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .draw()?;

    chart.configure_mesh().draw().unwrap();
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