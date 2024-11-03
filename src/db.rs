use rusqlite::Connection;

use crate::config::Config;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("migrations");
}

pub fn make_conn(config: &Config) -> Connection {
    let mut conn = Connection::open(config.db_path.clone()).unwrap();

    conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

    embedded::migrations::runner().run(&mut conn).unwrap();

    conn
}

#[derive(Debug, Clone)]
pub struct Jogo {
    pub id: u64,
    pub nome: String,
}

#[derive(Debug, Clone)]
pub struct Jogador {
    pub id: u64,
    pub nome: String,
    pub email: String,
    pub jogo: u64,
}

#[derive(Debug, Clone)]
pub struct Sorteio {
    pub id: u64,
    pub seed: String,
    pub jogadores_hash: String,
    pub jogadores_qtd: u64,
    pub jogo: u64,
}

#[derive(Debug, Clone)]
pub struct Envio {
    pub id: u64,
    pub sorteio: u64,
    pub destino: u64,
    pub sorteado: u64,
    pub sucesso: bool,
    pub erro: Option<String>,
}

/// Contém funções que abstraem TODAS as conexões com a base de dados
/// relacionadas á estrutura `Jogo`
pub mod jogo {
    use super::Jogo;
    use rusqlite::Connection;

    pub fn get_all_jogos(conn: &mut Connection) -> Vec<Jogo> {
        let mut query = conn.prepare("SELECT id, nome FROM jogos").unwrap();
        let jogos = query
            .query_map((), |row| {
                Ok(Jogo {
                    id: row.get(0).unwrap(),
                    nome: row.get(1).unwrap(),
                })
            })
            .unwrap()
            .map(|x| x.unwrap())
            .collect::<Vec<Jogo>>();
        jogos
    }

    pub fn create_jogo_with_nome(conn: &mut Connection, nome: &String) -> usize {
        let id = conn
            .execute("INSERT INTO jogos (nome) VALUES (?1)", [nome.clone()])
            .unwrap();
        id
    }

    pub fn delete_jogo_by_id(conn: &mut Connection, id: u64) -> usize {
        let id = conn.execute("DELETE FROM jogos WHERE id=?1", [id]).unwrap();
        id
    }
}

pub mod jogador {
    use super::Jogador;
    use rusqlite::{params, Connection};

    pub fn get_jogadores_by_jogo(conn: &mut Connection, jogo: u64) -> Vec<Jogador> {
        let mut query = conn
            .prepare("SELECT id, nome, email, jogo FROM jogadores WHERE jogo=?1")
            .unwrap();

        let jogadores = query
            .query_map([jogo.to_string()], |row| Ok(extract_jogador(row)))
            .unwrap()
            .map(|x| x.unwrap())
            .collect::<Vec<Jogador>>();

        jogadores
    }

    pub fn create_jogador(conn: &mut Connection, jogo: u64, nome: String, email: String) -> usize {
        conn.execute(
            "INSERT INTO jogadores (jogo, nome, email) VALUES (?1, ?2, ?3)",
            [jogo.to_string(), nome, email],
        )
        .unwrap()
    }

    pub fn get_jogador_by_id(conn: &mut Connection, id: u64) -> Jogador {
        let mut query = conn
            .prepare("SELECT id, nome, email, jogo FROM jogadores WHERE id=?1")
            .unwrap();

        let mut rows = query.query([id.to_string()]).unwrap();
        let jogador_row = rows.next().unwrap().unwrap();

        let jogador = extract_jogador(jogador_row);
        jogador
    }

    pub fn update_jogador_by_collumn(
        conn: &mut Connection,
        collumn: &String,
        new_value: String,
        id: u64,
    ) {
        conn.execute(
            &format!("UPDATE jogadores SET {collumn} = ?1 WHERE id=?2"),
            params![new_value, id],
        )
        .unwrap();
    }

    pub fn delete_jogador_by_id(conn: &mut Connection, id: u64) -> usize {
        let id = conn
            .execute("DELETE FROM jogadores WHERE id=?1", params![id])
            .unwrap();
        id
    }

    fn extract_jogador(row: &rusqlite::Row<'_>) -> Jogador {
        Jogador {
            id: row.get(0).unwrap(),
            nome: row.get(1).unwrap(),
            email: row.get(2).unwrap(),
            jogo: row.get(3).unwrap(),
        }
    }
}

pub mod sorteio {
    use std::hash::Hasher;

    use super::{Jogador, Sorteio};
    use rusqlite::{params, Connection};

    pub fn get_sorteio_by_id(conn: &mut Connection, id: &u64) -> Sorteio {
        let mut query = conn
            .prepare(
                "SELECT id, seed, jogadores_hash, jogadores_qtd, jogo FROM sorteios WHERE id=?1",
            )
            .unwrap();
        query
            .query_row(params![id], |x| Ok(extract_sorteio(x)))
            .unwrap()
    }

    pub fn get_sorteios_by_jogo(conn: &mut Connection, jogo: u64) -> Vec<Sorteio> {
        let mut query = conn
            .prepare(
                "SELECT id, seed, jogadores_hash, jogadores_qtd, jogo FROM sorteios WHERE jogo=?1",
            )
            .unwrap();

        query
            .query_map(params![jogo], |x| Ok(extract_sorteio(x)))
            .unwrap()
            .map(|x| x.unwrap())
            .collect()
    }

    pub fn get_sorteios(conn: &mut Connection) -> Vec<Sorteio> {
        let mut query = conn
            .prepare("SELECT id, seed, jogadores_hash, jogadores_qtd, jogo FROM sorteios")
            .unwrap();

        query
            .query_map(params![], |x| Ok(extract_sorteio(x)))
            .unwrap()
            .map(|x| x.unwrap())
            .collect()
    }

    pub fn create_sorteio<T: Hasher>(
        conn: &mut Connection,
        seed: &String,
        jogo: u64,
        hasher: T,
        jogadores: Vec<Jogador>,
    ) {
        conn
            .execute(
                "INSERT INTO sorteios (seed, jogo, jogadores_hash, jogadores_qtd) VALUES (?1, ?2, ?3, ?4)",
                params![seed.to_string(), jogo, hasher.finish().to_string(), jogadores.len()],
            )
            .unwrap();
    }

    fn extract_sorteio(row: &rusqlite::Row<'_>) -> Sorteio {
        return Sorteio {
            id: row.get(0).unwrap(),
            seed: row.get(1).unwrap(),
            jogadores_hash: row.get(2).unwrap(),
            jogadores_qtd: row.get(3).unwrap(),
            jogo: row.get(4).unwrap(),
        };
    }
}

pub mod envios {
    use super::Envio;
    use rusqlite::{params, Connection};

    pub fn get_envios_by_sorteio(conn: &mut Connection, sorteio: u64) -> Vec<Envio> {
        let mut query = conn
            .prepare(
                "SELECT id, sorteio, destino, sorteado, sucesso, erro FROM envios WHERE sorteio=?1",
            )
            .unwrap();
        query
            .query_map(params![sorteio], |x| extract_envio(x))
            .unwrap()
            .map(|x| x.unwrap())
            .collect()
    }

    fn extract_envio(x: &rusqlite::Row<'_>) -> Result<Envio, rusqlite::Error> {
        Ok(Envio {
            id: x.get(0).unwrap(),
            sorteio: x.get(1).unwrap(),
            destino: x.get(2).unwrap(),
            sorteado: x.get(3).unwrap(),
            sucesso: x.get(4).unwrap(),
            erro: x.get(5).unwrap(),
        })
    }
}
