use anyhow::{anyhow, Result};
use polars::prelude::*;
use sqlparser::ast::{
    BinaryOperator as SqlBinaryOperator, Expr as SqlExpr, Offset as SqlOffset, OrderByExpr,
    SelectItem, SetExpr, Statement, TableFactor, TableWithJoins, Value as SqlValue,
};

// SQL 抽象语法树结构
pub struct Sql<'a> {
    pub(crate) selection: Vec<Expr>,
    pub(crate) condition: Option<Expr>,
    pub(crate) source: &'a str,
    pub(crate) order_by: Vec<(String, bool)>,
    pub(crate) offset: Option<i64>,
    pub(crate) limit: Option<usize>,
}

// 包装结构体
pub struct Expression(pub(crate) Box<SqlExpr>);
pub struct Operation(pub(crate) SqlBinaryOperator);
pub struct Projection<'a>(pub(crate) &'a SelectItem);
pub struct Source<'a>(pub(crate) &'a [TableWithJoins]);
pub struct Order<'a>(pub(crate) &'a OrderByExpr);
pub struct Offset<'a>(pub(crate) &'a SqlOffset);
pub struct Value(pub(crate) SqlValue);

// Convert SqlValue to LiteralValue
impl TryFrom<Value> for LiteralValue {
    type Error = anyhow::Error;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.0 {
            SqlValue::Number(v, _) => {
                if v.contains('.') {
                    Ok(LiteralValue::Float64(v.parse()?))
                } else {
                    Ok(LiteralValue::Int64(v.parse()?))
                }
            }
            SqlValue::SingleQuotedString(s) | SqlValue::DoubleQuotedString(s) => {
                Ok(LiteralValue::String(s.into()))
            }
            SqlValue::Boolean(b) => Ok(LiteralValue::Boolean(b)),
            SqlValue::Null => Ok(LiteralValue::Null),
            _ => Err(anyhow!("Value type not supported")),
        }
    }
}

// Convert Operation to Operator
impl TryFrom<Operation> for Operator {
    type Error = anyhow::Error;

    fn try_from(op: Operation) -> Result<Self, Self::Error> {
        match op.0 {
            SqlBinaryOperator::Plus => Ok(Operator::Plus),
            SqlBinaryOperator::Minus => Ok(Operator::Minus),
            SqlBinaryOperator::Multiply => Ok(Operator::Multiply),
            SqlBinaryOperator::Divide => Ok(Operator::Divide),
            SqlBinaryOperator::Modulo => Ok(Operator::Modulus),
            SqlBinaryOperator::Gt => Ok(Operator::Gt),
            SqlBinaryOperator::Lt => Ok(Operator::Lt),
            SqlBinaryOperator::GtEq => Ok(Operator::GtEq),
            SqlBinaryOperator::LtEq => Ok(Operator::LtEq),
            SqlBinaryOperator::Eq => Ok(Operator::Eq),
            SqlBinaryOperator::NotEq => Ok(Operator::NotEq),
            SqlBinaryOperator::And => Ok(Operator::And),
            SqlBinaryOperator::Or => Ok(Operator::Or),
            _ => Err(anyhow!("Operator not supported")),
        }
    }
}

// Convert Offset
impl<'a> From<Offset<'a>> for i64 {
    fn from(offset: Offset<'a>) -> Self {
        match offset.0 {
            SqlOffset {
                value: SqlExpr::Value(SqlValue::Number(v, _)),
                ..
            } => v.parse().unwrap_or(0),
            _ => 0,
        }
    }
}

// Convert Source
impl<'a> TryFrom<Source<'a>> for &'a str {
    type Error = anyhow::Error;

    fn try_from(source: Source<'a>) -> Result<Self, Self::Error> {
        if source.0.len() != 1 {
            return Err(anyhow!("Only support single table"));
        }

        let table = &source.0[0];
        if !table.joins.is_empty() {
            return Err(anyhow!("Only support single table without joins"));
        }

        match &table.relation {
            TableFactor::Table { name, .. } => Ok(&name.0.first().unwrap().value),
            _ => Err(anyhow!("Only support table")),
        }
    }
}

// Convert Order
impl<'a> TryFrom<Order<'a>> for (String, bool) {
    type Error = anyhow::Error;

    fn try_from(order: Order<'a>) -> Result<Self, Self::Error> {
        let expr = &order.0.expr;
        let asc = !order.0.asc.unwrap_or(true);
        match expr {
            SqlExpr::Identifier(id) => Ok((id.value.clone(), asc)),
            _ => Err(anyhow!("Order by only support identifier")),
        }
    }
}

// Convert Projection
impl<'a> TryFrom<Projection<'a>> for Expr {
    type Error = anyhow::Error;

    fn try_from(proj: Projection<'a>) -> Result<Self, Self::Error> {
        match proj.0 {
            SelectItem::UnnamedExpr(expr) => Expression(Box::new(expr.clone())).try_into(),
            SelectItem::ExprWithAlias { expr, alias } => {
                let expr: Expr = Expression(Box::new(expr.clone())).try_into()?;
                Ok(expr.alias(&alias.value))
            }
            SelectItem::Wildcard(_) => Ok(col("*")),
            _ => Err(anyhow!("Projection not supported")),
        }
    }
}

// Convert Expression to Expr
impl TryFrom<Expression> for Expr {
    type Error = anyhow::Error;

    fn try_from(expr: Expression) -> Result<Self, Self::Error> {
        match *expr.0 {
            SqlExpr::BinaryOp { left, op, right } => {
                let l: Expr = Expression(left).try_into()?;
                let r: Expr = Expression(right).try_into()?;
                let op = Operation(op).try_into()?;
                Ok(match op {
                    Operator::Plus => l + r,
                    Operator::Minus => l - r,
                    Operator::Multiply => l * r,
                    Operator::Divide => l / r,
                    Operator::Modulus => l % r,
                    Operator::Gt => l.gt(r),
                    Operator::Lt => l.lt(r),
                    Operator::GtEq => l.gt_eq(r),
                    Operator::LtEq => l.lt_eq(r),
                    Operator::Eq => l.eq(r),
                    Operator::NotEq => l.neq(r),
                    Operator::And => l.and(r),
                    Operator::Or => l.or(r),
                    _ => return Err(anyhow!("Operator not supported")),
                })
            }
            SqlExpr::Identifier(id) => Ok(col(&id.value)),
            SqlExpr::Value(v) => {
                let value: LiteralValue = Value(v).try_into()?;
                Ok(lit(value))
            }
            v => Err(anyhow!("Expression {} not supported", v)),
        }
    }
}

// Convert Statement to Sql
impl<'a> TryFrom<&'a Statement> for Sql<'a> {
    type Error = anyhow::Error;

    fn try_from(sql: &'a Statement) -> Result<Self, Self::Error> {
        match sql {
            Statement::Query(q) => {
                let offset = q.offset.as_ref().map(|v| Offset(v).into());
                let limit = q.limit.as_ref().and_then(|v| match v {
                    SqlExpr::Value(SqlValue::Number(n, _)) => n.parse().ok(),
                    _ => None,
                });

                let mut order_by = Vec::new();
                if let Some(orders) = &q.order_by {
                    for expr in orders.exprs.iter() {
                        order_by.push((
                            match &expr.expr {
                                SqlExpr::Identifier(id) => id.value.clone(),
                                _ => return Err(anyhow!("Order by only support identifier")),
                            },
                            !expr.asc.unwrap_or(true),
                        ));
                    }
                }

                let mut selection = Vec::new();
                let condition;

                if let SetExpr::Select(s) = &*q.body {
                    for p in &s.projection {
                        selection.push(Projection(p).try_into()?);
                    }

                    condition = s.selection
                        .as_ref()
                        .map(|v| Expression(Box::new(v.clone())).try_into())
                        .transpose()?;

                    Ok(Sql {
                        selection,
                        condition,
                        source: Source(&s.from).try_into()?,
                        order_by,
                        offset,
                        limit,
                    })
                } else {
                    Err(anyhow!("Only support Select query"))
                }
            }
            _ => Err(anyhow!("Only support Query")),
        }
    }
}
