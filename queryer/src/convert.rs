use anyhow::{anyhow, Result};
use polars::prelude::*;
use sqlparser::ast::{
    BinaryOperator as SqlBinaryOperator, Expr as SqlExpr, Offset as SqlOffset, OrderByExpr, Select,
    SelectItem, SetExpr, Statement, TableFactor, TableWithJoins, Value as SqlValue,
    WildcardAdditionalOptions,
};
use std::sync::Arc;

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
pub struct Limit<'a>(pub(crate) &'a SqlExpr);
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
                let small_str: PlSmallStr = s
                    .as_str()
                    .try_into()
                    .map_err(|_| anyhow!("String too long for PlSmallStr"))?;
                Ok(LiteralValue::String(small_str))
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

// Convert Limit
impl<'a> TryFrom<Limit<'a>> for usize {
    type Error = anyhow::Error;

    fn try_from(limit: Limit<'a>) -> Result<Self, Self::Error> {
        match limit.0 {
            SqlExpr::Value(SqlValue::Number(v, _)) => Ok(v.parse()?),
            _ => Err(anyhow!("Limit must be a number")),
        }
    }
}

// Convert Source
impl<'a> TryFrom<Source<'a>> for &'a str {
    type Error = anyhow::Error;

    fn try_from(source: Source<'a>) -> Result<Self, Self::Error> {
        if source.0.len() != 1 {
            return Err(anyhow!("Only single table queries are supported"));
        }

        match &source.0[0].relation {
            TableFactor::Table { name, .. } => Ok(&name.0[0].value),
            _ => Err(anyhow!("Only table sources are supported")),
        }
    }
}

// Convert Order
impl<'a> TryFrom<Order<'a>> for (String, bool) {
    type Error = anyhow::Error;

    fn try_from(order: Order<'a>) -> Result<Self, Self::Error> {
        let asc = order.0.asc.unwrap_or(true);
        match &order.0.expr {
            SqlExpr::Identifier(ident) => Ok((ident.value.clone(), asc)),
            _ => Err(anyhow!("Order by only supports column names")),
        }
    }
}

// Convert Projection
impl<'a> TryFrom<Projection<'a>> for Expr {
    type Error = anyhow::Error;

    fn try_from(proj: Projection<'a>) -> Result<Self, Self::Error> {
        match proj.0 {
            SelectItem::UnnamedExpr(expr) => Expression(Box::new(expr.clone())).try_into(),
            SelectItem::ExprWithAlias { expr, alias } => Ok(Expression(Box::new(expr.clone()))
                .try_into::<Expr>()?
                .alias(&alias.value)),
            SelectItem::Wildcard(WildcardAdditionalOptions {
                opt_except,
                opt_exclude,
                ..
            }) => {
                if opt_except.is_some() || opt_exclude.is_some() {
                    Err(anyhow!("EXCEPT and EXCLUDE in wildcard are not supported"))
                } else {
                    Ok(col("*"))
                }
            }
            _ => Err(anyhow!("Projection type not supported")),
        }
    }
}

// Convert Expression to Expr
impl TryFrom<Expression> for Expr {
    type Error = anyhow::Error;

    fn try_from(expr: Expression) -> Result<Self, Self::Error> {
        match *expr.0 {
            SqlExpr::BinaryOp { left, op, right } => {
                let left: Expr = Expression(left).try_into()?;
                let right: Expr = Expression(right).try_into()?;
                match Operation(op).try_into()? {
                    Operator::Plus => Ok(left + right),
                    Operator::Minus => Ok(left - right),
                    Operator::Multiply => Ok(left * right),
                    Operator::Divide => Ok(left / right),
                    Operator::Modulus => Ok(left % right),
                    Operator::Gt => Ok(left.gt(right)),
                    Operator::Lt => Ok(left.lt(right)),
                    Operator::GtEq => Ok(left.gt_eq(right)),
                    Operator::LtEq => Ok(left.lt_eq(right)),
                    Operator::Eq => Ok(left.eq(right)),
                    Operator::NotEq => Ok(left.neq(right)),
                    Operator::And => Ok(left.and(right)),
                    Operator::Or => Ok(left.or(right)),
                    Operator::TrueDivide => Ok(left / right),
                    Operator::EqValidity => Ok(left.eq(right)),
                    Operator::NotEqValidity => Ok(left.neq(right)),
                    Operator::FloorDivide => Ok((left / right).cast(DataType::Int64)),
                    Operator::Xor => Ok(left.xor(right)),
                    Operator::LogicalAnd => Ok(left.and(right)),
                    Operator::LogicalOr => Ok(left.or(right)),
                }
            }
            SqlExpr::Identifier(id) => Ok(col(&id.value)),
            SqlExpr::Value(v) => Ok(lit(LiteralValue::try_from(Value(v))?)),
            SqlExpr::Wildcard => Ok(col("*")),
            SqlExpr::IsNull(expr) => Ok(Expression(expr).try_into::<Expr>()?.is_null()),
            v => Err(anyhow!("Expression type not supported: {:?}", v)),
        }
    }
}

// Convert Statement to Sql
impl<'a> TryFrom<&'a Statement> for Sql<'a> {
    type Error = anyhow::Error;

    fn try_from(sql: &'a Statement) -> Result<Self, Self::Error> {
        match sql {
            Statement::Query(query) => {
                let Select {
                    from: table_with_joins,
                    selection: where_clause,
                    projection,
                    ..
                } = match &*query.body {
                    SetExpr::Select(statement) => statement.as_ref(),
                    _ => return Err(anyhow!("Only SELECT queries are supported")),
                };

                let source = Source(table_with_joins).try_into()?;

                let mut selection = Vec::with_capacity(projection.len());
                for p in projection {
                    selection.push(Projection(p).try_into()?);
                }

                let condition = where_clause
                    .as_ref()
                    .map(|expr| Expression(Box::new(expr.clone())).try_into())
                    .transpose()?;

                let mut order_by = Vec::new();
                for expr in &query.order_by {
                    order_by.push(Order(expr).try_into()?);
                }

                let offset = query.offset.as_ref().map(|v| Offset(v).into());
                let limit = query
                    .limit
                    .as_ref()
                    .map(|v| Limit(v).try_into())
                    .transpose()?;

                Ok(Sql {
                    selection,
                    condition,
                    source,
                    order_by,
                    offset,
                    limit,
                })
            }
            _ => Err(anyhow!("Only SELECT queries are supported")),
        }
    }
}
