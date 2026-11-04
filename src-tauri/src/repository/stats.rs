use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::stats::Stats;

pub async fn get(pool: &SqlitePool) -> AppResult<Stats> {
    let stats = sqlx::query_as::<_, Stats>(
        "SELECT \
         (SELECT COUNT(*) FROM racks) AS rack_count, \
         (SELECT COUNT(*) FROM devices) AS device_count, \
         (SELECT COUNT(*) FROM credentials) AS credential_count, \
         (SELECT COUNT(*) FROM links) AS link_count",
    )
    .fetch_one(pool)
    .await?;
    Ok(stats)
}
