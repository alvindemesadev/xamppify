use mysql_async::prelude::*;
use mysql_async::{OptsBuilder, Pool, Row};
use std::sync::OnceLock;
use tokio::sync::Mutex;

const LOCAL_MYSQL: &str = "127.0.0.1";
const MYSQL_PORT: u16 = 3306;

static POOL: OnceLock<Mutex<Option<Pool>>> = OnceLock::new();

fn pool() -> &'static Mutex<Option<Pool>> {
    POOL.get_or_init(|| Mutex::new(None))
}

fn build_opts(host: &str, port: u16, user: &str, password: &str) -> OptsBuilder {
    OptsBuilder::default()
        .ip_or_hostname(host.to_string())
        .tcp_port(port)
        .user(Some(user.to_string()))
        .pass(Some(password.to_string()))
}

fn quote_identifier(identifier: &str) -> String {
    format!("`{}`", identifier.replace('`', "``"))
}

pub async fn connect(
    host: Option<String>,
    port: Option<u16>,
    user: String,
    password: String,
) -> Result<(), String> {
    let host = host.unwrap_or_else(|| LOCAL_MYSQL.to_string());
    let port = port.unwrap_or(MYSQL_PORT);

    let opts = build_opts(&host, port, &user, &password);
    let new_pool = Pool::new(opts);

    // verify connection works
    let conn = new_pool
        .get_conn()
        .await
        .map_err(|e| format!("MySQL connection failed: {}", e))?;
    drop(conn);

    let mut guard = pool().lock().await;
    *guard = Some(new_pool);
    Ok(())
}

pub async fn disconnect() {
    let mut guard = pool().lock().await;
    if let Some(p) = guard.take() {
        drop(p);
    }
}

pub async fn list_databases() -> Result<Vec<String>, String> {
    let mut guard = pool().lock().await;
    let p = guard
        .as_mut()
        .ok_or_else(|| "Not connected to MySQL".to_string())?;
    let mut conn = p
        .get_conn()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let mut rows: Vec<String> = conn
        .query("SHOW DATABASES")
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    rows.retain(|db| db != "information_schema" && db != "performance_schema" && db != "sys");
    Ok(rows)
}

pub async fn list_tables(database: &str) -> Result<Vec<String>, String> {
    let mut guard = pool().lock().await;
    let p = guard
        .as_mut()
        .ok_or_else(|| "Not connected to MySQL".to_string())?;
    let mut conn = p
        .get_conn()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let rows: Vec<String> = conn
        .query(format!("SHOW TABLES FROM {}", quote_identifier(database)))
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    Ok(rows)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<serde_json::Value>,
    pub affected_rows: u64,
}

pub async fn run_query(query: &str) -> Result<QueryResult, String> {
    let mut guard = pool().lock().await;
    let p = guard
        .as_mut()
        .ok_or_else(|| "Not connected to MySQL".to_string())?;
    let mut conn = p
        .get_conn()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let upper = query.trim().to_uppercase();
    let is_select = upper.starts_with("SELECT")
        || upper.starts_with("SHOW")
        || upper.starts_with("DESC")
        || upper.starts_with("EXPLAIN");

    if is_select {
        let rows: Vec<Row> = conn
            .query(query)
            .await
            .map_err(|e| format!("Query failed: {}", e))?;

        let columns = if let Some(first) = rows.first() {
            first
                .columns_ref()
                .iter()
                .map(|c| c.name_str().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let json_rows: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (i, col) in columns.iter().enumerate() {
                    let val: Option<String> = row.get(i).unwrap_or(None);
                    map.insert(
                        col.clone(),
                        serde_json::Value::String(val.unwrap_or_default()),
                    );
                }
                serde_json::Value::Object(map)
            })
            .collect();

        Ok(QueryResult {
            columns,
            rows: json_rows,
            affected_rows: 0,
        })
    } else {
        conn.exec_drop(query, ())
            .await
            .map_err(|e| format!("Query failed: {}", e))?;

        let affected_rows = conn.affected_rows();
        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows,
        })
    }
}

pub async fn export_database(database: &str) -> Result<String, String> {
    let tables = list_tables(database).await?;
    let mut output = String::new();
    output.push_str(&format!("-- Export of database: {}\n\n", database));

    let mut guard = pool().lock().await;
    let p = guard.as_mut().ok_or_else(|| "Not connected".to_string())?;
    let mut conn = p
        .get_conn()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    for table in &tables {
        let create_rows: Vec<Row> = conn
            .query(format!(
                "SHOW CREATE TABLE {}.{}",
                quote_identifier(database),
                quote_identifier(table)
            ))
            .await
            .map_err(|e| format!("Failed to get CREATE TABLE: {}", e))?;

        if let Some(row) = create_rows.first() {
            let stmt: Option<String> = row.get(1).unwrap_or(None);
            if let Some(s) = stmt {
                output.push_str(&s);
                output.push_str(";\n\n");
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::quote_identifier;

    #[test]
    fn quotes_mysql_identifiers() {
        assert_eq!(quote_identifier("client`data"), "`client``data`");
    }
}
