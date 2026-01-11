use crate::error::ApiError;
use crate::models::request::{ExposureType, PivotFilters, PivotRequest, SortDirection};
use crate::query::{Dimension, Metric};

pub struct PivotQueryBuilder {
    dimensions: Vec<Dimension>,
    metrics: Vec<Metric>,
    filters: PivotFilters,
    sort_field: Option<String>,
    sort_direction: SortDirection,
    limit: u32,
    offset: u32,
}

impl PivotQueryBuilder {
    pub fn from_request(req: &PivotRequest) -> Result<Self, ApiError> {
        if req.dimensions.is_empty() {
            return Err(ApiError::QueryValidation(
                "At least one dimension is required".to_string(),
            ));
        }

        if req.metrics.is_empty() {
            return Err(ApiError::QueryValidation(
                "At least one metric is required".to_string(),
            ));
        }

        let (sort_field, sort_direction) = match &req.sort {
            Some(sort) => (Some(sort.field.clone()), sort.direction.clone()),
            None => (None, SortDirection::Desc),
        };

        Ok(Self {
            dimensions: req.dimensions.clone(),
            metrics: req.metrics.clone(),
            filters: req.filters.clone(),
            sort_field,
            sort_direction,
            limit: req.limit,
            offset: req.offset,
        })
    }

    pub fn build(&self) -> String {
        let mut sql = String::new();

        // SELECT clause
        sql.push_str("SELECT ");

        // Add dimensions
        let dim_cols: Vec<&str> = self.dimensions.iter().map(|d| d.to_column()).collect();
        sql.push_str(&dim_cols.join(", "));

        // Add metrics
        for metric in &self.metrics {
            sql.push_str(", ");
            sql.push_str(metric.to_aggregation());
            sql.push_str(" AS ");
            sql.push_str(metric.alias());
        }

        // FROM clause
        sql.push_str(" FROM pivot.trades_1d");

        // WHERE clause
        let where_clauses = self.build_where_clauses();
        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
        }

        // GROUP BY clause
        sql.push_str(" GROUP BY ");
        sql.push_str(&dim_cols.join(", "));

        // ORDER BY clause
        if let Some(ref field) = self.sort_field {
            sql.push_str(" ORDER BY ");
            sql.push_str(field);
            sql.push_str(match self.sort_direction {
                SortDirection::Asc => " ASC",
                SortDirection::Desc => " DESC",
            });
        }

        // LIMIT and OFFSET
        sql.push_str(&format!(" LIMIT {}", self.limit));
        if self.offset > 0 {
            sql.push_str(&format!(" OFFSET {}", self.offset));
        }

        sql
    }

    fn build_where_clauses(&self) -> Vec<String> {
        let mut clauses = Vec::new();

        if let Some(ref date) = self.filters.trade_date {
            clauses.push(format!("trade_date = '{}'", Self::escape_string(date)));
        }

        if let Some(ref range) = self.filters.trade_date_range {
            clauses.push(format!(
                "trade_date >= '{}' AND trade_date <= '{}'",
                Self::escape_string(&range.start),
                Self::escape_string(&range.end)
            ));
        }

        if let Some(ref types) = self.filters.exposure_type {
            if !types.is_empty() {
                let values: Vec<String> = types
                    .iter()
                    .map(|t| format!("'{}'", Self::exposure_type_to_string(t)))
                    .collect();
                clauses.push(format!("exposure_type IN ({})", values.join(", ")));
            }
        }

        if let Some(ref ids) = self.filters.portfolio_manager_id {
            if !ids.is_empty() {
                let values: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
                clauses.push(format!("portfolio_manager_id IN ({})", values.join(", ")));
            }
        }

        if let Some(ref ids) = self.filters.fund_id {
            if !ids.is_empty() {
                let values: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
                clauses.push(format!("fund_id IN ({})", values.join(", ")));
            }
        }

        if let Some(ref classes) = self.filters.asset_class {
            if !classes.is_empty() {
                let values: Vec<String> = classes
                    .iter()
                    .map(|c| format!("'{}'", Self::escape_string(c)))
                    .collect();
                clauses.push(format!("asset_class IN ({})", values.join(", ")));
            }
        }

        if let Some(ref symbols) = self.filters.symbol {
            if !symbols.is_empty() {
                let values: Vec<String> = symbols
                    .iter()
                    .map(|s| format!("'{}'", Self::escape_string(s)))
                    .collect();
                clauses.push(format!("symbol IN ({})", values.join(", ")));
            }
        }

        if let Some(ref symbols) = self.filters.underlying_symbol {
            if !symbols.is_empty() {
                let values: Vec<String> = symbols
                    .iter()
                    .map(|s| format!("'{}'", Self::escape_string(s)))
                    .collect();
                clauses.push(format!("underlying_symbol IN ({})", values.join(", ")));
            }
        }

        if let Some(ref desks) = self.filters.desk {
            if !desks.is_empty() {
                let values: Vec<String> = desks
                    .iter()
                    .map(|d| format!("'{}'", Self::escape_string(d)))
                    .collect();
                clauses.push(format!("desk IN ({})", values.join(", ")));
            }
        }

        if let Some(ref books) = self.filters.book {
            if !books.is_empty() {
                let values: Vec<String> = books
                    .iter()
                    .map(|b| format!("'{}'", Self::escape_string(b)))
                    .collect();
                clauses.push(format!("book IN ({})", values.join(", ")));
            }
        }

        if let Some(ref regions) = self.filters.region {
            if !regions.is_empty() {
                let values: Vec<String> = regions
                    .iter()
                    .map(|r| format!("'{}'", Self::escape_string(r)))
                    .collect();
                clauses.push(format!("region IN ({})", values.join(", ")));
            }
        }

        if let Some(ref countries) = self.filters.country {
            if !countries.is_empty() {
                let values: Vec<String> = countries
                    .iter()
                    .map(|c| format!("'{}'", Self::escape_string(c)))
                    .collect();
                clauses.push(format!("country IN ({})", values.join(", ")));
            }
        }

        clauses
    }

    fn escape_string(s: &str) -> String {
        // For date strings, only allow date characters
        if s.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return s.to_string();
        }
        // For other strings, be more restrictive - only alphanumeric and underscores
        s.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }

    fn exposure_type_to_string(t: &ExposureType) -> &'static str {
        match t {
            ExposureType::Direct => "Direct",
            ExposureType::Etf => "ETF",
            ExposureType::Etc => "ETC",
            ExposureType::Constituent => "Constituent",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let req = PivotRequest {
            dimensions: vec![Dimension::AssetClass],
            metrics: vec![Metric::Notional, Metric::Pnl],
            filters: PivotFilters {
                trade_date: Some("2024-01-15".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let builder = PivotQueryBuilder::from_request(&req).unwrap();
        let sql = builder.build();

        assert!(sql.contains("SELECT asset_class"));
        assert!(sql.contains("sum(notional) AS total_notional"));
        assert!(sql.contains("sum(pnl) AS total_pnl"));
        assert!(sql.contains("FROM pivot.trades_1d"));
        assert!(sql.contains("WHERE trade_date = '2024-01-15'"));
        assert!(sql.contains("GROUP BY asset_class"));
    }

    #[test]
    fn test_exposure_type_filter() {
        let req = PivotRequest {
            dimensions: vec![Dimension::Symbol],
            metrics: vec![Metric::Notional],
            filters: PivotFilters {
                exposure_type: Some(vec![ExposureType::Direct, ExposureType::Etf]),
                ..Default::default()
            },
            ..Default::default()
        };

        let builder = PivotQueryBuilder::from_request(&req).unwrap();
        let sql = builder.build();

        assert!(sql.contains("exposure_type IN ('Direct', 'ETF')"));
    }

    #[test]
    fn test_multiple_dimensions() {
        let req = PivotRequest {
            dimensions: vec![
                Dimension::PortfolioManagerId,
                Dimension::AssetClass,
                Dimension::Symbol,
            ],
            metrics: vec![Metric::TradeCount],
            filters: PivotFilters::default(),
            ..Default::default()
        };

        let builder = PivotQueryBuilder::from_request(&req).unwrap();
        let sql = builder.build();

        assert!(sql.contains("SELECT portfolio_manager_id, asset_class, symbol"));
        assert!(sql.contains("GROUP BY portfolio_manager_id, asset_class, symbol"));
    }

    #[test]
    fn test_sql_injection_prevention() {
        let req = PivotRequest {
            dimensions: vec![Dimension::AssetClass],
            metrics: vec![Metric::Notional],
            filters: PivotFilters {
                trade_date: Some("2024-01-15'; DROP TABLE trades_1d; --".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let builder = PivotQueryBuilder::from_request(&req).unwrap();
        let sql = builder.build();

        assert!(!sql.contains("DROP TABLE"));
        assert!(!sql.contains(";"));
    }
}
