use lettre::{
    transport::smtp::{authentication::Credentials, client::TlsParameters},
    Message, SmtpTransport, Transport,
};
use rand::seq::SliceRandom;
use rand_chacha::ChaCha20Rng;
use rusqlite::{params, Connection};

use crate::{
    config::Config,
    db::{Jogador, Sorteio},
};

pub fn make_transport(ctx: &crate::config::Config) -> SmtpTransport {
    let credentials = Credentials::new(ctx.smtp_username.clone(), ctx.smtp_password.clone());

    let tls_param = TlsParameters::builder(ctx.smtp_relay.clone())
        .build()
        .unwrap();

    SmtpTransport::relay(&ctx.smtp_relay)
        .unwrap()
        .port(ctx.smtp_port)
        .tls(lettre::transport::smtp::client::Tls::Opportunistic(
            tls_param,
        ))
        .credentials(credentials)
        .build()
}

#[derive(Debug, Clone)]
pub struct ProcessoEnvio {
    pub destino: Jogador,
    pub sorteado: Jogador,
    pub sorteio: u64,
}

impl ProcessoEnvio {
    pub fn enviar<T: Transport>(
        self,
        sender: T,
        conn: &mut Connection,
        ctx: &crate::config::Config,
    ) -> usize
    where
        T::Error: ToString,
    {
        let message = Message::builder()
            .from(ctx.smtp_sender.parse().unwrap())
            .to(format!(
                "{} <{}>",
                self.destino.nome.clone(),
                self.destino.email.clone()
            )
            .parse()
            .unwrap())
            .subject(ctx.subject.clone())
            .body((ctx.format_message)(self.clone()))
            .unwrap();

        let result = sender.send(&message);

        match result {
            Ok(_) => register_success(&self, conn),
            Err(x) => register_error(&self, conn, x.to_string()),
        }
    }
}

fn register_success(processo: &ProcessoEnvio, conn: &mut Connection) -> usize {
    let mut query = conn
        .prepare("INSERT INTO envios (sorteio, destino, sorteado, sucesso) VALUES (?1, ?2, ?3, ?4) RETURNING id")
        .unwrap();

    query
        .query_row(
            params![
                processo.sorteio,
                processo.destino.id,
                processo.sorteado.id,
                true
            ],
            |x| Ok(x.get(0).unwrap()),
        )
        .unwrap()
}

#[tracing::instrument(level = "debug")]
fn register_error(processo: &ProcessoEnvio, conn: &mut Connection, error: String) -> usize {
    let mut query = conn
        .prepare("INSERT INTO envios (sorteio, destino, sorteado, sucesso, erro) VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id")
        .unwrap();

    query
        .query_row(
            params![
                processo.sorteio,
                processo.destino.id,
                processo.sorteado.id,
                false,
                error
            ],
            |x| Ok(x.get(0).unwrap()),
        )
        .unwrap()
}

/// Realmente roda o sorteio, enviando os emails.
///
/// É o núcleo de todo o funcionamento
pub fn run_and_email(
    sorteio: Sorteio,
    mut jogadores: Vec<Jogador>,
    smtp_ctx: &Config,
    conn: &mut Connection,
) -> Vec<usize> {
    // shuffle jogadores according to seed
    let mut rand: ChaCha20Rng = rand_seeder::Seeder::from(sorteio.seed.clone()).make_rng();
    jogadores.shuffle(&mut rand);

    let transport = make_transport(&smtp_ctx);

    let mut results = iter_and_send(jogadores, sorteio.clone(), transport, conn, smtp_ctx);

    results.shuffle(&mut rand); // so the cli printing order does not reflect actual emails;
    results
}

fn iter_and_send(
    jogadores: Vec<Jogador>,
    sorteio: Sorteio,
    transport: lettre::SmtpTransport,
    conn: &mut Connection,
    smtp_ctx: &Config,
) -> Vec<usize> {
    let mut results = vec![];

    for (idx, j) in jogadores.iter().enumerate() {
        let processo = {
            if idx == (jogadores.len() - 1) {
                ProcessoEnvio {
                    destino: j.clone(),
                    sorteado: jogadores[0].clone(),
                    sorteio: sorteio.id,
                }
            } else {
                ProcessoEnvio {
                    destino: j.clone(),
                    sorteado: jogadores[idx + 1].clone(),
                    sorteio: sorteio.id,
                }
            }
        };

        results.push(processo.enviar(transport.clone(), conn, &smtp_ctx));
    }

    return results;
}
