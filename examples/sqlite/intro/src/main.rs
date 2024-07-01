use std::env;
use sqlx::sqlite::SqlitePool;


#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    println!("\nSQLite Introspection Information");
    exec(&pool).await?;

    Ok(())
}


async fn exec(pool: &SqlitePool) -> anyhow::Result<()> {
    let recs = sqlx::query_file!("queries/intro.sql")
        .fetch_all(pool)
        .await?;
    
    for rec in recs {
        println!(
            "{}",
            rec.info.unwrap(),
        );
    }

    Ok(())
}
