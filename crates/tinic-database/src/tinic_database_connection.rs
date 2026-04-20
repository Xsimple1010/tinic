use sqlite::Connection;
use std::{path::PathBuf, sync::Arc};
use tinic_generics::error_handle::ErrorHandle;
use tokio::sync::{Mutex, MutexGuard};

#[derive(Clone)]
pub struct TinicDbConnection {
    conn: Arc<Mutex<Connection>>,
}

impl TinicDbConnection {
    pub fn new(path: PathBuf) -> Result<Self, ErrorHandle> {
        let connection = sqlite::open(path.join("games.sqlite"))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn in_memory() -> Result<Self, ErrorHandle> {
        let connection = sqlite::open(":memory:")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(connection)),
        })
    }

    pub async fn try_execute<T: AsRef<str>>(&self, statement: T) -> Result<(), ErrorHandle> {
        self.conn.lock().await.execute(statement)?;
        Ok(())
    }

    pub async fn execute<T: AsRef<str>>(&self, statement: T) -> Result<(), ErrorHandle> {
        Ok(self.conn.lock().await.execute(statement)?)
    }

    pub async fn with_statement<F, R>(&self, query: &str, mut callback: F) -> Result<R, ErrorHandle>
    where
        F: FnMut(&mut sqlite::Statement, &MutexGuard<Connection>) -> Result<R, ErrorHandle>,
    {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(query)?;

        callback(&mut stmt, &conn)
    }
}
