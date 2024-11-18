use anyhow::Result;
use polars::prelude::*;
use std::io::Cursor;
use reqwest;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";
    let data = reqwest::get(url).await?.text().await?;

    // 使用 polars 直接处理 csv 文件
    let df = CsvReader::new(Cursor::new(data))
        .finish()?;

    let filtered = df.lazy()
        .filter(col("new_deaths").gt_eq(lit(500)))
        .select(&[
            col("location").alias("name"),
            col("total_cases"),
            col("new_cases"),
            col("total_deaths"),
            col("new_deaths"),
        ])
        .sort(
            vec!["new_cases"],
            SortMultipleOptions {
                descending: vec![true],
                ..Default::default()
            },
        )
        .slice(5, 6);

    let df = filtered.collect()?;
    println!("{:?}", df);

    Ok(())
}
