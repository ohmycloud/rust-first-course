# 构建 Python 编译环境

使用 `uv` 创建 virtual env, 然后使用 `maturin development` 构建:

```bash
uv venv
.venv\Scripts\activate
uv pip install pip
uvx maturin develop --verbose
python
```

构建完成后, 进入 Python 的 REPL 进行测试:

```
>>> import queryer_py
>>> sql = queryer_py.example_sql()
>>> print(queryer_py.query(sql))
shape: (6, 5)
┌───────────────────────┬──────────────┬───────────┬──────────────┬────────────┐
│ name                  ┆ total_cases  ┆ new_cases ┆ total_deaths ┆ new_deaths │
│ ---                   ┆ ---          ┆ ---       ┆ ---          ┆ ---        │
│ str                   ┆ f64          ┆ f64       ┆ f64          ┆ f64        │
╞═══════════════════════╪══════════════╪═══════════╪══════════════╪════════════╡
│ United States         ┆ 1.03436829e8 ┆ null      ┆ 1.193165e6   ┆ 619.0      │
│ North America         ┆ 1.24492666e8 ┆ 454.0     ┆ 1.671178e6   ┆ 619.0      │
│ European Union (27)   ┆ 1.85822587e8 ┆ 25642.0   ┆ 1.262988e6   ┆ 150.0      │
│ High-income countries ┆ 4.29044049e8 ┆ 32293.0   ┆ 2.997359e6   ┆ 786.0      │
│ Europe                ┆ 2.52916868e8 ┆ 39047.0   ┆ 2.102483e6   ┆ 162.0      │
│ World                 ┆ 7.75866783e8 ┆ 47169.0   ┆ 7.057132e6   ┆ 815.0      │
└───────────────────────┴──────────────┴───────────┴──────────────┴────────────┘
```