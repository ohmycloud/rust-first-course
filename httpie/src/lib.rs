use std::str::FromStr;
use clap::{Args, Parser, Subcommand};
use reqwest::{Url};
use anyhow::{anyhow, Result};

fn parse_url(s: &str) -> Result<String> {
    // 检查 URl 是否合法
    let _url: Url = s.parse()?;
    Ok(s.into())
}

#[derive(Args, Debug)]
pub struct Get {
    /// HTTP 请求的 URL
    #[arg(value_parser = parse_url)]
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct KvPair {
    pub k: String,
    pub v: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // 使用 = 进行 split, 得到一个迭代器
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self {
            k: (split.next().ok_or_else(err)?).into(),
            v: (split.next().ok_or_else(err)?).into(),
        })
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

#[derive(Args, Debug)]
pub struct Post {
    /// HTTP 请求的 URL
    #[arg(value_parser = parse_url)]
    pub url: String,
    /// HTTP 请求的 Body
    #[arg(value_parser = parse_kv_pair)]
    pub body: Vec<KvPair>,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    Get(Get),
    Post(Post),
}

#[derive(Parser, Debug)]
#[command(author = "ohmycloud", version = "0.1.0", about = "httpie", long_about = "Rust httpie")]
pub struct Cli {
    #[command(subcommand)]
    pub command: SubCommand
}

pub fn get_args() -> Cli {
    let args = Cli::parse();
    args
}