use anyhow::{Ok, Result};
use queryer::query;

pub fn example_sql1() -> String {
    let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";
    let sql = format!(
        "SELECT location name, total_cases, new_cases, total_deaths, new_deaths \
        FROM {} where new_deaths >= 100 ORDER BY new_cases DESC LIMIT 6",
        url
    );
    sql
}

pub fn example_sql2() -> String {
    let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";

    // 使用 SQL 从 url 中获取数据
    let sql = format!(
        "select location name, total_cases, new_cases, total_deaths, new_deaths \
        FROM {} where new_deaths >= 500 ORDER BY new_cases DESC",
        url
    );
    sql
}

#[tokio::main]
async fn main() -> Result<()> {
    let df1 = query(example_sql1()).await?;
    println!("{:?}", df1);

    let df2 = query(example_sql2()).await?;
    println!("{:?}", df2);

    Ok(())
}
