use std::env;
use sqlx::sqlite::SqlitePool;


#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    println!("SQLite version number");
    exec(&pool).await?;

    Ok(())
}


async fn exec(pool: &SqlitePool) -> anyhow::Result<()> {
    let recs = sqlx::query!(
        r#"
            SELECT
                1 AS id,
                (SELECT group_concat(name) FROM pragma_module_list()) AS name
        "#
    )
    .fetch_all(pool)
    .await?;

    for rec in recs {
        println!(
            "- {}: {}",
            rec.id,
            rec.name.unwrap(),
        );
    }

    Ok(())
}
