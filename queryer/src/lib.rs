use anyhow::{anyhow, Result};
use polars::prelude::*;
use sqlparser::parser::Parser;
use std::{convert::TryInto, ops::{Deref, DerefMut}};
use tracing::info;

mod convert;
mod dialect;
use convert::Sql;
use dialect::TyrDialect;

#[derive(Debug)]
pub struct DataFrame(polars::frame::DataFrame);

impl Deref for DataFrame {
    type Target = polars::frame::DataFrame;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DataFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<polars::frame::DataFrame> for DataFrame {
    fn from(df: polars::frame::DataFrame) -> Self {
        Self(df)
    }
}

/// 从 SQL 查询中得到 DataFrame
pub async fn query<T: AsRef<str>>(sql: T) -> Result<DataFrame> {
    let ast = Parser::parse_sql(&TyrDialect::default(), sql.as_ref())?;

    if ast.len() != 1 {
        return Err(anyhow!("Only support single sql at the moment"));
    }

    let sql = &ast[0];

    // 整个 SQL AST 转换成我们定义的 Sql 结构的细节都在 convert/mod.rs 中
    let Sql {
        source,
        condition,
        selection,
        order_by,
        offset,
        limit,
    } = sql.try_into()?;

    info!("retrieving data from source: {}", source);
    let df = match source {
        source if source.starts_with("http") => {
            let data = reqwest::get(source).await?.text().await?;
            CsvReader::new(std::io::Cursor::new(data))
                .finish()?
        }
        _ => CsvReader::new(std::fs::File::open(source)?)
            .finish()?,
    };

    let mut filtered = match condition {
        Some(expr) => df.lazy().filter(expr),
        None => df.lazy(),
    };

    filtered = filtered.select(&selection);

    // 处理 order by
    for (col, ascending) in order_by {
        filtered = filtered.sort(vec![col], SortMultipleOptions {
            descending: vec![!ascending],
            ..Default::default()
        });
    }

    if let Some(offset) = offset {
        let height = filtered.clone().collect()?.height();
        filtered = filtered.slice(offset as i64, height as u32);
    }

    if let Some(limit) = limit {
        filtered = filtered.limit(limit as u32);
    }

    Ok(filtered.collect()?.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::{fmt, EnvFilter};

    #[tokio::test]
    async fn query_covid() -> Result<()> {
        // 设置日志
        fmt().with_test_writer()
            .with_env_filter(EnvFilter::from_default_env())
            .init();

        let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";

        // 将 URL 里的 csv 文件读入成 polars 的 DataFrame
        let sql = format!(
            "SELECT location name, total_cases, new_cases, total_deaths, new_deaths \
             FROM {} where new_deaths >= 500 ORDER BY new_cases DESC LIMIT 6 OFFSET 5",
            url
        );

        let df = query(sql).await?;
        println!("{:?}", df);
        Ok(())
    }
}
