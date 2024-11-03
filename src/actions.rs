pub mod jogo {
    use rusqlite::Connection;

    pub fn jogo_ls(conn: &mut Connection) {
        let jogos = crate::db::jogo::get_all_jogos(conn);

        for j in jogos {
            println!("{:?}", j)
        }
    }

    pub fn jogo_new(conn: &mut Connection, nome: String) {
        let id = crate::db::jogo::create_jogo_with_nome(conn, &nome);

        println!("Criado jogo `{}` com id {}", nome, id);
    }

    pub fn jogo_rm(conn: &mut Connection, id: u64) {
        let id = crate::db::jogo::delete_jogo_by_id(conn, id);

        println!("Deletado jogo de id {}", id)
    }
}

pub mod jogador {
    use crate::cli::JogadoresSetParams;
    use rusqlite::Connection;

    pub fn jogadores_ls_with_jogo(conn: &mut Connection, jogo: u64) {
        let jogadores = crate::db::jogador::get_jogadores_by_jogo(conn, jogo);

        for j in jogadores {
            println!("{:?}", j)
        }
    }

    pub fn jogadores_ls_all(conn: &mut Connection) {
        let jogadores = crate::db::jogador::get_all_jogadores(conn);

        for j in jogadores {
            println!("{:?}", j)
        }
    }

    pub fn jogadores_add(conn: &mut Connection, jogo: u64, nome: String, email: String) {
        let id = crate::db::jogador::create_jogador(conn, jogo, nome, email);
        println!("Criado jogador com id {id}")
    }

    pub fn jogadores_inspect(conn: &mut Connection, id: u64) {
        let jogador = crate::db::jogador::get_jogador_by_id(conn, id);

        println!("{:#?}", jogador);
    }

    pub fn jogadores_set(conn: &mut Connection, id: u64, param: JogadoresSetParams) {
        let (collumn, new_value) = match param {
            JogadoresSetParams::Email { val } => ("email".to_string(), val),
            JogadoresSetParams::Nome { val } => ("nome".to_string(), val),
        };

        crate::db::jogador::update_jogador_by_collumn(conn, &collumn, new_value, id);

        println!("Atualizada propriedade {collumn} do id {id}");
    }

    pub fn jogadores_rm(conn: &mut Connection, id: u64) {
        let id = crate::db::jogador::delete_jogador_by_id(conn, id);

        println!("Removido jogador de id {id}")
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
        let jogadores = crate::db::jogador::get_jogadores_by_jogo(conn, jogo);

        let mut jogadores_ids = jogadores.iter().map(|x| x.id).collect::<Vec<u64>>();
        jogadores_ids.sort();

        let mut hasher = std::hash::DefaultHasher::new();
        jogadores_ids.hash(&mut hasher);

        let id = crate::db::sorteio::create_sorteio(conn, &seed, jogo, hasher, jogadores);

        println!("Criado sorteio com id {id} e semente {seed}");
        println!("Use o comando `sorteio run` para rodá-lo");
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

        let results = crate::envio::run_and_email(sorteio, jogadores, smtp_ctx, conn);

        for r in crate::db::envios::get_envios_by_sorteio(conn, id) {
            if r.sucesso {
                println!("Envio para id {} exitoso", r.destino)
            } else {
                println!(
                    "Envio para id {} não-exitoso com mensagem de erro {:?}",
                    r.destino, r.erro
                )
            }
        }

        println!("Resultados completos: {:#?}", results);
    }

    pub fn sorteios_ls_by_jogo(conn: &mut Connection, jogo: u64) {
        let sorteios = crate::db::sorteio::get_sorteios_by_jogo(conn, jogo);

        for s in sorteios {
            println!("{:?}", s)
        }
    }

    pub fn sorteio_ls(conn: &mut Connection) {
        let sorteios = crate::db::sorteio::get_sorteios(conn);

        for s in sorteios {
            println!("{:?}", s)
        }
    }

    pub fn sorteio_inspect(conn: &mut Connection, id: u64) {
        let sorteio = crate::db::sorteio::get_sorteio_by_id(conn, &id);

        println!("{:#?}", sorteio)
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

        processo.enviar(transport, conn, ctx).unwrap();
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
