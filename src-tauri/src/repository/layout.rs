use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::layout::LayoutNode;

pub async fn load(pool: &SqlitePool, view: &str) -> AppResult<Vec<LayoutNode>> {
    let nodes = sqlx::query_as::<_, LayoutNode>(
        "SELECT device_id, x, y FROM topology_layout WHERE view_name = ? ORDER BY device_id",
    )
    .bind(view)
    .fetch_all(pool)
    .await?;
    Ok(nodes)
}

pub async fn save(pool: &SqlitePool, view: &str, nodes: &[LayoutNode]) -> AppResult<()> {
    let mut transaction = pool.begin().await?;

    sqlx::query("DELETE FROM topology_layout WHERE view_name = ?")
        .bind(view)
        .execute(&mut *transaction)
        .await?;

    for node in nodes {
        sqlx::query("INSERT INTO topology_layout (view_name, device_id, x, y) VALUES (?, ?, ?, ?)")
            .bind(view)
            .bind(node.device_id)
            .bind(node.x)
            .bind(node.y)
            .execute(&mut *transaction)
            .await?;
    }

    transaction.commit().await?;
    Ok(())
}
