use anyhow::Result;
use polars::prelude::*;
use std::io::Cursor;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";
    // let data = reqwest::get(url).await?.text().await;

    // 使用 polars
    let q = LazyCsvReader::new("./covid.csv")
        .with_infer_schema_length(Some(1000))
        .has_header(true)
        .finish()?
        .filter(col("total_deaths").gt(lit(5000)))
        .select([
            col("location"),
            col("total_cases"),
            col("new_cases"),
            col("total_deaths"),
            col("new_deaths"),
        ]);

    println!("{:?}", q.collect()?);
    Ok(())
}
