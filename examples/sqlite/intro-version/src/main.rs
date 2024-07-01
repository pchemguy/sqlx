use std::env;
use sqlx::sqlite::SqlitePool;


#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    println!("\nSQLite version number:\n");
    exec(&pool).await?;

    Ok(())
}


async fn exec(pool: &SqlitePool) -> anyhow::Result<()> {
    let recs = sqlx::query!(
        r#"
            SELECT sqlite_version() AS version
        "#
    )
    .fetch_all(pool)
    .await?;

    for rec in recs {
        println!(
            "{}",
            rec.version.unwrap(),
        );
    }

    Ok(())
}
