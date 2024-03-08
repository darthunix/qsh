use rust_decimal::prelude::*;
use std::fmt::{Display, Formatter};

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use clap::Parser;
use qsh_rs::{
    header, inflate,
    types::{Deal, Side, Stream},
    DealReader, QshRead,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file to read
    #[arg(short, long)]
    file: String,
}

struct Operation(Side);

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Side::Buy => write!(f, "Buy"),
            Side::Sell => write!(f, "Sell"),
            Side::UNKNOWN => write!(f, "Unknown"),
        }
    }
}

impl From<Side> for Operation {
    fn from(side: Side) -> Self {
        Self(side)
    }
}

/// Amount of milliseconds since 0 AD
struct CeTime {
    ms: i64,
}

impl CeTime {
    fn new(ms: i64) -> Self {
        Self { ms }
    }
}

impl From<CeTime> for DateTime<Utc> {
    fn from(ce_time: CeTime) -> Self {
        let nd = NaiveDate::from_num_days_from_ce_opt(1).expect("Failed to create NaiveDate");
        let nd = nd
            .and_hms_opt(0, 0, 0)
            .expect("Failed to create NaiveDateTime");
        let nd = nd + Duration::try_milliseconds(ce_time.ms).expect("Failed to create Duration");
        Utc.from_utc_datetime(&nd)
    }
}

struct DealRow {
    frame_time_delta: i64,
    exchange: DateTime<Utc>,
    deal_id: i64,
    side: Operation,
    price: Decimal,
    amount: i64,
    oi: i64,
}

impl DealRow {
    fn new(deal: &Deal) -> Self {
        let exchange = CeTime::new(deal.timestamp);
        // We use decimal to avoid floating point inaccuracy.
        let mut price = Decimal::from_i64(deal.price).expect("Failed to convert price to Decimal");
        price.set_scale(2).expect("Failed to set scale for price");
        price.normalize_assign();

        Self {
            frame_time_delta: deal.frame_time_delta,
            exchange: exchange.into(),
            deal_id: deal.deal_id,
            side: deal.side.into(),
            price,
            amount: deal.amount,
            oi: deal.oi,
        }
    }

    fn as_csv(&self, delim: &str) -> String {
        format!(
            "{}{delim}{}{delim}{}{delim}{}{delim}{}{delim}{}{delim}{}",
            self.frame_time_delta,
            self.exchange.format("%d.%m.%Y %H:%M:%S%.3f"),
            self.deal_id,
            self.side,
            self.price,
            self.amount,
            self.oi,
            delim = delim
        )
    }
}

fn main() {
    let args = Args::parse();
    let mut parser = inflate(args.file.into()).expect("Failed to open file.");
    let header = header(&mut parser).expect("Failed to read qsh file header.");
    assert!(
        !(Stream::DEALS != header.stream),
        "Currently only deals stream is supported."
    );
    println!("FrameTimeDelta;ExchTime;DealId;Type;Price;Volume;OI");
    let iter = parser.into_iter::<DealReader>();
    for deal in iter {
        println!("{}", DealRow::new(&deal).as_csv(";"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{self, BufRead};

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
    use std::path::Path;

    struct DealRowWithoutFtd {
        exchange: DateTime<Utc>,
        deal_id: i64,
        side: Operation,
        price: Decimal,
        amount: i64,
        oi: i64,
    }

    impl From<DealRow> for DealRowWithoutFtd {
        fn from(deal_row: DealRow) -> Self {
            Self {
                exchange: deal_row.exchange,
                deal_id: deal_row.deal_id,
                side: deal_row.side,
                price: deal_row.price,
                amount: deal_row.amount,
                oi: deal_row.oi,
            }
        }
    }

    impl DealRowWithoutFtd {
        fn as_csv(&self, delim: &str) -> String {
            format!(
                "{}{delim}{}{delim}{}{delim}{}{delim}{}{delim}{}",
                self.exchange.format("%d.%m.%Y %H:%M:%S%.3f"),
                self.deal_id,
                self.side,
                self.price,
                self.amount,
                self.oi,
                delim = delim
            )
        }
    }

    #[test]
    fn test_deal_row() {
        let mut parser =
            inflate("data/SBER.2024-02-20.Deals.qsh".into()).expect("Failed to open file.");
        let header = header(&mut parser).expect("Failed to read qsh file header.");
        if Stream::DEALS != header.stream {
            panic!("Currently only deals stream is supported.");
        };
        let mut qsh_iter = parser.into_iter::<DealReader>();

        if let Ok(lines) = read_lines("data/SBER.2024-02-20.Deals.txt") {
            for line in lines.flatten() {
                let expected: String = line.split(";").skip(1).collect::<Vec<_>>().join(";");
                let actual: String =
                    DealRowWithoutFtd::from(DealRow::new(&qsh_iter.next().unwrap())).as_csv(";");
                assert_eq!(expected, actual);
            }
        }
    }
}
