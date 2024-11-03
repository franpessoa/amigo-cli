use crate::envio::ProcessoEnvio;

pub struct Config {
    pub smtp_sender: String,
    pub smtp_relay: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub format_message: fn(ProcessoEnvio) -> String,
    pub subject: String,

    pub db_path: String,
}

impl Config {
    pub fn from_env() -> Config {
        dotenvy::dotenv().unwrap();

        Config {
            smtp_sender: std::env::var("SMTP_SENDER").unwrap(),
            smtp_relay: std::env::var("SMTP_RELAY").unwrap(),
            smtp_port: std::env::var("SMTP_PORT")
                .unwrap_or("587".to_owned())
                .parse()
                .unwrap(),
            smtp_username: std::env::var("SMTP_USER").unwrap(),
            smtp_password: std::env::var("SMTP_PASSWORD").unwrap(),
            format_message: |e| {
                return format!("Seu amigo secreto foi sorteado! É {}", e.sorteado.nome);
            },
            subject: "Amigo Secreto".to_owned(),
            db_path: std::env::var("DB_PATH").unwrap_or("sqlite.db".to_string()),
        }
    }
}
