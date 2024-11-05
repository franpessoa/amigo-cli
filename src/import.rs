use serde::Deserialize;
use std::path::PathBuf;

pub trait Importer {
    fn from_path(
        path: PathBuf,
        conn: &mut rusqlite::Connection,
        jogo_nome: &str,
    ) -> (usize, Vec<usize>);
}

#[derive(Debug, Deserialize)]
struct ImportedJogador {
    #[serde(rename = "Nome")]
    nome: String,
    #[serde(rename = "Email")]
    email: String,
}

pub mod csv {
    use super::{ImportedJogador, Importer};
    pub struct CsvImporter {}

    impl Importer for CsvImporter {
        fn from_path(
            path: std::path::PathBuf,
            conn: &mut rusqlite::Connection,
            jogo_nome: &str,
        ) -> (usize, Vec<usize>) {
            let mut reader = csv::Reader::from_path(path).unwrap();
            let jogo = crate::db::jogo::create_jogo_with_nome(conn, &jogo_nome.to_owned());
            let mut jogadores: Vec<usize> = Vec::new();

            tracing::info!("Criado jogo com id {jogo}");

            for r in reader.deserialize() {
                let record: ImportedJogador = r.unwrap();

                let id = crate::db::jogador::create_jogador(
                    conn,
                    jogo.try_into().unwrap(),
                    record.nome.clone(),
                    record.email.clone(),
                );

                tracing::info!(
                    "Criado jogador com id {id}, nome {} e email <{}>",
                    record.nome.clone(),
                    record.email.clone()
                );

                jogadores.push(id);
            }

            return (jogo, jogadores);
        }
    }
}
