use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};

// ─── Modeller ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserInput {
    pub name: String,
    pub email: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserInput {
    pub id: i64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub price: f64,
    pub stock: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProductInput {
    pub name: String,
    pub category: String,
    pub price: f64,
    pub stock: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProductInput {
    pub id: i64,
    pub name: Option<String>,
    pub category: Option<String>,
    pub price: Option<f64>,
    pub stock: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_products: i64,
    pub active_users: i64,
    pub active_products: i64,
}

// ─── Kullanıcı Komutları ─────────────────────────────────────────────────────

#[tauri::command]
pub fn get_users(db: Db<'_>) -> Result<Vec<User>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, email, role, status, created_at, updated_at
             FROM users ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let users = stmt
        .query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                role: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(users)
}

#[tauri::command]
pub fn create_user(db: Db<'_>, input: CreateUserInput) -> Result<User, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let role = input.role.unwrap_or_else(|| "user".to_string());

    conn.execute(
        "INSERT INTO users (name, email, role) VALUES (?1, ?2, ?3)",
        params![input.name, input.email, role],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let user = conn
        .query_row(
            "SELECT id, name, email, role, status, created_at, updated_at
             FROM users WHERE id = ?1",
            params![id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    email: row.get(2)?,
                    role: row.get(3)?,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(user)
}

#[tauri::command]
pub fn update_user(db: Db<'_>, input: UpdateUserInput) -> Result<User, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE users SET
            name       = COALESCE(?1, name),
            email      = COALESCE(?2, email),
            role       = COALESCE(?3, role),
            status     = COALESCE(?4, status),
            updated_at = datetime('now')
         WHERE id = ?5",
        params![input.name, input.email, input.role, input.status, input.id],
    )
    .map_err(|e| e.to_string())?;

    let user = conn
        .query_row(
            "SELECT id, name, email, role, status, created_at, updated_at
             FROM users WHERE id = ?1",
            params![input.id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    email: row.get(2)?,
                    role: row.get(3)?,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(user)
}

#[tauri::command]
pub fn delete_user(db: Db<'_>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM users WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Ürün Komutları ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_products(db: Db<'_>) -> Result<Vec<Product>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, category, price, stock, status, created_at, updated_at
             FROM products ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let products = stmt
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                price: row.get(3)?,
                stock: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(products)
}

#[tauri::command]
pub fn create_product(db: Db<'_>, input: CreateProductInput) -> Result<Product, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let stock = input.stock.unwrap_or(0);

    conn.execute(
        "INSERT INTO products (name, category, price, stock) VALUES (?1, ?2, ?3, ?4)",
        params![input.name, input.category, input.price, stock],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let product = conn
        .query_row(
            "SELECT id, name, category, price, stock, status, created_at, updated_at
             FROM products WHERE id = ?1",
            params![id],
            |row| {
                Ok(Product {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    price: row.get(3)?,
                    stock: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(product)
}

#[tauri::command]
pub fn update_product(db: Db<'_>, input: UpdateProductInput) -> Result<Product, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE products SET
            name       = COALESCE(?1, name),
            category   = COALESCE(?2, category),
            price      = COALESCE(?3, price),
            stock      = COALESCE(?4, stock),
            status     = COALESCE(?5, status),
            updated_at = datetime('now')
         WHERE id = ?6",
        params![
            input.name,
            input.category,
            input.price,
            input.stock,
            input.status,
            input.id
        ],
    )
    .map_err(|e| e.to_string())?;

    let product = conn
        .query_row(
            "SELECT id, name, category, price, stock, status, created_at, updated_at
             FROM products WHERE id = ?1",
            params![input.id],
            |row| {
                Ok(Product {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    price: row.get(3)?,
                    stock: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(product)
}

#[tauri::command]
pub fn delete_product(db: Db<'_>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM products WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Dashboard İstatistikleri ─────────────────────────────────────────────────

#[tauri::command]
pub fn get_dashboard_stats(db: Db<'_>) -> Result<DashboardStats, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let total_users: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let active_users: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE status = 'active'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let total_products: i64 = conn
        .query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let active_products: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM products WHERE status = 'active'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(DashboardStats {
        total_users,
        total_products,
        active_users,
        active_products,
    })
}
