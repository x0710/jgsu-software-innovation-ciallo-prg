use sqlx::SqlitePool;
use std::env;

pub async fn create_pool() -> Result<SqlitePool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://blog.db".to_string());

    let pool = SqlitePool::connect(&database_url).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS questions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            answer TEXT NOT NULL DEFAULT '待回答...',
            author TEXT NOT NULL,
            color TEXT NOT NULL DEFAULT 'yellow',
            created_at TEXT
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS timeline_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            weekday TEXT NOT NULL,
            time TEXT NOT NULL,
            title TEXT NOT NULL,
            event_type TEXT NOT NULL DEFAULT 'info'
        )"
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn seed_data(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM questions")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if count.0 == 0 {
        seed_questions(pool).await?;
        seed_timeline(pool).await?;
    }

    Ok(())
}

async fn seed_questions(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let questions = vec![
        (
            "实验室主要研究什么？",
            "我们聚焦软件工程、人工智能、大数据、物联网等方向的研究与实践，致力于用代码解决实际问题，产出有价值的创新成果。",
            "小萌新",
            "yellow",
        ),
        (
            "新生可以参加哪些项目？",
            "提供多样化项目机会，覆盖不同方向：\n• 校内外学科竞赛（蓝桥杯、挑战杯等）\n• 企业合作项目实践\n• 科研课题参与\n• 开源项目共建\n总有一款适合你！",
            "好奇宝宝",
            "orange",
        ),
        (
            "需要提前学习哪些技术？",
            "不用焦虑！我们更看重学习能力和兴趣~\n建议先掌握这些基础：\n• 一门编程语言（C/Java/Python任选其一）\n• 数据结构与算法基础\n• 计算机基础知识（操作系统、网络等）\n实验室会提供系统的学习资源和培训！",
            "预备役",
            "pink",
        ),
    ];

    for (title, answer, author, color) in &questions {
        sqlx::query(
            "INSERT INTO questions (id, title, answer, author, color, created_at) VALUES (?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(title)
        .bind(answer)
        .bind(author)
        .bind(color)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_timeline(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let events = vec![
        ("5月20日", "周一", "10:00", "实验室招新启动！", "info"),
        ("5月22日", "周三", "15:00", "Q&A第一弹", "qa"),
        ("5月24日", "周五", "19:30", "项目学长面对面", "meet"),
        ("5月27日", "周一", "10:00", "Q&A第二弹", "qa"),
        ("5月30日", "周四", "20:00", "招新宣讲会", "info"),
        ("6月5日", "周三", "23:59", "招新报名截止", "deadline"),
    ];

    for (date, weekday, time, title, event_type) in &events {
        sqlx::query(
            "INSERT INTO timeline_events (date, weekday, time, title, event_type) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(date)
        .bind(weekday)
        .bind(time)
        .bind(title)
        .bind(event_type)
        .execute(pool)
        .await?;
    }

    Ok(())
}
