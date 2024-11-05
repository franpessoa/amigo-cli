pub mod jogo {
    use std::path::PathBuf;

    use rusqlite::Connection;

    use crate::{
        cli::JogoFromFormat,
        import::{csv::CsvImporter, Importer},
    };

    pub fn jogo_ls(conn: &mut Connection) {
        let jogos = crate::db::jogo::get_all_jogos(conn);

        for j in jogos {
            tracing::info!("{:?}", j)
        }
    }

    pub fn jogo_new(conn: &mut Connection, nome: String) {
        tracing::info!("Criando jogo");
        let id = crate::db::jogo::create_jogo_with_nome(conn, &nome);

        tracing::info!("Criado jogo `{}` com id {}", nome, id);
    }

    pub fn jogo_rm(conn: &mut Connection, id: u64) {
        tracing::info!("Deletando jogo");
        let id = crate::db::jogo::delete_jogo_by_id(conn, id);

        tracing::info!("Deletado jogo de id {}", id)
    }

    pub fn jogo_from(conn: &mut Connection, format: JogoFromFormat, path: PathBuf, nome: String) {
        match format {
            JogoFromFormat::Csv => CsvImporter::from_path(path, conn, &nome),
        };
    }

    #[tracing::instrument(skip_all)]
    pub fn jogo_inspect(conn: &mut Connection, id: u64) {
        let jogo = crate::db::jogo::get_jogo_by_id(conn, id);

        tracing::info!("{:#?}", jogo);

        super::jogador::jogadores_ls_with_jogo(conn, jogo.id);
        super::sorteio::sorteios_ls_by_jogo(conn, jogo.id);
    }
}

pub mod jogador {
    use crate::cli::JogadoresSetParams;
    use rusqlite::Connection;

    pub fn jogadores_ls_with_jogo(conn: &mut Connection, jogo: u64) {
        let jogadores = crate::db::jogador::get_jogadores_by_jogo(conn, jogo);

        for j in jogadores {
            tracing::info!("{:?}", j)
        }
    }

    pub fn jogadores_ls_all(conn: &mut Connection) {
        let jogadores = crate::db::jogador::get_all_jogadores(conn);

        for j in jogadores {
            tracing::info!("{:?}", j)
        }
    }

    pub fn jogadores_add(conn: &mut Connection, jogo: u64, nome: String, email: String) {
        let id = crate::db::jogador::create_jogador(conn, jogo, nome, email);
        tracing::info!("Criado jogador com id {id}")
    }

    pub fn jogadores_inspect(conn: &mut Connection, id: u64) {
        let jogador = crate::db::jogador::get_jogador_by_id(conn, id);

        tracing::info!("{:#?}", jogador);
    }

    pub fn jogadores_set(conn: &mut Connection, id: u64, param: JogadoresSetParams) {
        let (collumn, new_value) = match param {
            JogadoresSetParams::Email { val } => ("email".to_string(), val),
            JogadoresSetParams::Nome { val } => ("nome".to_string(), val),
        };

        crate::db::jogador::update_jogador_by_collumn(conn, &collumn, new_value, id);

        tracing::info!("Atualizada propriedade {collumn} do id {id}");
    }

    pub fn jogadores_rm(conn: &mut Connection, id: u64) {
        let id = crate::db::jogador::delete_jogador_by_id(conn, id);

        tracing::info!("Removido jogador de id {id}")
    }
}

pub mod sorteio {
    use std::hash::Hash;

    use rand::{distributions::Alphanumeric, Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use rusqlite::Connection;

    use crate::config::Config;

    pub fn sorteio_new(conn: &mut Connection, jogo: u64) {
        let seed = create_seed();
        tracing::info!("Sorteada semente {}", seed);

        let jogadores = crate::db::jogador::get_jogadores_by_jogo(conn, jogo);
        let mut jogadores_ids = jogadores.iter().map(|x| x.id).collect::<Vec<u64>>();
        jogadores_ids.sort();

        let mut hasher = std::hash::DefaultHasher::new();
        jogadores_ids.hash(&mut hasher);

        let id = crate::db::sorteio::create_sorteio(conn, &seed, jogo, hasher, jogadores);

        tracing::info!("Criado sorteio com id {id}");
        tracing::info!("Use o comando `sorteio run` para rodá-lo");
    }

    fn create_seed() -> String {
        ChaCha20Rng::from_entropy()
            .sample_iter(Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>()
    }

    pub fn sorteio_run(conn: &mut Connection, id: u64, smtp_ctx: &Config) {
        let sorteio = crate::db::sorteio::get_sorteio_by_id(conn, &id);
        let jogadores = crate::db::jogador::get_jogadores_by_jogo(conn, sorteio.jogo);

        let ids = crate::db::envios::delete_envios_by_sorteio(conn, sorteio.id);

        tracing::warn!("Deletados envios com ids {:?}", ids);

        let _ = crate::envio::run_and_email(sorteio, jogadores, smtp_ctx, conn);

        for r in crate::db::envios::get_envios_by_sorteio(conn, id) {
            if r.sucesso {
                tracing::info!("Envio para id {} exitoso", r.destino)
            } else {
                tracing::info!(
                    "Envio para id {} não-exitoso com mensagem de erro {:?}",
                    r.destino,
                    r.erro
                )
            }
        }
    }

    pub fn sorteios_ls_by_jogo(conn: &mut Connection, jogo: u64) {
        let sorteios = crate::db::sorteio::get_sorteios_by_jogo(conn, jogo);

        for s in sorteios {
            tracing::info!("{:?}", s)
        }
    }

    pub fn sorteio_ls(conn: &mut Connection) {
        let sorteios = crate::db::sorteio::get_sorteios(conn);

        for s in sorteios {
            tracing::info!("{:?}", s)
        }
    }

    pub fn sorteio_inspect(conn: &mut Connection, id: u64) {
        let sorteio = crate::db::sorteio::get_sorteio_by_id(conn, &id);

        tracing::info!("{:#?}", sorteio)
    }
}

pub mod envio {
    use rusqlite::Connection;

    use crate::{
        config::Config,
        envio::{make_transport, ProcessoEnvio},
    };

    pub fn envio_inspect(conn: &mut Connection, envio: u64) {
        let envio = crate::db::envios::get_envio_by_id(conn, envio);

        tracing::info!("{:?}", envio);
    }

    pub fn envio_redo(conn: &mut Connection, ctx: &Config, envio: u64) {
        let envio = crate::db::envios::get_envio_by_id(conn, envio);
        let destino = crate::db::jogador::get_jogador_by_id(conn, envio.destino);
        let sorteado = crate::db::jogador::get_jogador_by_id(conn, envio.sorteado);
        let transport = make_transport(ctx);

        let processo = ProcessoEnvio {
            destino,
            sorteado,
            sorteio: envio.sorteio,
        };

        let id = processo.enviar(transport, conn, ctx);
        let new_envio = crate::db::envios::get_envio_by_id(conn, id.try_into().unwrap());

        tracing::info!("Criado um novo envio com id {}", id);
        if new_envio.sucesso {
            tracing::info!("Envio exitoso: {:#?}", new_envio)
        } else {
            tracing::error!("Erro! {:#?}", new_envio)
        }
    }

    pub fn envio_ls_all(conn: &mut Connection) {
        let envio = crate::db::envios::get_all_envios(conn);

        for e in envio {
            tracing::info!("{:?}", e)
        }
    }

    pub fn envio_ls_with_sorteio(conn: &mut Connection, sorteio: u64) {
        let envios = crate::db::envios::get_envios_by_sorteio(conn, sorteio);

        for e in envios {
            tracing::info!("{:?}", e)
        }
    }
}
