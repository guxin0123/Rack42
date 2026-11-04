use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误：{0}")]
    Database(#[from] sqlx::Error),
    #[error("文件操作失败：{0}")]
    Io(#[from] std::io::Error),
    #[error("数据序列化失败：{0}")]
    Serde(#[from] serde_json::Error),
    #[error("尚未设置主密码")]
    NotInitialized,
    #[error("会话已锁定，请重新解锁")]
    Locked,
    #[error("主密码错误")]
    InvalidPassword,
    #[error("未找到对应记录")]
    NotFound,
    #[error("{0}")]
    Validation(String),
    #[error("加密操作失败：{0}")]
    Crypto(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub fn unique_violation(error: sqlx::Error, message: &str) -> AppError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.is_unique_violation() {
            return AppError::Validation(message.to_string());
        }
    }
    AppError::Database(error)
}
