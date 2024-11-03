pub mod actions;
pub mod cli;
pub mod config;
pub mod db;
pub mod envio;

use crate::cli::{Arguments, Commands};
use clap::Parser;
use cli::{JogadoresAction, JogoAction, SorteioAction};
use config::Config;
use rusqlite::Connection;

fn main() {
    tracing_subscriber::fmt::init();
    let args = Arguments::parse();
    let config = crate::config::Config::from_env();

    let mut conn = db::make_conn(&config);

    exec_args(args, &mut conn, &config);
}

fn exec_args(args: Arguments, conn: &mut Connection, ctx: &Config) {
    match args.cmd {
        Commands::Jogo { action } => match action {
            JogoAction::Ls => actions::jogo::jogo_ls(conn),
            JogoAction::New { name: nome } => actions::jogo::jogo_new(conn, nome),
            JogoAction::Rm { id } => actions::jogo::jogo_rm(conn, id),
        },

        Commands::Jogadores { action } => match action {
            JogadoresAction::Ls { jogo } => match jogo {
                Some(j) => actions::jogador::jogadores_ls_with_jogo(conn, j),
                None => actions::jogador::jogadores_ls_all(conn),
            },
            JogadoresAction::Add { jogo, nome, email } => {
                actions::jogador::jogadores_add(conn, jogo, nome, email)
            }
            JogadoresAction::Inspect { jogador } => {
                actions::jogador::jogadores_inspect(conn, jogador)
            }
            JogadoresAction::Set { id, param } => actions::jogador::jogadores_set(conn, id, param),
            JogadoresAction::Rm { jogador } => actions::jogador::jogadores_rm(conn, jogador),
        },

        Commands::Sorteio { action } => match action {
            SorteioAction::New { jogo } => actions::sorteio::sorteio_new(conn, jogo),
            SorteioAction::Run { sorteio } => actions::sorteio::sorteio_run(conn, sorteio, ctx),
            SorteioAction::Ls { jogo } => match jogo {
                Some(j) => actions::sorteio::sorteios_ls_by_jogo(conn, j),
                None => actions::sorteio::sorteio_ls(conn),
            },
            SorteioAction::Inspect { sorteio } => actions::sorteio::sorteio_inspect(conn, sorteio),
        },

        Commands::Envio { action } => match action {
            cli::EnvioAction::Inspect { envio } => actions::envio::envio_inspect(conn, envio),
            cli::EnvioAction::Redo { envio } => actions::envio::envio_redo(conn, ctx, envio),
            cli::EnvioAction::Ls { sorteio } => match sorteio {
                Some(s) => actions::envio::envio_ls_with_sorteio(conn, s),
                None => actions::envio::envio_ls_all(conn),
            },
        },
    }
}
